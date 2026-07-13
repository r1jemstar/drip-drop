use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id:            String,
    pub name:          String,
    pub brand:         String,
    pub category:      String,
    pub current_price: f64,
    pub was_price:     f64,
    pub drop_percent:  f64,
    pub currency:      String,
    pub affiliate_url: String,
    pub sizes:         Vec<String>,
    pub in_stock:      bool,
    pub region:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price:       f64,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GeoResponse { country: String }

const API_BASE: &str = "https://drip-drop-api.onrender.com";

fn symbol(currency: &str) -> &str {
    match currency { "GBP"=>"£","USD"=>"$","CAD"=>"C$","AUD"=>"A$",_=>"£" }
}
fn country_to_region(c: &str) -> String {
    match c { "GB"=>"GB","US"=>"US","CA"=>"CA",_=>"CA" }.to_string()
}

async fn fetch_deals(region: String) -> Vec<Deal> {
    let url = format!("{API_BASE}/api/deals?region={region}");
    match gloo_net::http::Request::get(&url).send().await {
        Ok(r) => r.json::<Vec<Deal>>().await.unwrap_or_default(),
        Err(_) => vec![],
    }
}
async fn fetch_history(id: String) -> Vec<PricePoint> {
    let url = format!("{API_BASE}/api/deals/{id}/history");
    match gloo_net::http::Request::get(&url).send().await {
        Ok(r) => r.json::<Vec<PricePoint>>().await.unwrap_or_default(),
        Err(_) => vec![],
    }
}
async fn detect_region() -> String {
    match gloo_net::http::Request::get("/geo").send().await {
        Ok(r) => match r.json::<GeoResponse>().await {
            Ok(g) => country_to_region(&g.country), Err(_) => "CA".into(),
        },
        Err(_) => "CA".into(),
    }
}

// Build the price history chart SVG — prototype teal aesthetic, straight lines
fn build_chart_svg(points: &[PricePoint], currency: &str) -> String {
    let sym = symbol(currency);

    if points.is_empty() {
        return String::from("<div class='chart-empty'>No price data yet.</div>");
    }

    let w = 520.0; let h = 200.0;
    let pad_l = 44.0; let pad_r = 20.0; let pad_t = 20.0; let pad_b = 36.0;

    // Single point: flat line across, no callout
    if points.len() == 1 {
        let price = points[0].price;
        let y = pad_t + (h - pad_t - pad_b) / 2.0;
        return format!(
            "<svg viewBox='0 0 {w} {h}' xmlns='http://www.w3.org/2000/svg' style='overflow:visible'>\
              <line x1='{pad_l}' y1='{y:.1}' x2='{:.1}' y2='{y:.1}' stroke='#1A3326' stroke-width='1' stroke-dasharray='4 4'/>\
              <text x='{:.1}' y='{:.1}' fill='#2D6650' font-size='9' text-anchor='end'>{sym}{price:.0}</text>\
              <line x1='{pad_l}' y1='{y:.1}' x2='{:.1}' y2='{y:.1}' stroke='#2DD4A0' stroke-width='2.5' stroke-linecap='round'/>\
              <circle cx='{pad_l}' cy='{y:.1}' r='3.5' fill='#0A1410' stroke='#2DD4A0' stroke-width='2'/>\
              <circle cx='{:.1}' cy='{y:.1}' r='3.5' fill='#0A1410' stroke='#2DD4A0' stroke-width='2'/>\
              <text x='{:.1}' y='{:.1}' fill='#2D6650' font-size='10' text-anchor='middle'>price steady - tracking from here</text>\
            </svg>",
            w - pad_r, pad_l - 6.0, y + 4.0, w - pad_r, w - pad_r, w/2.0, y - 12.0
        );
    }

    let prices: Vec<f64> = points.iter().map(|p| p.price).collect();
    let min_p = prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_p = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max_p - min_p).abs() < 0.01 { 1.0 } else { max_p - min_p };
    let n = points.len();

    // Coordinates
    let coords: Vec<(f64,f64)> = points.iter().enumerate().map(|(i, pt)| {
        let x = pad_l + (i as f64 / (n as f64 - 1.0)) * (w - pad_l - pad_r);
        let y = pad_t + (1.0 - (pt.price - min_p) / range) * (h - pad_t - pad_b);
        (x, y)
    }).collect();

    // Straight-line path
    let mut line = String::new();
    for (i,(x,y)) in coords.iter().enumerate() {
        line.push_str(&format!("{}{:.1} {:.1}", if i==0 {"M "} else {" L "}, x, y));
    }
    let mut area = line.clone();
    area.push_str(&format!(" L {:.1} {:.1} L {:.1} {:.1} Z",
        coords[n-1].0, h - pad_b, coords[0].0, h - pad_b));

    let low_idx = prices.iter().enumerate()
        .min_by(|a,b| a.1.partial_cmp(b.1).unwrap()).map(|(i,_)| i).unwrap_or(0);

    // Grid lines
    let mut grid = String::new();
    for k in 0..=4 {
        let f = k as f64 / 4.0;
        let val = min_p + f * range;
        let y = pad_t + (1.0 - f) * (h - pad_t - pad_b);
        grid.push_str(&format!(
            "<line x1='{pad_l}' y1='{y:.1}' x2='{:.1}' y2='{y:.1}' stroke='#1A3326' stroke-width='1' stroke-dasharray='4 4'/>\
             <text x='{:.1}' y='{:.1}' fill='#2D6650' font-size='9' text-anchor='end'>{sym}{val:.0}</text>",
            w - pad_r, pad_l - 6.0, y + 4.0
        ));
    }

    // Outlined dots at every point
    let mut dots = String::new();
    for (i,(x,y)) in coords.iter().enumerate() {
        if i == low_idx { continue; }
        dots.push_str(&format!(
            "<circle cx='{x:.1}' cy='{y:.1}' r='3.5' fill='#0A1410' stroke='#2DD4A0' stroke-width='2'/>"
        ));
    }

    // Lowest dot — glowing, NO number callout
    let (lx, ly) = coords[low_idx];
    let low_dot = format!(
        "<circle cx='{lx:.1}' cy='{ly:.1}' r='6' fill='#2DD4A0' filter='url(#dotglow)'/>\
         <circle cx='{lx:.1}' cy='{ly:.1}' r='3.5' fill='#0A1410'/>"
    );

    format!(
        "<svg viewBox='0 0 {w} {h}' xmlns='http://www.w3.org/2000/svg' style='overflow:visible'>\
          <defs>\
            <linearGradient id='areaGrad' x1='0' y1='0' x2='0' y2='1'>\
              <stop offset='0%' stop-color='#2DD4A0' stop-opacity='0.18'/>\
              <stop offset='100%' stop-color='#2DD4A0' stop-opacity='0'/>\
            </linearGradient>\
            <filter id='glow'><feGaussianBlur stdDeviation='2.5' result='b'/><feMerge><feMergeNode in='b'/><feMergeNode in='SourceGraphic'/></feMerge></filter>\
            <filter id='dotglow'><feGaussianBlur stdDeviation='3' result='b'/><feMerge><feMergeNode in='b'/><feMergeNode in='SourceGraphic'/></feMerge></filter>\
          </defs>\
          {grid}\
          <path d='{area}' fill='url(#areaGrad)'/>\
          <path d='{line}' fill='none' stroke='#2DD4A0' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round' filter='url(#glow)'/>\
          {dots}{low_dot}\
        </svg>"
    )
}

#[component]
fn PriceModal(deal: Deal, on_close: WriteSignal<Option<Deal>>) -> impl IntoView {
    let id = deal.id.clone();
    let currency = deal.currency.clone();
    let sym = symbol(&deal.currency).to_string();

    // Signals holding the rendered chart SVG and stats HTML (Strings = easy).
    let (chart_html, set_chart_html) = create_signal(String::from("<div class='chart-empty'>Loading price history…</div>"));
    let (stats_html, set_stats_html) = create_signal(String::new());

    // Fetch history once on mount, compute strings, store in signals.
    spawn_local(async move {
        let pts = fetch_history(id).await;
        let svg = build_chart_svg(&pts, &currency);
        set_chart_html.set(svg);
        if pts.len() >= 2 {
            let prices: Vec<f64> = pts.iter().map(|p| p.price).collect();
            let low = prices.iter().cloned().fold(f64::INFINITY, f64::min);
            let high = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let biggest = high - low;
            let stats = format!(
                "<div class='chart-stats'>                 <div class='cstat'><div class='cstat-val green'>{sym}{low:.0}</div><div class='cstat-label'>All-time low</div></div>                 <div class='cstat'><div class='cstat-val'>{sym}{high:.0}</div><div class='cstat-label'>All-time high</div></div>                 <div class='cstat'><div class='cstat-val green'>{sym}{biggest:.0}</div><div class='cstat-label'>Biggest drop</div></div>                 <div class='cstat'><div class='cstat-val'>{sym}{avg:.0}</div><div class='cstat-label'>Average</div></div>                 </div>"
            );
            set_stats_html.set(stats);
        }
    });

    let brand = deal.brand.clone();
    let name = deal.name.clone();
    let sym2 = symbol(&deal.currency).to_string();
    let now_str = format!("{}{:.0}", sym2, deal.current_price);
    let was_str = format!("{}{:.0}", sym2, deal.was_price);
    let show_was = deal.was_price > deal.current_price;
    let drop_pct = deal.drop_percent as i64;
    let shop_url = deal.affiliate_url.clone();

    view! {
        <div class="modal-overlay" on:click=move |_| on_close.set(None)>
            <div class="price-modal" on:click=move |e| e.stop_propagation()>
                <button class="modal-close" on:click=move |_| on_close.set(None)>"✕"</button>
                <div class="modal-brand">{brand}</div>
                <div class="modal-name">{name}</div>
                <div class="modal-prices">
                    <span class="modal-now">{now_str}</span>
                    {show_was.then(|| view!{
                        <>
                          <span class="modal-was">{was_str}</span>
                          <span class="modal-drop">"↓ " {drop_pct} "% off"</span>
                        </>
                    })}
                </div>
                <div class="chart-wrap">
                    <div inner_html=move || chart_html.get()></div>
                </div>
                <div inner_html=move || stats_html.get()></div>
                <a href={shop_url} target="_blank" class="modal-shop-btn">"Shop this deal →"</a>
            </div>
        </div>
    }
}


#[component]
fn DealCard(deal: Deal, on_open: WriteSignal<Option<Deal>>) -> impl IntoView {
    let sym = symbol(&deal.currency).to_string();
    let save = deal.was_price - deal.current_price;
    let sizes = deal.sizes.clone();
    let deal_click = deal.clone();
    let deal_shop = deal.clone();
    let recent = use_context::<RwSignal<Vec<Deal>>>();

    let (faved, set_faved) = create_signal(false);
    let (alerted, set_alerted) = create_signal(false);
    let (shared, set_shared) = create_signal(false);

    let share_url = deal.affiliate_url.clone();

    let (quality, qclass) = if deal.drop_percent >= 40.0 { ("Amazing deal","q-hot") }
        else if deal.drop_percent >= 25.0 { ("Great deal","q-good") }
        else if deal.drop_percent > 0.0 { ("Small drop","q-small") }
        else { ("","") };

    let do_share = move |_ev: leptos::ev::MouseEvent| {
        let url = share_url.clone();
        if let Some(win) = web_sys::window() {
            let clip = win.navigator().clipboard();
            let _ = clip.write_text(&url);
        }
        set_shared.set(true);
    };

    view! {
        <div class="deal-card" on:click=move |_| {
            let d = deal_click.clone();
            if let Some(r) = recent {
                r.update(|list| {
                    list.retain(|x| x.id != d.id);
                    list.insert(0, d.clone());
                    list.truncate(6);
                });
            }
            on_open.set(Some(d));
        }>
            <div class="card-img">
                <div class="card-emoji">"\u{1F6CD}"</div>
                {(deal.drop_percent > 0.0).then(|| view!{
                    <div class="card-badge">"-" {deal.drop_percent as i64} "%"</div>
                })}

                <div class="card-actions">
                    <button class="card-act" class:on=move || faved.get()
                        title="Save to favourites"
                        on:click=move |e| { e.stop_propagation(); set_faved.update(|f| *f = !*f); }>
                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M12 20s-7-4.35-7-9.5A3.5 3.5 0 0 1 12 7a3.5 3.5 0 0 1 7 3.5c0 5.15-7 9.5-7 9.5z"
                                stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" class="heart-path"/>
                        </svg>
                    </button>
                    <button class="card-act" title="Share this deal"
                        on:click=move |e| { e.stop_propagation(); do_share(e); }>
                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <circle cx="18" cy="5" r="2.4" stroke="currentColor" stroke-width="1.8"/>
                            <circle cx="6" cy="12" r="2.4" stroke="currentColor" stroke-width="1.8"/>
                            <circle cx="18" cy="19" r="2.4" stroke="currentColor" stroke-width="1.8"/>
                            <line x1="8.1" y1="10.8" x2="15.9" y2="6.2" stroke="currentColor" stroke-width="1.8"/>
                            <line x1="8.1" y1="13.2" x2="15.9" y2="17.8" stroke="currentColor" stroke-width="1.8"/>
                        </svg>
                    </button>
                    <button class="card-act" class:on=move || alerted.get()
                        title="Set price alert"
                        on:click=move |e| { e.stop_propagation(); set_alerted.update(|a| *a = !*a); }>
                        <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M18 8a6 6 0 1 0-12 0c0 7-3 9-3 9h18s-3-2-3-9z"
                                stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/>
                            <path d="M13.7 21a2 2 0 0 1-3.4 0" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
                        </svg>
                    </button>
                </div>

                <div class="chart-glyph" title="Tap for price history">
                    <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <polyline points="3,17 9,11 13,14 21,6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                        <circle cx="9" cy="11" r="1.6" fill="currentColor"/>
                        <circle cx="21" cy="6" r="1.6" fill="currentColor"/>
                        <polyline points="3,21 21,21" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.4"/>
                    </svg>
                </div>

                {move || shared.get().then(|| view!{ <div class="share-toast">"Link copied \u{2713}"</div> })}
            </div>
            <div class="card-body">
                <div class="card-brand">{deal.brand.clone()}</div>
                <div class="card-name">{deal.name.clone()}</div>
                {(!quality.is_empty()).then(|| view!{ <div class=format!("quality {qclass}")>{quality}</div> })}
                <div class="size-row">
                    {sizes.into_iter().map(|s| view!{ <div class="sz">{s}</div> }).collect_view()}
                </div>
                <div class="card-price-row">
                    <span class="price-now">{sym.clone()}{format!("{:.0}", deal.current_price)}</span>
                    {(save > 0.0).then(|| view!{
                        <>
                          <span class="price-was">{sym.clone()}{format!("{:.0}", deal.was_price)}</span>
                          <span class="price-save">"Save " {sym.clone()}{format!("{:.0}", save)}</span>
                        </>
                    })}
                </div>
                <a href={deal_shop.affiliate_url.clone()} target="_blank" class="card-go-btn"
                   on:click=move |e| e.stop_propagation()>"Shop this deal \u{2192}"</a>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (region, set_region) = create_signal("CA".to_string());
    let (panel_open, set_panel_open) = create_signal(false);
    let (detecting, set_detecting) = create_signal(true);
    let (modal_deal, set_modal_deal) = create_signal::<Option<Deal>>(None);
    let recent = create_rw_signal::<Vec<Deal>>(Vec::new());
    provide_context(recent);
    let recent_ctx = recent;
    let (active_tab, set_active_tab) = create_signal("deals".to_string());
    let (active_filter, set_active_filter) = create_signal("all".to_string());

    spawn_local(async move {
        let d = detect_region().await;
        set_region.set(d);
        set_detecting.set(false);
    });

    let deals = create_resource(move || region.get(), |r| async move { fetch_deals(r).await });
    let region_name = move || match region.get().as_str() {
        "GB"=>"United Kingdom","US"=>"United States","CA"=>"Canada",_=>"Canada" };
    let cur_sym = move || match region.get().as_str() {
        "GB"=>"£","US"=>"$","CA"=>"C$",_=>"C$" };

    // ── computed stats from real deals ──
    let stat_count = move || deals.get().map(|d| d.len()).unwrap_or(0);
    let stat_avg = move || {
        deals.get().map(|d| {
            let drops: Vec<f64> = d.iter().filter(|x| x.drop_percent > 0.0).map(|x| x.drop_percent).collect();
            if drops.is_empty() { 0.0 } else { drops.iter().sum::<f64>() / drops.len() as f64 }
        }).unwrap_or(0.0)
    };
    let stat_low = move || {
        deals.get().and_then(|d| d.iter().map(|x| x.current_price).fold(None, |acc: Option<f64>, p| {
            Some(acc.map_or(p, |a| a.min(p)))
        })).unwrap_or(0.0)
    };

    // ── ticker items from deals ──
    let ticker_items = move || {
        deals.get().map(|d| {
            d.iter().take(8).map(|deal| {
                let sym = symbol(&deal.currency);
                format!("<span class='ti'><span class='ti-brand'>{}</span> {} <span class='ti-drop'>↓{}%</span> · {}{:.0}</span>",
                    deal.brand,
                    deal.name.split('—').next().unwrap_or(&deal.name).trim(),
                    deal.drop_percent as i64, sym, deal.current_price)
            }).collect::<Vec<_>>().join("")
        }).unwrap_or_default()
    };

    let is_deals = move || active_tab.get() == "deals";

    view! {
        // ── ROTATING TICKER ──
        <div class="ticker-wrap">
            <div class="ticker" inner_html=move || {
                let items = ticker_items();
                format!("{items}{items}") // duplicate for seamless loop
            }></div>
        </div>

        // ── NAV ──
        <nav>
            <button class="panel-toggle" on:click=move |_| set_panel_open.update(|o| *o=!*o)>"☰"</button>
            <div class="logo">
                <span class="logo-drip">"Drip"</span><div class="logo-dot"></div><span class="logo-drop">"Drop"</span>
            </div>
            <div class="nav-region" on:click=move |_| set_panel_open.set(true)>{move || region.get()}" ▾"</div>
        </nav>

        // ── TAB BAR ──
        <div class="tab-bar">
            <div class="tab" class:active=move || active_tab.get()=="deals"
                on:click=move |_| set_active_tab.set("deals".into())>"🔥 Deals"</div>
            <div class="tab" class:active=move || active_tab.get()=="trending"
                on:click=move |_| set_active_tab.set("trending".into())>"📈 Trending"</div>
            <div class="tab" class:active=move || active_tab.get()=="outfit"
                on:click=move |_| set_active_tab.set("outfit".into())>"✨ Outfit Builder"</div>
            <div class="tab" class:active=move || active_tab.get()=="boards"
                on:click=move |_| set_active_tab.set("boards".into())>"📌 Style Boards"</div>
            <div class="tab" class:active=move || active_tab.get()=="alerts"
                on:click=move |_| set_active_tab.set("alerts".into())>"🔔 Alerts"</div>
            <div class="tab" class:active=move || active_tab.get()=="brands"
                on:click=move |_| set_active_tab.set("brands".into())>"🏷 Brands"</div>
        </div>

        // ── SIDE PANEL (region + AI search + filters) ──
        <div class=move || if panel_open.get() {"side-panel open"} else {"side-panel"}>
            <div class="panel-header"><span>"Menu"</span>
                <button class="panel-close" on:click=move |_| set_panel_open.set(false)>"✕"</button></div>

            <div class="panel-section">
                <div class="panel-label">"✦ AI Search"</div>
                <div class="panel-hint">"Describe what you want in plain words."</div>
                <input class="panel-search" placeholder="e.g. beige blazer under C$50" />
                <div class="ai-chips">
                    <span class="ai-chip">"going out dress"</span>
                    <span class="ai-chip">"chunky trainers"</span>
                    <span class="ai-chip">"linen set"</span>
                </div>
                <div class="panel-soon">"Coming soon"</div>
            </div>

            <div class="panel-section">
                <div class="panel-label">"Region"</div>
                <div class="region-options">
                    <button class="region-opt" class:active=move || region.get()=="CA"
                        on:click=move |_| set_region.set("CA".into())>"🇨🇦 Canada"</button>
                    <button class="region-opt" class:active=move || region.get()=="US"
                        on:click=move |_| set_region.set("US".into())>"🇺🇸 United States"</button>
                    <button class="region-opt" class:active=move || region.get()=="GB"
                        on:click=move |_| set_region.set("GB".into())>"🇬🇧 United Kingdom"</button>
                </div>
            </div>

            <div class="panel-section">
                <div class="panel-label">"Followed brands"</div>
                <div class="panel-hint">"Follow brands to get deal alerts."</div>
                <div class="panel-soon">"Coming soon"</div>
            </div>
        </div>
        <div class=move || if panel_open.get() {"panel-overlay show"} else {"panel-overlay"}
             on:click=move |_| set_panel_open.set(false)></div>

        // ── MAIN ──
        <div class="page">
            {move || if is_deals() {
                view! {
                    // HERO
                    <div class="hero">
                        <div class="hero-eyebrow">"— Price drops. Updated daily."</div>
                        <h1 class="hero-title">"Quality drips with the best "<em>"drops."</em></h1>
                        <p class="hero-tagline">"Track prices across Canadian brands and more. Get alerts the moment your wishlist drops — before everyone else finds out."</p>
                        <div class="hero-actions">
                            <button class="btn-primary" on:click=move |_| set_panel_open.set(true)>"Follow your brands"</button>
                            <button class="btn-ghost" on:click=move |_| set_active_tab.set("boards".into())>"My Style Boards"</button>
                        </div>
                        <div class="stats-row">
                            <div class="stat-card">
                                <div class="stat-num">{move || stat_count().to_string()}</div>
                                <div class="stat-label">"Live deals right now"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num">{move || format!("{:.0}%", stat_avg())}</div>
                                <div class="stat-label">"Avg. discount"</div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-num">{move || format!("{}{:.0}", cur_sym(), stat_low())}</div>
                                <div class="stat-label">"Lowest price today"</div>
                            </div>
                        </div>
                    </div>

                    // FILTER CHIPS
                    <div class="filter-row">
                        <div class="fchip" class:active=move || active_filter.get()=="all"
                            on:click=move |_| set_active_filter.set("all".into())>"All"</div>
                        <div class="fchip" class:active=move || active_filter.get()=="womenswear"
                            on:click=move |_| set_active_filter.set("womenswear".into())>"👗 Womenswear"</div>
                        <div class="fchip" class:active=move || active_filter.get()=="footwear"
                            on:click=move |_| set_active_filter.set("footwear".into())>"👟 Footwear"</div>
                        <div class="fchip" class:active=move || active_filter.get()=="outerwear"
                            on:click=move |_| set_active_filter.set("outerwear".into())>"🧥 Outerwear"</div>
                        <div class="fchip" class:active=move || active_filter.get()=="accessories"
                            on:click=move |_| set_active_filter.set("accessories".into())>"👜 Accessories"</div>
                    </div>

                    <div class="sec-hdr"><div class="sec-title">"Today's drops — "{region_name}</div></div>

                    <div class="deal-grid">
                        <Suspense fallback=move || view!{ <div class="loading">"Loading deals…"</div> }>
                            {move || {
                                if detecting.get() { return view!{ <div class="loading">"Finding your region…"</div> }.into_view(); }
                                let filt = active_filter.get();
                                deals.get().map(|list| {
                                    let filtered: Vec<Deal> = list.into_iter()
                                        .filter(|d| filt=="all" || d.category==filt).collect();
                                    if filtered.is_empty() {
                                        view!{ <div class="loading">"No deals match — try another filter or region."</div> }.into_view()
                                    } else {
                                        filtered.into_iter().map(|d| view!{ <DealCard deal=d on_open=set_modal_deal/> }).collect_view()
                                    }
                                }).unwrap_or_else(|| view!{ <div class="loading">"Loading…"</div> }.into_view())
                            }}
                        </Suspense>
                    </div>
                }.into_view()
            } else {
                // COMING SOON for other tabs
                let tab = active_tab.get();
                let (icon, title, desc) = match tab.as_str() {
                    "trending" => ("📈", "Trending", "The hottest drops right now, ranked by how fast they're moving."),
                    "outfit"   => ("✨", "Outfit Builder", "Combine tracked deals into a full look and see the total price."),
                    "boards"   => ("📌", "Style Boards", "Save deals into shareable boards. Friends see live prices."),
                    "alerts"   => ("🔔", "Price Alerts", "Get notified the moment a tracked item drops in price."),
                    "brands"   => ("🏷", "Brands", "Follow your favourite brands and exclude the ones you don't want."),
                    _ => ("✨","Coming soon","")
                };
                view! {
                    <div class="coming-soon">
                        <div class="cs-icon">{icon}</div>
                        <div class="cs-title">{title}</div>
                        <div class="cs-desc">{desc}</div>
                        <div class="cs-badge">"Coming soon"</div>
                        <button class="btn-ghost" on:click=move |_| set_active_tab.set("deals".into())>"← Back to deals"</button>
                    </div>
                }.into_view()
            }}
        </div>

        {move || modal_deal.get().map(|d| view!{ <PriceModal deal=d on_close=set_modal_deal/> })}
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}