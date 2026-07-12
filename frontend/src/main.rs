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

#[derive(Debug, Clone, Deserialize)]
struct GeoResponse { country: String }

// Your live backend on Render
const API_BASE: &str = "https://drip-drop-api.onrender.com";

fn symbol(currency: &str) -> &str {
    match currency {
        "GBP" => "£", "USD" => "$", "CAD" => "C$", "AUD" => "A$", _ => "£",
    }
}

// Map a detected country code to a supported region (fallback CA — where our data is)
fn country_to_region(country: &str) -> String {
    match country {
        "GB" => "GB",
        "US" => "US",
        "CA" => "CA",
        _ => "CA", // default everyone else to CA for now
    }.to_string()
}

async fn fetch_deals(region: String) -> Vec<Deal> {
    let url = format!("{API_BASE}/api/deals?region={region}");
    match gloo_net::http::Request::get(&url).send().await {
        Ok(resp) => resp.json::<Vec<Deal>>().await.unwrap_or_default(),
        Err(_) => vec![],
    }
}

// Ask our Cloudflare Pages Function which country the visitor is in
async fn detect_region() -> String {
    match gloo_net::http::Request::get("/geo").send().await {
        Ok(resp) => match resp.json::<GeoResponse>().await {
            Ok(g) => country_to_region(&g.country),
            Err(_) => "CA".to_string(),
        },
        Err(_) => "CA".to_string(),
    }
}

#[component]
fn DealCard(deal: Deal) -> impl IntoView {
    let sym = symbol(&deal.currency).to_string();
    let save = deal.was_price - deal.current_price;
    let sizes = deal.sizes.clone();
    let has_image = false; // no image_url yet — falls back to emoji

    // deal quality label based on drop %
    let (quality, quality_class) = if deal.drop_percent >= 40.0 {
        ("🔥 Amazing deal", "q-hot")
    } else if deal.drop_percent >= 25.0 {
        ("Great deal", "q-good")
    } else if deal.drop_percent > 0.0 {
        ("Small drop", "q-small")
    } else {
        ("", "")
    };

    view! {
        <div class="deal-card">
            <div class="card-img">
                {if has_image {
                    view!{ <div></div> }.into_view()
                } else {
                    view!{ <div class="card-emoji">"🛍"</div> }.into_view()
                }}
                {(deal.drop_percent > 0.0).then(|| view!{
                    <div class="card-badge">"-" {deal.drop_percent as i64} "%"</div>
                })}
            </div>
            <div class="card-body">
                <div class="card-brand">{deal.brand.clone()}</div>
                <div class="card-name">{deal.name.clone()}</div>
                {(!quality.is_empty()).then(|| view!{
                    <div class=format!("quality {quality_class}")>{quality}</div>
                })}
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
                <a href={deal.affiliate_url.clone()} target="_blank" class="card-go-btn">"Shop this deal →"</a>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (region, set_region) = create_signal("CA".to_string());
    let (panel_open, set_panel_open) = create_signal(false);
    let (detecting, set_detecting) = create_signal(true);

    // On load: detect region from Cloudflare, then set it
    spawn_local(async move {
        let detected = detect_region().await;
        set_region.set(detected);
        set_detecting.set(false);
    });

    let deals = create_resource(
        move || region.get(),
        |r| async move { fetch_deals(r).await },
    );

    let region_name = move || match region.get().as_str() {
        "GB" => "United Kingdom", "US" => "United States", "CA" => "Canada", _ => "Canada",
    };

    view! {
        <div class="ticker-wrap">
            <div class="ticker-static">"✦ DRIP DROP — quality drips with the best drops"</div>
        </div>

        <nav>
            <button class="panel-toggle" on:click=move |_| set_panel_open.update(|o| *o = !*o)>
                "☰"
            </button>
            <div class="logo">
                <span class="logo-drip">"Drip"</span>
                <div class="logo-dot"></div>
                <span class="logo-drop">"Drop"</span>
            </div>
            <div class="nav-region" on:click=move |_| set_panel_open.set(true)>
                {move || region.get()} " ▾"
            </div>
        </nav>

        // ── LEFT SIDE PANEL ──
        <div class=move || if panel_open.get() { "side-panel open" } else { "side-panel" }>
            <div class="panel-header">
                <span>"Settings"</span>
                <button class="panel-close" on:click=move |_| set_panel_open.set(false)>"✕"</button>
            </div>
            <div class="panel-section">
                <div class="panel-label">"Region"</div>
                <div class="panel-hint">"Prices and deals shown for your region"</div>
                <div class="region-options">
                    <button class="region-opt" class:active=move || region.get()=="CA"
                        on:click=move |_| set_region.set("CA".to_string())>
                        "🇨🇦 Canada"
                    </button>
                    <button class="region-opt" class:active=move || region.get()=="US"
                        on:click=move |_| set_region.set("US".to_string())>
                        "🇺🇸 United States"
                    </button>
                    <button class="region-opt" class:active=move || region.get()=="GB"
                        on:click=move |_| set_region.set("GB".to_string())>
                        "🇬🇧 United Kingdom"
                    </button>
                </div>
            </div>
            <div class="panel-section">
                <div class="panel-label">"More coming soon"</div>
                <div class="panel-hint">"Filters, followed brands, and style boards will live here."</div>
            </div>
        </div>
        <div class=move || if panel_open.get() { "panel-overlay show" } else { "panel-overlay" }
             on:click=move |_| set_panel_open.set(false)></div>

        <div class="page">
            <div class="hero">
                <div class="hero-eyebrow">"Price drops. Updated daily."</div>
                <h1 class="hero-title">"Quality drips with the best "<em>"drops."</em></h1>
            </div>

            <div class="sec-hdr">
                <div class="sec-title">"Today's drops — " {region_name}</div>
            </div>

            <div class="deal-grid">
                <Suspense fallback=move || view!{ <div class="loading">"Loading deals…"</div> }>
                    {move || {
                        if detecting.get() {
                            return view!{ <div class="loading">"Finding your region…"</div> }.into_view();
                        }
                        deals.get().map(|list| {
                            if list.is_empty() {
                                view!{ <div class="loading">"No deals in this region yet — try switching region in the menu."</div> }.into_view()
                            } else {
                                list.into_iter().map(|d| view!{ <DealCard deal=d/> }).collect_view()
                            }
                        }).unwrap_or_else(|| view!{ <div class="loading">"Loading…"</div> }.into_view())
                    }}
                </Suspense>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}