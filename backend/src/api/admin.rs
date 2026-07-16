//! Admin endpoints for manually adding real products and updating prices.
//! for manually adding the beginning data

use axum::{extract::{Query,State}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::pricing::record_price;

#[derive(Deserialize)]
pub struct NewProduct {
    pub sku:           String,
    pub brand_slug:    String,   // must already exist in brands, e.g. "aritzia"
    pub name:          String,
    pub category:      String,   // 'womenswear' | 'footwear' | ...
    pub price:         f64,
    pub was_price:     f64,
    pub affiliate_url: String,
    pub sizes:         Vec<String>,
    pub region:        String,   // 'GB' | 'US' | 'CA'
    pub currency:      String,   // 'GBP' | 'USD' | 'CAD'
}

#[derive(Serialize)]
pub struct AddResult { pub id: Uuid, pub drop_percent: f64 }

pub async fn add_product(
    State(pool): State<PgPool>,
    Json(p): Json<NewProduct>,
) -> Result<Json<AddResult>, (StatusCode, String)> {
    let err = |e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());

    // Find the brand by slug + region
    let brand: (Uuid,) = sqlx::query_as(
        "SELECT id FROM brands WHERE slug = $1 AND region = $2",
    )
    .bind(&p.brand_slug)
    .bind(&p.region)
    .fetch_optional(&pool)
    .await
    .map_err(err)?
    .ok_or((StatusCode::BAD_REQUEST, format!("brand '{}' not found in region {}", p.brand_slug, p.region)))?;

    // Insert the item (upsert on sku+brand+region)
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO items
            (sku, brand_id, name, category, current_price, was_price,
             drop_percent, affiliate_url, image_url, sizes, region, currency)
        VALUES ($1,$2,$3,$4::category,$5,$6,0,$7,'',$8,$9,$10)
        ON CONFLICT (sku, brand_id, region)
        DO UPDATE SET name = EXCLUDED.name, affiliate_url = EXCLUDED.affiliate_url
        RETURNING id
        "#,
    )
    .bind(&p.sku).bind(brand.0).bind(&p.name).bind(&p.category)
    .bind(p.price).bind(p.was_price)
    .bind(&p.affiliate_url).bind(&p.sizes).bind(&p.region).bind(&p.currency)
    .fetch_one(&pool)
    .await
    .map_err(err)?;

    // Record the price → calculates history + drop %
    let update = record_price(&pool, row.0, p.price)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AddResult { id: row.0, drop_percent: update.drop_percent }))
}

#[derive(Deserialize)]
pub struct PriceChange { pub item_id: Uuid, pub new_price: f64 }

/// Update an existing item's price — simulates a price drop for testing charts/alerts.
pub async fn update_price(
    State(pool): State<PgPool>,
    Json(c): Json<PriceChange>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let update = record_price(&pool, c.item_id, c.new_price)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "dropped": update.dropped,
        "drop_percent": update.drop_percent
    })))
}

#[derive(Deserialize)]
pub struct BatchProducts { pub products: Vec<NewProduct> }

#[derive(Serialize)]
pub struct BatchResult { pub added: usize, pub failed: Vec<String> }

pub async fn add_batch(
    State(pool): State<PgPool>,
    Json(batch): Json<BatchProducts>,
) -> Json<BatchResult> {
    let mut added = 0;
    let mut failed = Vec::new();

    for p in batch.products {
        // resolve brand
        let brand: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM brands WHERE slug = $1 AND region = $2",
        )
        .bind(&p.brand_slug).bind(&p.region)
        .fetch_optional(&pool).await.ok().flatten();

        let Some(brand) = brand else {
            failed.push(format!("{}: brand '{}' not in {}", p.sku, p.brand_slug, p.region));
            continue;
        };

        let row: Result<(Uuid,), _> = sqlx::query_as(
            r#"
            INSERT INTO items
              (sku, brand_id, name, category, current_price, was_price,
               drop_percent, affiliate_url, image_url, sizes, region, currency)
            VALUES ($1,$2,$3,$4::category,$5,$6,0,$7,'',$8,$9,$10)
            ON CONFLICT (sku, brand_id, region)
            DO UPDATE SET name = EXCLUDED.name, affiliate_url = EXCLUDED.affiliate_url
            RETURNING id
            "#,
        )
        .bind(&p.sku).bind(brand.0).bind(&p.name).bind(&p.category)
        .bind(p.price).bind(p.was_price)
        .bind(&p.affiliate_url).bind(&p.sizes).bind(&p.region).bind(&p.currency)
        .fetch_one(&pool).await;

        match row {
            Ok((id,)) => {
                let _ = record_price(&pool, id, p.price).await;
                added += 1;
            }
            Err(e) => failed.push(format!("{}: {}", p.sku, e)),
        }
    }

    Json(BatchResult { added, failed })
}

#[derive(Debug, Deserialize)]
pub struct IngestQuery {
    pub url:    String,
    pub region: Option<String>,
    pub brand:  Option<String>,
}

pub async fn run_ingest(
    State(pool): State<PgPool>,
    Query(q): Query<IngestQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let region = q.region.unwrap_or_else(|| "CA".into());
    let brand  = q.brand.unwrap_or_else(|| "Unknown".into());
    let rep = crate::ingest::ingest_feed(&pool, &q.url, &region, &brand)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({
        "rows_seen": rep.rows_seen, "parsed": rep.parsed,
        "upserted": rep.upserted, "drops_found": rep.drops_found,
        "skips": rep.skips, "errors": rep.errors
    })))
}