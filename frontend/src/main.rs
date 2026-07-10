use leptos::*;
use serde::{Deserialize, Serialize};

// ── Mirror of the backend DealResponse ──
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

const API_BASE: &str = "http://localhost:3000";

// Currency symbol from code
fn symbol(currency: &str) -> &str {
    match currency {
        "GBP" => "£",
        "USD" => "$",
        "CAD" => "CAD$",
        "AUD" => "AUS$",
        _ => "£",
    }
}

async fn fetch_deals(region: String) -> Vec<Deal> {
    let url = format!("{API_BASE}/api/deals?region={region}");
    match gloo_net::http::Request::get(&url).send().await {
        Ok(resp) => resp.json::<Vec<Deal>>().await.unwrap_or_default(),
        Err(_) => vec![],
    }
}

#[component]
fn DealCard(deal: Deal) -> impl IntoView {
    let sym = symbol(&deal.currency).to_string();
    let save = deal.was_price - deal.current_price;
    let sizes = deal.sizes.clone();
    view! {
        <div class="deal-card">
            <div class="card-img">
                <div class="card-emoji">"👗"</div>
                <div class="card-badge">"-" {deal.drop_percent as i64} "%"</div>
            </div>
            <div class="card-body">
                <div class="card-brand">{deal.brand.clone()}</div>
                <div class="card-name">{deal.name.clone()}</div>
                <div class="size-row">
                    {sizes.into_iter().map(|s| view!{ <div class="sz">{s}</div> }).collect_view()}
                </div>
                <div class="card-price-row">
                    <span class="price-now">{sym.clone()}{format!("{:.0}", deal.current_price)}</span>
                    <span class="price-was">{sym.clone()}{format!("{:.0}", deal.was_price)}</span>
                    <span class="price-save">"Save " {sym.clone()}{format!("{:.0}", save)}</span>
                </div>
                <a href={deal.affiliate_url.clone()} target="_blank" class="card-go-btn">"Shop this deal →"</a>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (region, set_region) = create_signal("GB".to_string());

    // re-fetches whenever region changes
    let deals = create_resource(
        move || region.get(),
        |r| async move { fetch_deals(r).await },
    );

    view! {
        <div class="ticker-wrap">
            <div class="ticker-static">"✦ DRIP DROP — quality drips with the best drops"</div>
        </div>
        <nav>
            <div class="logo">
                <span class="logo-drip">"Drip"</span>
                <div class="logo-dot"></div>
                <span class="logo-drop">"Drop"</span>
            </div>
            <div class="region-switch">
                <button class="region-btn" class:active=move || region.get()=="GB"
                    on:click=move |_| set_region.set("GB".to_string())>"🇬🇧 GB"</button>
                <button class="region-btn" class:active=move || region.get()=="US"
                    on:click=move |_| set_region.set("US".to_string())>"🇺🇸 US"</button>
                <button class="region-btn" class:active=move || region.get()=="CA"
                    on:click=move |_| set_region.set("CA".to_string())>"🇨🇦 CA"</button>
            </div>
        </nav>

        <div class="page">
            <div class="hero">
                <div class="hero-eyebrow">"Price drops. Updated daily."</div>
                <h1 class="hero-title">"Quality drips with the best "<em>"drops."</em></h1>
            </div>

            <div class="sec-hdr">
                <div class="sec-title">"Today's drops — " {move || region.get()}</div>
            </div>

            <div class="deal-grid">
                <Suspense fallback=move || view!{ <div class="loading">"Loading deals…"</div> }>
                    {move || deals.get().map(|list| {
                        if list.is_empty() {
                            view!{ <div class="loading">"No deals in this region yet."</div> }.into_view()
                        } else {
                            list.into_iter().map(|d| view!{ <DealCard deal=d/> }).collect_view()
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}
