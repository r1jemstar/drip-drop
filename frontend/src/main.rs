use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
mod legal;
use legal::{Privacy, Terms, Disclosure, Footer,CookieNotice};
use leptos_router::*;

type DealsRes = Resource<(String, String, bool), Vec<Deal>> ;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub category: String,
    pub current_price: f64,
    pub was_price: f64,
    pub drop_percent: f64,
    pub currency: String,
    pub affiliate_url: String,
    pub image_url: Option<String>,
    pub sizes: Vec<String>,
    pub in_stock: bool,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price: f64,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GeoResponse {
    country: String,
}

const API_BASE: &str = "https://drip-drop-api.onrender.com";

fn symbol(currency: &str) -> &str {
    match currency {
        "GBP" => "£",
        "USD" => "$",
        "CAD" => "C$",
        "AUD" => "A$",
        _ => "£",
    }
}

fn cat_to_slot(c: &str) -> &'static str {
    match c {
        "footwear" => "shoes",
        "outerwear" => "top",
        "accessories" => "bag",
        _ => "bottom",
    }
}

fn country_to_region(c: &str) -> String {
    match c {
        "GB" => "GB",
        "US" => "US",
        "CA" => "CA",
        _ => "CA",
    }
    .to_string()
}

async fn fetch_deals(region: String, sort: String, sale_only: bool) -> Vec<Deal> {
    let min_drop = if sale_only { 1 } else { 0 };
    let url = format!("{API_BASE}/api/deals?region={region}&sort={sort}&min_drop={min_drop}&limit=100");
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
            Ok(g) => country_to_region(&g.country),
            Err(_) => "CA".into(),
        },
        Err(_) => "CA".into(),
    }
}

#[derive(Clone)]
struct Pt {
    x: f64,
    y: f64,
    price: f64,
    label: String,
}

#[derive(Clone)]
struct ChartGeo {
    pts: Vec<Pt>,
    backdrop: String,
    low_idx: usize,
    sym: String,
}

fn compute_geo(points: &[PricePoint], currency: &str) -> Option<ChartGeo> {
    if points.len() < 2 {
        return None;
    }
    let sym = symbol(currency).to_string();
    let w = 520.0;
    let h = 200.0;
    let pad_l = 44.0;
    let pad_r = 20.0;
    let pad_t = 20.0;
    let pad_b = 36.0;
    let prices: Vec<f64> = points.iter().map(|p| p.price).collect();
    let min_p = prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_p = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max_p - min_p).abs() < 0.01 {
        1.0
    } else {
        max_p - min_p
    };
    let n = points.len();
    let pts: Vec<Pt> = points
        .iter()
        .enumerate()
        .map(|(i, pt)| {
            let x = pad_l + (i as f64 / (n as f64 - 1.0)) * (w - pad_l - pad_r);
            let y = pad_t + (1.0 - (pt.price - min_p) / range) * (h - pad_t - pad_b);
            let label = if pt.recorded_at.len() >= 10 {
                pt.recorded_at[5..10].to_string()
            } else {
                pt.recorded_at.clone()
            };
            Pt {
                x,
                y,
                price: pt.price,
                label,
            }
        })
        .collect();
    let mut line = String::new();
    for (i, p) in pts.iter().enumerate() {
        line.push_str(&format!(
            "{}{:.1} {:.1}",
            if i == 0 { "M " } else { " L " },
            p.x,
            p.y
        ));
    }
    let mut area = line.clone();
    area.push_str(&format!(
        " L {:.1} {:.1} L {:.1} {:.1} Z",
        pts[n - 1].x,
        h - pad_b,
        pts[0].x,
        h - pad_b
    ));
    let low_idx = prices
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut grid = String::new();
    for k in 0..=4 {
        let f = k as f64 / 4.0;
        let val = min_p + f * range;
        let y = pad_t + (1.0 - f) * (h - pad_t - pad_b);
        grid.push_str(&format!("<line x1='{pad_l}' y1='{y:.1}' x2='{:.1}' y2='{y:.1}' stroke='#1A3326' stroke-width='1' stroke-dasharray='4 4'/><text x='{:.1}' y='{:.1}' fill='#2D6650' font-size='9' text-anchor='end'>{sym}{val:.0}</text>", w-pad_r, pad_l-6.0, y+4.0));
    }
    let mut xlabels = String::new();
    for p in &pts {
        xlabels.push_str(&format!(
            "<text x='{:.1}' y='{:.1}' fill='#2D6650' font-size='9' text-anchor='middle'>{}</text>",
            p.x,
            h - 6.0,
            p.label
        ));
    }
    let backdrop = format!(
        "<defs>\
          <linearGradient id='areaGrad' x1='0' y1='0' x2='0' y2='1'>\
            <stop offset='0%' stop-color='#2DD4A0' stop-opacity='0.18'/>\
            <stop offset='100%' stop-color='#2DD4A0' stop-opacity='0'/>\
          </linearGradient>\
          <filter id='glow'><feGaussianBlur stdDeviation='2.5' result='b'/><feMerge><feMergeNode in='b'/><feMergeNode in='SourceGraphic'/></feMerge></filter>\
          <filter id='dotglow'><feGaussianBlur stdDeviation='3' result='b'/><feMerge><feMergeNode in='b'/><feMergeNode in='SourceGraphic'/></feMerge></filter>\
        </defs>{grid}{xlabels}\
        <path d='{area}' fill='url(#areaGrad)'/>\
        <path d='{line}' fill='none' stroke='#2DD4A0' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round' filter='url(#glow)'/>"
    );
    Some(ChartGeo {
        pts,
        backdrop,
        low_idx,
        sym,
    })
}

#[component]
fn PriceModal(deal: Deal, on_close: WriteSignal<Option<Deal>>) -> impl IntoView {
    let id = deal.id.clone();
    let currency = deal.currency.clone();
    let drop_pct = deal.drop_percent as i64;

    let (loaded, set_loaded) = create_signal(false);
    let (chart, set_chart) = create_signal::<Option<ChartGeo>>(None);
    let (stats_html, set_stats_html) = create_signal(String::new());

    spawn_local(async move {
        let pts = fetch_history(id).await;
        set_chart.set(compute_geo(&pts, &currency));
        if pts.len() >= 2 {
            let sym = symbol(&currency).to_string();
            let prices: Vec<f64> = pts.iter().map(|p| p.price).collect();
            let low = prices.iter().cloned().fold(f64::INFINITY, f64::min);
            let high = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            let biggest = high - low;
            let stats = format!(
                "<div class='chart-stats'>\
                 <div class='cstat'><div class='cstat-val green'>{sym}{low:.0}</div><div class='cstat-label'>All-time low</div></div>\
                 <div class='cstat'><div class='cstat-val'>{sym}{high:.0}</div><div class='cstat-label'>All-time high</div></div>\
                 <div class='cstat'><div class='cstat-val green'>{sym}{biggest:.0}</div><div class='cstat-label'>Biggest drop</div></div>\
                 <div class='cstat'><div class='cstat-val red'>-{drop_pct}%</div><div class='cstat-label'>Off now</div></div>\
                 <div class='cstat'><div class='cstat-val'>{sym}{avg:.0}</div><div class='cstat-label'>Average</div></div>\
                 </div>"
            );
            set_stats_html.set(stats);
        }
        set_loaded.set(true);
    });

    let brand = deal.brand.clone();
    let name = deal.name.clone();
    let sym2 = symbol(&deal.currency).to_string();
    let now_str = format!("{}{:.0}", sym2, deal.current_price);
    let was_str = format!("{}{:.0}", sym2, deal.was_price);
    let show_was = deal.was_price > deal.current_price;
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
                    {move || match (loaded.get(), chart.get()) {
                        (false, _) => view!{ <div class="chart-empty">"Loading price history…"</div> }.into_view(),
                        (true, None) => view!{ <div class="chart-empty">"Not enough price data yet."</div> }.into_view(),
                        (true, Some(g)) => {
                            let hover = create_rw_signal::<Option<usize>>(None);
                            let low = g.low_idx;
                            let sym = g.sym.clone();
                            let pts_sv = store_value(g.pts.clone());
                            view!{
                                <svg viewBox="0 0 520 200" xmlns="http://www.w3.org/2000/svg" style="overflow:visible"
                                     on:mouseleave=move |_| hover.set(None)>
                                    <g inner_html=g.backdrop.clone()></g>
                                    {g.pts.iter().enumerate().map(|(i,p)|{
                                        let (cx,cy)=(format!("{:.1}",p.x),format!("{:.1}",p.y));
                                        if i==low {
                                            view!{ <>
                                                <circle cx=cx.clone() cy=cy.clone() r="6" fill="#2DD4A0" filter="url(#dotglow)"/>
                                                <circle cx=cx cy=cy r="3.5" fill="#0A1410"/>
                                            </> }.into_view()
                                        } else {
                                            view!{ <circle cx=cx cy=cy r="3.5" fill="#0A1410" stroke="#2DD4A0" stroke-width="2"/> }.into_view()
                                        }
                                    }).collect_view()}
                                    {move || hover.get().map(|i|{
                                        let x = pts_sv.with_value(|v| format!("{:.1}", v[i].x));
                                        view!{ <line x1=x.clone() y1="20" x2=x y2="164" stroke="#2DD4A0" stroke-width="1" stroke-dasharray="3 3" opacity="0.5"/> }
                                    })}
                                    {g.pts.iter().enumerate().map(|(i,p)|{
                                        view!{ <circle cx=format!("{:.1}",p.x) cy=format!("{:.1}",p.y) r="14" fill="transparent" style="cursor:crosshair"
                                            on:mouseenter=move |_| hover.set(Some(i))/> }
                                    }).collect_view()}
                                    {move || hover.get().map(|i|{
                                        let p = pts_sv.with_value(|v| v[i].clone());
                                        let tx = (p.x-30.0).max(2.0).min(458.0);
                                        let ty = (p.y-40.0).max(2.0);
                                        view!{ <g>
                                            <rect x=format!("{:.1}",tx) y=format!("{:.1}",ty) width="60" height="32" rx="6" fill="#0F2A1E" stroke="#2DD4A0" stroke-width="1"/>
                                            <text x=format!("{:.1}",tx+8.0) y=format!("{:.1}",ty+14.0) fill="#2DD4A0" font-size="11" font-weight="700">{sym.clone()}{format!("{:.0}",p.price)}</text>
                                            <text x=format!("{:.1}",tx+8.0) y=format!("{:.1}",ty+26.0) fill="#4A7A5A" font-size="9">{p.label.clone()}</text>
                                        </g> }
                                    })}
                                </svg>
                            }.into_view()
                        }
                    }}
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

    let (quality, qclass) = if deal.drop_percent >= 40.0 {
        ("Amazing deal", "q-hot")
    } else if deal.drop_percent >= 25.0 {
        ("Great deal", "q-good")
    } else if deal.drop_percent > 0.0 {
        ("Small drop", "q-small")
    } else {
        ("", "")
    };

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
                {match deal.image_url.clone() {
                    Some(url) => view!{ <img class="card-photo" src=url alt=deal.name.clone() loading="lazy"/> }.into_view(),
                    None => view!{ <div class="card-emoji">"\u{1F6CD}"</div> }.into_view(),
                }}
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
fn RecentStrip(on_open: WriteSignal<Option<Deal>>) -> impl IntoView {
    let recent = use_context::<RwSignal<Vec<Deal>>>();
    view! {
        {move || {
            let list = recent.map(|r| r.get()).unwrap_or_default();
            if list.is_empty() { return view!{}.into_view(); }
            view! {
                <div class="recent-strip">
                    <div class="recent-hdr">"Recently viewed"</div>
                    <div class="recent-scroll">
                        {list.into_iter().map(|d| {
                            let sym = symbol(&d.currency).to_string();
                            let d2 = d.clone();
                            view! {
                                <div class="recent-card" on:click=move |_| on_open.set(Some(d2.clone()))>
                                    <div class="recent-emoji">"\u{1F6CD}"</div>
                                    <div class="recent-brand">{d.brand.clone()}</div>
                                    <div class="recent-price">{sym}{format!("{:.0}", d.current_price)}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            }.into_view()
        }}
    }
}

// ───────────────────────── TRENDING ─────────────────────────
#[component]
fn TrendingTab(deals: DealsRes , on_open: WriteSignal<Option<Deal>>) -> impl IntoView {
    let ranked = move || {
        let mut v = deals.get().unwrap_or_default();
        v.sort_by(|a, b| {
            b.drop_percent
                .partial_cmp(&a.drop_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    };
    view! {
        <div class="sec-hdr"><div class="sec-title">"🔥 Hot right now"</div>
            <span class="sec-sub">"Updated hourly"</span></div>
        <p class="tab-lede">"Ranked by how hard the price just dropped. Move fast — these sell out."</p>
        <div class="trending-strip">
            {move || ranked().into_iter().take(8).enumerate().map(|(i,d)| {
                let sym = symbol(&d.currency).to_string();
                let short = d.name.split('—').next().unwrap_or(&d.name).trim().to_string();
                view! {
                    <div class="trend-card">
                        <div class="trend-rank">"#"{(i+1).to_string()}" TRENDING"</div>
                        <div class="trend-emoji">"\u{1F6CD}"</div>
                        <div class="trend-name">{d.brand.clone()}" "{short}</div>
                        <div class="trend-drop">"↓ "{d.drop_percent as i64}"% · "{sym}{format!("{:.0}", d.current_price)}</div>
                    </div>
                }
            }).collect_view()}
        </div>
        <div class="divider"></div>
        <div class="sec-hdr"><div class="sec-title">"Biggest savings 💰"</div></div>
        <div class="deal-grid">
            {move || ranked().into_iter().take(6)
                .map(|d| view!{ <DealCard deal=d on_open=on_open/> }).collect_view()}
        </div>
    }
}

// ───────────────────────── OUTFIT BUILDER ─────────────────────────
#[component]
fn OutfitTab(deals: DealsRes ) -> impl IntoView {
    let slots = create_rw_signal::<HashMap<&'static str, Deal>>(HashMap::new());
    let (search, set_search) = create_signal(String::new());
    let defs = [
        ("top", "Top / outerwear", "👕"),
        ("bottom", "Trousers / skirt", "👖"),
        ("shoes", "Shoes", "👟"),
        ("bag", "Bag / accessory", "👜"),
    ];

    let total = move || slots.get().values().map(|d| d.current_price).sum::<f64>();
    let cur = move || {
        slots
            .get()
            .values()
            .next()
            .map(|d| symbol(&d.currency).to_string())
            .unwrap_or_else(|| "£".into())
    };

    view! {
        <div class="sec-hdr"><div class="sec-title">"✨ Outfit Builder"</div></div>
        <p class="tab-lede">"Build a full look from tracked deals. See the total, share it."</p>
        <div class="outfit-wrap">
            <div class="outfit-canvas">
                <div class="outfit-canvas-header"><span class="outfit-canvas-title">"Your look"</span></div>
                <div class="outfit-slots">
                    {move || defs.iter().map(|(key,label,emoji)| {
                        let (key,label,emoji) = (*key,*label,*emoji);
                        match slots.get().get(key).cloned() {
                            Some(d) => {
                                let sym = symbol(&d.currency).to_string();
                                let short = d.name.split('—').next().unwrap_or(&d.name).trim().to_string();
                                view!{
                                    <div class="outfit-slot filled">
                                        <div class="slot-emoji">{emoji}</div>
                                        <div class="slot-item-name">{d.brand.clone()}<br/>{short}</div>
                                        <div class="slot-item-price">{sym}{format!("{:.0}", d.current_price)}</div>
                                        <div class="slot-remove" on:click=move |_| slots.update(|m| { m.remove(key); })>"✕"</div>
                                    </div>
                                }.into_view()
                            }
                            None => view!{
                                <div class="outfit-slot">
                                    <div class="slot-emoji">{emoji}</div>
                                    <div class="slot-label">"+ Add "{label}</div>
                                </div>
                            }.into_view()
                        }
                    }).collect_view()}
                </div>
                <div class="outfit-total">
                    <div>
                        <div class="outfit-total-label">"Total outfit cost"</div>
                        <div class="outfit-total-val">{move || format!("{}{:.0}", cur(), total())}</div>
                    </div>
                    <div style="display:flex;gap:8px">
                        <button class="btn-ghost" style="font-size:13px;padding:8px 16px"
                            on:click=move |_| slots.set(HashMap::new())>"Clear"</button>
                        <button class="outfit-save-btn">"Share look →"</button>
                    </div>
                </div>
            </div>
            <div class="item-picker">
                <div class="picker-header">"Add items to outfit"</div>
                <input class="picker-search" placeholder="Search items…"
                    on:input=move |e| set_search.set(event_target_value(&e)) />
                <div class="picker-list">
                    {move || {
                        let q = search.get().to_lowercase();
                        deals.get().unwrap_or_default().into_iter()
                            .filter(|d| q.is_empty() || d.name.to_lowercase().contains(&q) || d.brand.to_lowercase().contains(&q))
                            .map(|d| {
                                let sym = symbol(&d.currency).to_string();
                                let d2 = d.clone();
                                view!{
                                    <div class="picker-item">
                                        <div class="picker-emoji">"\u{1F6CD}"</div>
                                        <div class="picker-info">
                                            <div class="picker-name">{d.brand.clone()}" — "{d.name.clone()}</div>
                                            <div class="picker-price">{sym}{format!("{:.0}", d.current_price)}</div>
                                        </div>
                                        <button class="picker-add" on:click=move |_| {
                                            let d = d2.clone();
                                            let slot = cat_to_slot(&d.category);
                                            slots.update(|m| { m.insert(slot, d); });
                                        }>"Add"</button>
                                    </div>
                                }
                            }).collect_view()
                    }}
                </div>
            </div>
        </div>
    }
}

// ───────────────────────── BRANDS (live-counted) ─────────────────────────
#[component]
fn BrandsTab(deals: DealsRes ) -> impl IntoView {
    let followed = create_rw_signal::<HashSet<String>>(HashSet::new());
    let excluded = create_rw_signal::<HashSet<String>>(HashSet::new());
    view! {
        <div class="sec-hdr"><div class="sec-title">"Browse brands"</div></div>
        <p class="tab-lede">"Follow for deal alerts · Exclude to hide a brand's items"</p>
        <div class="brands-grid">
            {move || {
                let mut counts: BTreeMap<String, usize> = BTreeMap::new();
                for d in deals.get().unwrap_or_default() { *counts.entry(d.brand).or_insert(0) += 1; }
                counts.into_iter().map(|(name, n)| {
                    let (nf, ne, nc, nt, nb) = (name.clone(), name.clone(), name.clone(), name.clone(), name.clone());
                    view! {
                        <div class="brand-card"
                             class:followed=move || followed.get().contains(&nc)
                             class:excluded=move || excluded.get().contains(&nt)>
                            <div class="brand-excl" title="Exclude brand"
                                on:click=move |_| excluded.update(|s| { if !s.remove(&ne) { s.insert(ne.clone()); } })>"🚫"</div>
                            <div class="brand-logo">"\u{1F6CD}"</div>
                            <div class="brand-name">{name}</div>
                            <div class="brand-deals">{n.to_string()}" deals"</div>
                            {

                            view!{
                                <button class="brand-follow-btn"
                                    on:click=move |_| followed.update(|s| { if !s.remove(&nf) { s.insert(nf.clone()); } })>
                                    {move || if followed.get().contains(&nb) {"Following"} else {"Follow"}}
                                </button>
                            }
                        }
                        </div>
                    }
                }).collect_view()
            }}
        </div>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (region, set_region) = create_signal("CA".to_string());
    let (panel_open, set_panel_open) = create_signal(false);
    let (detecting, set_detecting) = create_signal(true);
    let (modal_deal, set_modal_deal) = create_signal::<Option<Deal>>(None);
    let recent = create_rw_signal::<Vec<Deal>>(Vec::new());
    provide_context(recent);
    let (active_tab, set_active_tab) = create_signal("deals".to_string());
    let (active_filter, set_active_filter) = create_signal("all".to_string());
    let (sort_by, set_sort_by) = create_signal("newest".to_string());
    let (sale_only, set_sale_only) = create_signal(false);

    spawn_local(async move {
        let d = detect_region().await;
        set_region.set(d);
        set_detecting.set(false);
    });

    let deals = create_resource(
        move || (region.get(), sort_by.get(), sale_only.get()),
        |(r, sort, sale)| async move { fetch_deals(r, sort, sale).await },
    );
    let region_name = move || match region.get().as_str() {
        "GB" => "United Kingdom",
        "US" => "United States",
        "CA" => "Canada",
        _ => "Canada",
    };
    let cur_sym = move || match region.get().as_str() {
        "GB" => "£",
        "US" => "$",
        "CA" => "C$",
        _ => "C$",
    };

    let stat_count = move || deals.get().map(|d| d.len()).unwrap_or(0);
    let stat_avg = move || {
        deals
            .get()
            .map(|d| {
                let drops: Vec<f64> = d
                    .iter()
                    .filter(|x| x.drop_percent > 0.0)
                    .map(|x| x.drop_percent)
                    .collect();
                if drops.is_empty() {
                    0.0
                } else {
                    drops.iter().sum::<f64>() / drops.len() as f64
                }
            })
            .unwrap_or(0.0)
    };
    let stat_low = move || {
        deals
            .get()
            .and_then(|d| {
                d.iter()
                    .map(|x| x.current_price)
                    .fold(None, |acc: Option<f64>, p| {
                        Some(acc.map_or(p, |a| a.min(p)))
                    })
            })
            .unwrap_or(0.0)
    };

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
        <div class="ticker-wrap">
            <div class="ticker" inner_html=move || {
                let items = ticker_items();
                format!("{items}{items}")
            }></div>
        </div>

        <nav>
            <button class="panel-toggle" on:click=move |_| set_panel_open.update(|o| *o = !*o)>"☰"</button>
            <div class="logo">
                <span class="logo-drip">"Drip"</span><div class="logo-dot"></div><span class="logo-drop">"Drop"</span>
            </div>
            <div class="nav-region" on:click=move |_| set_panel_open.set(true)>{move || region.get()}" ▾"</div>
        </nav>

        <div class="tab-bar">
            <div class="tab" class:active=move || active_tab.get() == "deals"
                on:click=move |_| set_active_tab.set("deals".into())>"🔥 Deals"</div>
            <div class="tab" class:active=move || active_tab.get() == "trending"
                on:click=move |_| set_active_tab.set("trending".into())>"📈 Trending"</div>
            <div class="tab" class:active=move || active_tab.get() == "outfit"
                on:click=move |_| set_active_tab.set("outfit".into())>"✨ Outfit Builder"</div>
            <div class="tab" class:active=move || active_tab.get() == "boards"
                on:click=move |_| set_active_tab.set("boards".into())>"📌 Style Boards"</div>
            <div class="tab" class:active=move || active_tab.get() == "alerts"
                on:click=move |_| set_active_tab.set("alerts".into())>"🔔 Alerts"</div>
            <div class="tab" class:active=move || active_tab.get() == "brands"
                on:click=move |_| set_active_tab.set("brands".into())>"🏷 Brands"</div>
        </div>

        <div class=move || if panel_open.get() { "side-panel open" } else { "side-panel" }>
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
                    <button class="region-opt" class:active=move || region.get() == "CA"
                        on:click=move |_| set_region.set("CA".into())>"🇨🇦 Canada"</button>
                    <button class="region-opt" class:active=move || region.get() == "US"
                        on:click=move |_| set_region.set("US".into())>"🇺🇸 United States"</button>
                    <button class="region-opt" class:active=move || region.get() == "GB"
                        on:click=move |_| set_region.set("GB".into())>"🇬🇧 United Kingdom"</button>
                </div>
            </div>

            <div class="panel-section">
                <div class="panel-label">"Followed brands"</div>
                <div class="panel-hint">"Follow brands to get deal alerts."</div>
                <div class="panel-soon">"Coming soon"</div>
            </div>
        </div>
        <div class=move || if panel_open.get() { "panel-overlay show" } else { "panel-overlay" }
             on:click=move |_| set_panel_open.set(false)></div>

        <div class="page">
            {move || {
                if is_deals() {
                    view! {
                        <div class="hero">
                            <div class="hero-eyebrow">"— Price drops. Updated daily."</div>
                            <h1 class="hero-title">"Quality drips with the best "<em>"drops."</em></h1>
                            <p class="hero-tagline">"A shop that shows its receipts. Browse real products, track real prices, and catch the genuine drops — never a fake \"was\" price."</p>
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

                        <div class="filter-row">
                            <div class="fchip" class:active=move || active_filter.get() == "all"
                                on:click=move |_| set_active_filter.set("all".into())>"All"</div>
                            <div class="fchip" class:active=move || active_filter.get() == "womenswear"
                                on:click=move |_| set_active_filter.set("womenswear".into())>"👗 Womenswear"</div>
                            <div class="fchip" class:active=move || active_filter.get() == "footwear"
                                on:click=move |_| set_active_filter.set("footwear".into())>"👟 Footwear"</div>
                            <div class="fchip" class:active=move || active_filter.get() == "outerwear"
                                on:click=move |_| set_active_filter.set("outerwear".into())>"🧥 Outerwear"</div>
                            <div class="fchip" class:active=move || active_filter.get() == "accessories"
                                on:click=move |_| set_active_filter.set("accessories".into())>"👜 Accessories"</div>
                        </div>
                        
                        <div class="shop-controls">
                            <div class="sale-toggle" class:on=move || sale_only.get()
                                on:click=move |_| set_sale_only.update(|v| *v = !*v)>
                                <span class="sale-dot"></span>"On sale only"
                            </div>
                            <select class="sort-select" on:change=move |e| set_sort_by.set(event_target_value(&e))>
                                <option value="newest">"Newest"</option>
                                <option value="drop">"Biggest drop"</option>
                                <option value="price_asc">"Price: low to high"</option>
                                <option value="price_desc">"Price: high to low"</option>
                            </select>
                        </div>
                        
                        <RecentStrip on_open=set_modal_deal/>

                        <div class="sec-hdr"><div class="sec-title">"Shop — "{region_name}</div></div>

                        <div class="deal-grid">
                            <Suspense fallback=move || view!{ <div class="loading">"Loading deals…"</div> }>
                                {move || {
                                    if detecting.get() { return view!{ <div class="loading">"Finding your region…"</div> }.into_view(); }
                                    let filt = active_filter.get();
                                    deals.get().map(|list| {
                                        let filtered: Vec<Deal> = list.into_iter()
                                            .filter(|d| filt == "all" || d.category == filt).collect();
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
                    match active_tab.get().as_str() {
                        "trending" => view!{ <TrendingTab deals=deals on_open=set_modal_deal/> }.into_view(),
                        "outfit"   => view!{ <OutfitTab deals=deals/> }.into_view(),
                        "brands"   => view!{ <BrandsTab deals=deals/> }.into_view(),
                        other => {
                            let (icon, title, desc) = match other {
                                "boards" => ("📌", "Style Boards", "Save deals into shareable boards. Friends see live prices."),
                                "alerts" => ("🔔", "Price Alerts", "Get notified the moment a tracked item drops in price."),
                                _ => ("✨", "Coming soon", ""),
                            };
                            view!{
                                <div class="coming-soon">
                                    <div class="cs-icon">{icon}</div>
                                    <div class="cs-title">{title}</div>
                                    <div class="cs-desc">{desc}</div>
                                    <div class="cs-badge">"Coming soon"</div>
                                    <button class="btn-ghost" on:click=move |_| set_active_tab.set("deals".into())>"← Back to deals"</button>
                                </div>
                            }.into_view()
                        }
                    }
                }
            }}
        </div>

        {move || modal_deal.get().map(|d| view!{ <PriceModal deal=d on_close=set_modal_deal/> })}
         <Footer/>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <CookieNotice/>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/privacy" view=Privacy/>
                <Route path="/terms" view=Terms/>
                <Route path="/disclosure" view=Disclosure/>
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
