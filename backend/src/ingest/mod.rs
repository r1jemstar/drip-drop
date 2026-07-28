//! Forgiving feed ingest — CSV / TSV / JSONL / JSON / XML → items + price_history.
//! Column names vary per advertiser. We alias aggressively, skip junk rows, never panic.

use anyhow::{Result, anyhow};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::pricing::record_price;

/// One raw row = loose key→value bag. Keys lowercased, trimmed.
type Row = HashMap<String, String>;

/// Normalized product, ready for DB.
#[derive(Debug, Clone)]
pub struct RawItem {
    pub sku:           String,
    pub name:          String,
    pub brand:         String,
    pub category:      String,
    pub current_price: f64,
    pub was_price:     Option<f64>,
    pub currency:      String,
    pub affiliate_url: String,
    pub image_url:     Option<String>,
    pub sizes:         Vec<String>,
    pub in_stock:      bool,
    pub region:        String,
}

// ── Column aliases. Add freely — first hit wins. ──
const A_SKU:   &[&str] = &["sku","product_id","aw_product_id","id","merchant_product_id","gtin","mpn","ean"];
const A_NAME:  &[&str] = &["name","product_name","title","product_title","display_name"];
const A_BRAND: &[&str] = &["brand","brand_name","manufacturer","merchant_name","advertiser","vendor"];
const A_CAT:   &[&str] = &["category","merchant_category","category_name","product_type","aw_product_category","google_product_category","department"];
const A_PRICE: &[&str] = &["search_price","price","sale_price","current_price","display_price","product_price","final_price"];
const A_WAS:   &[&str] = &["rrp_price","rrp","was_price","base_price","list_price","regular_price","msrp","original_price","store_price"];
const A_CUR:   &[&str] = &["currency","currency_code","price_currency"];
const A_URL:   &[&str] = &["aw_deep_link","deep_link","affiliate_url","link","product_url","merchant_deep_link","url","tracking_url"];
const A_IMG:   &[&str] = &["large_image","merchant_image_url","aw_image_url","image_url","image","image_link","thumb_url"];
const A_SIZE:  &[&str] = &["size","sizes","variant_size","available_sizes","size_stock_amount"];
const A_STOCK: &[&str] = &["in_stock","stock_status","availability","is_available","stock"];

fn pick(row: &Row, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = row.get(*k) {
            let v = v.trim();
            if !v.is_empty() { return Some(v.to_string()); }
        }
    }
    None
}

/// Parse money from anything: "£34.99", "34,99 EUR", "C$ 1,299.00", "34.99"
fn parse_money(s: &str) -> Option<f64> {
    let cleaned: String = s.chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
        .collect();
    if cleaned.is_empty() { return None; }

    // Decide decimal separator: last , or . that has <=2 trailing digits
    let last_dot = cleaned.rfind('.');
    let last_com = cleaned.rfind(',');
    let normalized = match (last_dot, last_com) {
        (Some(d), Some(c)) => {
            // whichever is further right is the decimal sep
            if d > c { cleaned.replace(',', "") }           // 1,299.00
            else { cleaned.replace('.', "").replace(',', ".") } // 1.299,00
        }
        (Some(_), None) => {
            // could be 1.299 (thousands) or 34.99 (decimal)
            let tail = cleaned.split('.').last().unwrap_or("");
            if tail.len() == 3 && cleaned.matches('.').count() == 1 && cleaned.len() > 4 {
                cleaned.replace('.', "")
            } else { cleaned.clone() }
        }
        (None, Some(_)) => {
            let tail = cleaned.split(',').last().unwrap_or("");
            if tail.len() <= 2 { cleaned.replace(',', ".") } else { cleaned.replace(',', "") }
        }
        (None, None) => cleaned.clone(),
    };
    normalized.parse::<f64>().ok().filter(|p| *p > 0.0 && p.is_finite())
}

/// Currency from explicit field, else sniff symbol in price string, else region default.
fn parse_currency(explicit: Option<&str>, price_raw: &str, region: &str) -> String {
    if let Some(c) = explicit {
        let c = c.trim().to_uppercase();
        if c.len() == 3 { return c; }
    }
    if price_raw.contains('£') { return "GBP".into(); }
    if price_raw.contains('€') { return "EUR".into(); }
    if price_raw.contains("C$") || price_raw.contains("CAD") { return "CAD".into(); }
    if price_raw.contains('$') {
        return match region { "CA" => "CAD".into(), _ => "USD".into() };
    }
    match region { "GB" => "GBP", "US" => "USD", "CA" => "CAD", _ => "CAD" }.to_string()
}

/// Map messy category strings onto our 4 buckets.
fn normalize_category(raw: &str) -> String {
    let s = raw.to_lowercase();
    let words: std::collections::HashSet<&str> = s
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let hasw = |terms: &[&str]| terms.iter().any(|t| words.contains(t));
    let hass = |terms: &[&str]| terms.iter().any(|t| s.contains(t));

    if hasw(&["shoe","shoes","footwear","trainer","trainers","sneaker","sneakers","boot","boots","sandal","sandals","heel","heels","loafer","loafers"]) { return "footwear".into(); }
    if hasw(&["bag","bags","handbag","backpack","purse","wallet","hat","scarf","belt","watch","sunglasses","glove","gloves"]) || hass(&["accessor","jewel"]) { return "accessories".into(); }
    if hasw(&["workwear","vis","safety","overall","overalls","coverall","tabard"]) || hass(&["hi-vis","hi vis"]) { return "workwear".into(); }
    if hasw(&["men","mens"]) { return "menswear".into(); }
    "womenswear".into()
}

fn parse_stock(v: Option<String>) -> bool {
    match v {
        None => true, // absent = assume available
        Some(s) => {
            let s = s.trim().to_lowercase();
            !matches!(s.as_str(), "0"|"false"|"no"|"n"|"out of stock"|"outofstock"|"oos"|"unavailable"|"discontinued")
        }
    }
}

fn parse_sizes(v: Option<String>) -> Vec<String> {
    match v {
        None => vec![],
        Some(s) => s.split(|c| c == ',' || c == '|' || c == ';')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty() && x.len() <= 8)
            .take(12)
            .collect(),
    }
}

/// Row → RawItem. Returns None (skip) rather than erroring — feeds have junk rows.
fn row_to_item(row: &Row, region: &str, fallback_brand: &str) -> Result<RawItem, &'static str> {
    let price_raw = pick(row, A_PRICE).ok_or("no_price_column")?;
    let current_price = parse_money(&price_raw).ok_or("price_unparseable")?;
    let name = pick(row, A_NAME).ok_or("no_name")?;
    let affiliate_url = pick(row, A_URL).ok_or("no_url")?;
    if !affiliate_url.starts_with("http") { return Err("bad_url"); }

    let sku = pick(row, A_SKU).unwrap_or_else(|| format!("{:x}", md5_lite(&affiliate_url)));
    let was_price = pick(row, A_WAS).and_then(|w| parse_money(&w)).filter(|w| *w > current_price);
    let currency = parse_currency(pick(row, A_CUR).as_deref(), &price_raw, region);
    let brand = pick(row, A_BRAND).unwrap_or_else(|| fallback_brand.to_string());
    let category = normalize_category(&pick(row, A_CAT).unwrap_or_default());

    Ok(RawItem {
        sku,
        name: name.chars().take(300).collect(),
        brand, category, current_price, was_price, currency, affiliate_url,
        image_url: pick(row, A_IMG).filter(|u| u.starts_with("http")),
        sizes: parse_sizes(pick(row, A_SIZE)),
        in_stock: parse_stock(pick(row, A_STOCK)),
        region: region.to_string(),
    })
}

/// Tiny non-crypto hash so we can synthesize SKUs without a new dep.
fn md5_lite(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() { h ^= b as u64; h = h.wrapping_mul(1099511628211); }
    h
}

// ── FORMAT DETECTION + READERS ──

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format { Csv, Tsv, Jsonl, Json, Xml }

/// Sniff format from bytes + url. Never guesses blindly.
pub fn detect_format(url: &str, bytes: &[u8]) -> Format {
    let head: String = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_string();
    let t = head.trim_start();
    if t.starts_with('<') { return Format::Xml; }
    if t.starts_with('[') { return Format::Json; }
    if t.starts_with('{') {
        // { on line 1 + } newline { = jsonl; single { = json doc
        return if t.lines().take(3).filter(|l| l.trim_start().starts_with('{')).count() > 1
            { Format::Jsonl } else { Format::Json };
    }
    let u = url.to_lowercase();
    if u.contains(".tsv") { return Format::Tsv; }
    if u.contains(".jsonl") || u.contains(".ndjson") { return Format::Jsonl; }
    // delimiter vote on first line
    let first = t.lines().next().unwrap_or("");
    if first.matches('\t').count() > first.matches(',').count() { Format::Tsv } else { Format::Csv }
}

fn flatten_json(v: &serde_json::Value, out: &mut Row, prefix: &str) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, val) in m {
                let key = if prefix.is_empty() { k.clone() } else { format!("{prefix}_{k}") };
                flatten_json(val, out, &key);
            }
        }
        serde_json::Value::Array(a) => {
            let joined: Vec<String> = a.iter().map(|x| match x {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string().trim_matches('"').to_string(),
            }).collect();
            out.insert(prefix.to_lowercase(), joined.join(","));
        }
        serde_json::Value::Null => {}
        other => {
            let s = match other {
                serde_json::Value::String(s) => s.clone(),
                o => o.to_string(),
            };
            out.insert(prefix.to_lowercase(), s);
        }
    }
}

pub fn read_rows(bytes: &[u8], fmt: Format) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    match fmt {
        Format::Csv | Format::Tsv => {
            let delim = if fmt == Format::Tsv { b'\t' } else { b',' };
            let mut rdr = csv::ReaderBuilder::new()
                .delimiter(delim)
                .flexible(true)          // ← ragged rows OK
                .has_headers(true)
                .from_reader(bytes);
            let headers: Vec<String> = rdr.headers()?
                .iter().map(|h| h.trim().to_lowercase()).collect();
            for rec in rdr.records() {
                let Ok(rec) = rec else { continue }; // skip bad row, don't abort
                let mut row = Row::new();
                for (i, val) in rec.iter().enumerate() {
                    if let Some(h) = headers.get(i) {
                        row.insert(h.clone(), val.to_string());
                    }
                }
                rows.push(row);
            }
        }
        Format::Jsonl => {
            for line in String::from_utf8_lossy(bytes).lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { continue };
                let mut row = Row::new();
                flatten_json(&v, &mut row, "");
                rows.push(row);
            }
        }
        Format::Json => {
            let v: serde_json::Value = serde_json::from_slice(bytes)?;
            // find the array: root, or first array-valued key (products/items/data/results)
            let arr = if v.is_array() { v }
                else {
                    v.as_object()
                        .and_then(|m| ["products","items","data","results","offers","feed"].iter()
                            .find_map(|k| m.get(*k).filter(|x| x.is_array()).cloned())
                            .or_else(|| m.values().find(|x| x.is_array()).cloned()))
                        .ok_or_else(|| anyhow!("no array found in JSON feed"))?
                };
            for item in arr.as_array().unwrap_or(&vec![]) {
                let mut row = Row::new();
                flatten_json(item, &mut row, "");
                rows.push(row);
            }
        }
        Format::Xml => {
            use quick_xml::events::Event;
            use quick_xml::Reader;
            let text = String::from_utf8_lossy(bytes);
            let mut rdr = Reader::from_str(&text);
            rdr.trim_text(true);
            let mut buf = Vec::new();
            let mut cur: Row = Row::new();
            let mut depth = 0usize;
            let mut tag = String::new();
            loop {
                match rdr.read_event_into(&mut buf) {
                    Ok(Event::Start(e)) => {
                        depth += 1;
                        tag = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                        // attributes count as fields too
                        for a in e.attributes().flatten() {
                            let k = String::from_utf8_lossy(a.key.as_ref()).to_lowercase();
                            let val = a.unescape_value().unwrap_or_default().to_string();
                            cur.insert(k, val);
                        }
                    }
                    Ok(Event::Text(t)) => {
                        let val = t.unescape().unwrap_or_default().trim().to_string();
                        if !val.is_empty() && !tag.is_empty() {
                            cur.insert(tag.clone(), val);
                        }
                    }
                    Ok(Event::End(_)) => {
                        depth = depth.saturating_sub(1);
                        // product boundary: heuristic — row has a price + name
                        if depth <= 2 && !cur.is_empty() {
                            if pick(&cur, A_PRICE).is_some() && pick(&cur, A_NAME).is_some() {
                                rows.push(std::mem::take(&mut cur));
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }
            if pick(&cur, A_PRICE).is_some() { rows.push(cur); }
        }
    }
    Ok(rows)
}

/// Fetch bytes. Handles gzip + plain. Feed URLs are often .gz.
pub async fn fetch_feed(url: &str) -> Result<Vec<u8>> {
    let resp = reqwest::Client::new()
        .get(url)
        .header("User-Agent", "DripDrop/0.1 (+https://drip-drop-2vk.pages.dev)")
        .send().await?
        .error_for_status()?;
    let bytes = resp.bytes().await?.to_vec();
    // gzip magic
    if bytes.len() > 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
        use std::io::Read;
        let mut d = flate2::read::GzDecoder::new(&bytes[..]);
        let mut out = Vec::new();
        d.read_to_end(&mut out)?;
        return Ok(out);
    }
    Ok(bytes)
}

#[derive(Debug, Default)]
pub struct IngestReport {
    pub rows_seen:   usize,
    pub parsed:      usize,
    pub upserted:    usize,
    pub drops_found: usize,
    pub skips:       HashMap<String, usize>,  // reason → count
    pub errors:      Vec<String>,
}

impl IngestReport {
    fn skip(&mut self, reason: &str) {
        *self.skips.entry(reason.to_string()).or_insert(0) += 1;
    }
}

/// "Let's Swim & Co." → "lets-swim-co"
fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = true; // trims leading dashes
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') { out.pop(); }
    if out.is_empty() { out.push_str("brand"); }
    out.chars().take(80).collect()
}

async fn upsert_brand(pool: &PgPool, name: &str) -> Result<Uuid> {
    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM brands WHERE lower(name) = lower($1)")
        .bind(name).fetch_optional(pool).await?;
    if let Some((id,)) = row { return Ok(id); }

    let slug = slugify(name);
    let (id,): (Uuid,) = sqlx::query_as(
        r#"INSERT INTO brands (name, slug) VALUES ($1, $2)
           ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
           RETURNING id"#
    )
    .bind(name).bind(&slug).fetch_one(pool).await?;
    Ok(id)
}

async fn upsert_item(pool: &PgPool, it: &RawItem, brand_id: Uuid) -> Result<Uuid> {
    // existing?
    let found: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM items WHERE sku = $1 AND region = $2"
    ).bind(&it.sku).bind(&it.region).fetch_optional(pool).await?;

    if let Some((id,)) = found {
        sqlx::query(
            r#"UPDATE items SET name=$2, brand_id=$3, category=$4::text::category,
                affiliate_url=$5, image_url=COALESCE($6, image_url), sizes=$7,
                in_stock=$8, currency=$9, updated_at=NOW(), last_seen_at=NOW()
                WHERE id=$1"#
        )
        .bind(id).bind(&it.name).bind(brand_id).bind(&it.category)
        .bind(&it.image_url).bind(&it.sizes)
        .bind(it.in_stock).bind(&it.currency)
        .execute(pool).await?;
        return Ok(id);
    }

    let was = it.was_price.unwrap_or(it.current_price);
    let (id,): (Uuid,) = sqlx::query_as(
        r#"INSERT INTO items (sku, name, brand_id, category, current_price, was_price,
                                drop_percent, currency, affiliate_url, image_url, sizes, in_stock, region)
            VALUES ($1,$2,$3,$4::text::category,$5,$6,0,$7,$8,$9,$10,$11,$12) RETURNING id"#
    )
    .bind(&it.sku).bind(&it.name).bind(brand_id).bind(&it.category)
    .bind(it.current_price).bind(was).bind(&it.currency)
    .bind(&it.affiliate_url).bind(&it.image_url).bind(&it.sizes).bind(it.in_stock).bind(&it.region)
    .fetch_one(pool).await?;
    Ok(id)
}

/// Main entry. Forgiving: one bad row never kills the run.
pub async fn ingest_feed(
    pool: &PgPool,
    url: &str,
    region: &str,
    fallback_brand: &str,
) -> Result<IngestReport> {
    let mut rep = IngestReport::default();
    let bytes = fetch_feed(url).await?;
    let fmt = detect_format(url, &bytes);
    let rows = read_rows(&bytes, fmt)?;
    rep.rows_seen = rows.len();

    for row in &rows {
            let item = match row_to_item(row, region, fallback_brand) {
                Ok(i) => i,
                Err(reason) => { rep.skip(reason); continue; }
        };
        rep.parsed += 1;

        let res: Result<()> = async {
            let brand_id = upsert_brand(pool, &item.brand).await?;
            let item_id = upsert_item(pool, &item, brand_id).await?;
            let upd = record_price(pool, item_id, item.current_price).await?;
            if upd.dropped { rep.drops_found += 1; }
            Ok(())
        }.await;

        match res {
                    Ok(_) => rep.upserted += 1,
                    Err(e) => {
                        rep.skip("db_error");
                        if rep.errors.len() < 20 { rep.errors.push(format!("{}: {e}", item.sku)); }
            }
        }
    }
    Ok(rep)
}

#[derive(Debug, sqlx::FromRow)]
pub struct FeedRow {
    pub id:             Uuid,
    pub label:          String,
    pub url:            String,
    pub region:         String,
    pub fallback_brand: String,
}

/// Ingest every feed whose interval has elapsed. Returns per-feed reports.
pub async fn run_due_feeds(pool: &PgPool) -> Result<Vec<(String, IngestReport)>> {
    let due: Vec<FeedRow> = sqlx::query_as(
        r#"
        SELECT id, label, url, region, fallback_brand
        FROM feeds
        WHERE active
          AND (last_run_at IS NULL
               OR last_run_at < NOW() - (interval_hours || ' hours')::interval)
        ORDER BY last_run_at NULLS FIRST
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for f in due {
        let res = ingest_feed(pool, &f.url, &f.region, &f.fallback_brand).await;
        let status = match &res {
            Ok(r)  => format!("ok: {} upserted, {} drops", r.upserted, r.drops_found),
            Err(e) => format!("error: {e}"),
        };
        sqlx::query("UPDATE feeds SET last_run_at = NOW(), last_status = $2 WHERE id = $1")
            .bind(f.id).bind(&status)
            .execute(pool).await.ok();

        match res {
            Ok(r)  => out.push((f.label, r)),
            Err(e) => {
                let mut rep = IngestReport::default();
                rep.errors.push(e.to_string());
                out.push((f.label, rep));
            }
        }
    }
    Ok(out)
}


#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_money ──
    #[test]
    fn money_plain() {
        assert_eq!(parse_money("34.99"), Some(34.99));
        assert_eq!(parse_money("129"), Some(129.0));
        assert_eq!(parse_money("0.50"), Some(0.50));
    }

    #[test]
    fn money_with_symbols() {
        assert_eq!(parse_money("£34.99"), Some(34.99));
        assert_eq!(parse_money("C$ 1,299.00"), Some(1299.00));
        assert_eq!(parse_money("$19.95 USD"), Some(19.95));
        assert_eq!(parse_money("€ 45"), Some(45.0));
    }

    #[test]
    fn money_thousands_us() {
        // comma thousands, dot decimal
        assert_eq!(parse_money("1,299.00"), Some(1299.00));
        assert_eq!(parse_money("12,345.67"), Some(12345.67));
    }

    #[test]
    fn money_european() {
        // dot thousands, comma decimal
        assert_eq!(parse_money("1.299,00"), Some(1299.00));
        assert_eq!(parse_money("34,99"), Some(34.99));
        assert_eq!(parse_money("1.234.567,89"), Some(1234567.89));
    }

    #[test]
    fn money_ambiguous_single_separator() {
        // "1.299" — is it 1299 (thousands) or 1.299 (decimal)?
        // 3 trailing digits + long enough → treated as thousands
        assert_eq!(parse_money("1.299"), Some(1299.0));
        // "34.99" — 2 trailing → decimal
        assert_eq!(parse_money("34.99"), Some(34.99));
        // "1,299" comma + 3 trailing → thousands
        assert_eq!(parse_money("1,299"), Some(1299.0));
        // "34,5" comma + <=2 trailing → decimal
        assert_eq!(parse_money("34,5"), Some(34.5));
    }

    #[test]
    fn money_junk_and_empty() {
        assert_eq!(parse_money(""), None);
        assert_eq!(parse_money("free"), None);
        assert_eq!(parse_money("N/A"), None);
        assert_eq!(parse_money("£"), None);
        assert_eq!(parse_money("--"), None);
    }

    #[test]
    fn money_rejects_zero_and_negative() {
        // guard: price must be > 0 and finite
        assert_eq!(parse_money("0"), None);
        assert_eq!(parse_money("0.00"), None);
        assert_eq!(parse_money("-5.00"), None);
    }

    // ── normalize_category ──
    #[test]
    fn category_footwear() {
        assert_eq!(normalize_category("Women's Shoes"), "footwear");
        assert_eq!(normalize_category("SNEAKERS"), "footwear");
        assert_eq!(normalize_category("Chelsea Boot"), "footwear");
        assert_eq!(normalize_category("high heel sandal"), "footwear");
    }

    #[test]
    fn category_accessories() {
        assert_eq!(normalize_category("Handbag"), "accessories");
        assert_eq!(normalize_category("Gold Jewellery"), "accessories");
        assert_eq!(normalize_category("Leather Belt"), "accessories");
    }

    #[test]
    fn category_menswear() {
        assert_eq!(normalize_category("Mens Shirts"), "menswear");
        assert_eq!(normalize_category("men's trousers"), "menswear");
    }

    #[test]
    fn category_workwear() {
        assert_eq!(normalize_category("Hi-Vis Jacket"), "workwear");
        assert_eq!(normalize_category("safety coverall"), "workwear");
    }

    #[test]
    fn category_default_womenswear() {
        // unknown / empty falls through to womenswear
        assert_eq!(normalize_category(""), "womenswear");
        assert_eq!(normalize_category("Midi Dress"), "womenswear");
        assert_eq!(normalize_category("random garbage"), "womenswear");
    }

    #[test]
    fn category_precedence() {
        // footwear checked before menswear — "men's shoes" is footwear, not menswear
        assert_eq!(normalize_category("Men's Shoes"), "footwear");
    }

    // ── slugify ──
    #[test]
    fn slug_basic() {
        assert_eq!(slugify("Lululemon"), "lululemon");
        assert_eq!(slugify("New Look"), "new-look");
    }

    #[test]
    fn slug_messy() {
        assert_eq!(slugify("Let's Swim & Co."), "let-s-swim-co");
        assert_eq!(slugify("encalife (US & Canada)"), "encalife-us-canada");
        assert_eq!(slugify("  spaced  out  "), "spaced-out");
        assert_eq!(slugify("Needs No Label"), "needs-no-label");
    }

    #[test]
    fn slug_edge() {
        assert_eq!(slugify(""), "brand");
        assert_eq!(slugify("!!!"), "brand");
        assert_eq!(slugify("---"), "brand");
    }

    // ── parse_stock ──
    #[test]
    fn stock_truthy() {
        assert!(parse_stock(None)); // absent = assume in stock
        assert!(parse_stock(Some("1".into())));
        assert!(parse_stock(Some("true".into())));
        assert!(parse_stock(Some("in stock".into())));
        assert!(parse_stock(Some("yes".into())));
    }

    #[test]
    fn stock_falsy() {
        assert!(!parse_stock(Some("0".into())));
        assert!(!parse_stock(Some("false".into())));
        assert!(!parse_stock(Some("out of stock".into())));
        assert!(!parse_stock(Some("OOS".into())));
        assert!(!parse_stock(Some("unavailable".into())));
    }

    // ── parse_sizes ──
    #[test]
    fn sizes_delimiters() {
        assert_eq!(parse_sizes(Some("S,M,L".into())), vec!["S","M","L"]);
        assert_eq!(parse_sizes(Some("S|M|L".into())), vec!["S","M","L"]);
        assert_eq!(parse_sizes(Some("S; M; L".into())), vec!["S","M","L"]);
    }

    #[test]
    fn sizes_empty_and_junk() {
        assert_eq!(parse_sizes(None), Vec::<String>::new());
        assert_eq!(parse_sizes(Some("".into())), Vec::<String>::new());
        assert_eq!(parse_sizes(Some(",,,".into())), Vec::<String>::new());
    }
}

#[test]
    fn category_women_not_men() {
        assert_eq!(normalize_category("Womens Dress"), "womenswear");
        assert_eq!(normalize_category("Women's Blazer"), "womenswear");
    }