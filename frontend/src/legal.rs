use leptos::*;

const CONTACT: &str = "dripdrop.deals42@gmail.com";
const UPDATED: &str = "July 2026";

#[component]
pub fn Privacy() -> impl IntoView {
    view! {
        <div class="legal-page">
            <a href="/" class="legal-back">"← Back to Drip Drop"</a>
            <h1 class="legal-h1">"Privacy Policy"</h1>
            <p class="legal-updated">"Last updated: "{UPDATED}</p>

            <p class="legal-p">"Drip Drop (\"we\", \"us\") is a personal, independently-run fashion price-tracking project. This policy explains what limited data we handle and how. We are committed to collecting as little as possible."</p>

            <h2 class="legal-h2">"Who we are"</h2>
            <p class="legal-p">"Drip Drop is operated by a sole individual based in Prince Edward Island, Canada. For any privacy question or request, contact "<a class="legal-link" href=format!("mailto:{CONTACT}")>{CONTACT}</a>"."</p>

            <h2 class="legal-h2">"What we collect"</h2>
            <p class="legal-p">"We do not require accounts, and we do not ask for your name, email, or any personal details to browse the site. We do not build user profiles."</p>
            <p class="legal-p">"We use "<strong>"Cloudflare Web Analytics"</strong>", a privacy-first, cookieless analytics tool. It measures aggregate engagement only — such as total page views, referring sites, and visitor country — without cookies and without tracking or identifying individual people. We see numbers, not users."</p>

            <h2 class="legal-h2">"Cookies"</h2>
            <p class="legal-p">"Drip Drop does not set its own tracking cookies. Our analytics are cookieless."</p>
            <p class="legal-p">"When you click an outbound \"Shop this deal\" link, our affiliate partner "<strong>"AWIN"</strong>" (and the retailer you visit) may set cookies on their own domains to attribute a resulting purchase to us so we can earn a commission. These are third-party cookies governed by AWIN's and the retailer's own privacy policies. They activate only when you choose to click through to a store."</p>

            <h2 class="legal-h2">"How we use data"</h2>
            <p class="legal-p">"Aggregate analytics are used solely to understand overall traffic and improve the site. We do not sell data. We do not share data with advertisers beyond the standard affiliate referral described above."</p>

            <h2 class="legal-h2">"Your rights"</h2>
            <p class="legal-p">"Because we do not collect personal information, there is typically nothing personal to access, correct, or delete. If you believe we hold any data about you, or have questions under Canada's PIPEDA, the UK/EU GDPR, or similar laws, email "<a class="legal-link" href=format!("mailto:{CONTACT}")>{CONTACT}</a>" and we will respond."</p>

            <h2 class="legal-h2">"Third-party sites"</h2>
            <p class="legal-p">"Outbound links lead to third-party retailers. We are not responsible for their content or privacy practices. Review their policies before purchasing."</p>

            <h2 class="legal-h2">"Changes"</h2>
            <p class="legal-p">"We may update this policy. The \"last updated\" date above reflects the latest version."</p>
        </div>
    }
}

#[component]
pub fn Terms() -> impl IntoView {
    view! {
        <div class="legal-page">
            <a href="/" class="legal-back">"← Back to Drip Drop"</a>
            <h1 class="legal-h1">"Terms of Use"</h1>
            <p class="legal-updated">"Last updated: "{UPDATED}</p>

            <p class="legal-p">"By using Drip Drop, you agree to these terms. If you disagree, please do not use the site."</p>

            <h2 class="legal-h2">"What Drip Drop is"</h2>
            <p class="legal-p">"Drip Drop is an independent fashion price-tracking tool. We display prices, discounts, and price history gathered from retailers and affiliate feeds, and link out to those retailers."</p>

            <h2 class="legal-h2">"Accuracy of prices"</h2>
            <p class="legal-p">"Prices, discounts, availability, and stock shown on Drip Drop are provided for reference and may be delayed, cached, or out of date. The retailer's own site is always the authoritative source. Always confirm the final price on the retailer's page before buying. We are not liable for discrepancies."</p>

            <h2 class="legal-h2">"Affiliate links"</h2>
            <p class="legal-p">"Many outbound links are affiliate links. If you click one and make a purchase, we may earn a commission at no extra cost to you. See our "<a class="legal-link" href="/disclosure">"Affiliate Disclosure"</a>" for details."</p>

            <h2 class="legal-h2">"No warranty"</h2>
            <p class="legal-p">"Drip Drop is provided \"as is\", without warranties of any kind. We do not guarantee the site will be uninterrupted, error-free, or that any deal will remain available."</p>

            <h2 class="legal-h2">"Limitation of liability"</h2>
            <p class="legal-p">"To the fullest extent permitted by law, Drip Drop and its owner are not liable for any loss arising from use of the site, reliance on displayed prices, or purchases made through outbound links."</p>

            <h2 class="legal-h2">"Governing law"</h2>
            <p class="legal-p">"These terms are governed by the laws of the Province of Prince Edward Island and the federal laws of Canada applicable therein. Any dispute is subject to the courts of Prince Edward Island, Canada."</p>

            <h2 class="legal-h2">"Contact"</h2>
            <p class="legal-p">"Questions: "<a class="legal-link" href=format!("mailto:{CONTACT}")>{CONTACT}</a>"."</p>
        </div>
    }
}

#[component]
pub fn Disclosure() -> impl IntoView {
    view! {
        <div class="legal-page">
            <a href="/" class="legal-back">"← Back to Drip Drop"</a>
            <h1 class="legal-h1">"Affiliate Disclosure"</h1>
            <p class="legal-updated">"Last updated: "{UPDATED}</p>

            <p class="legal-p">"Drip Drop is a member of affiliate programmes, including the "<strong>"AWIN"</strong>" affiliate network. This is how the site is funded."</p>

            <h2 class="legal-h2">"How it works"</h2>
            <p class="legal-p">"When you click a \"Shop this deal\" or similar outbound link and go on to make a purchase at the retailer, we may earn a small commission. This comes from the retailer's marketing budget — "<strong>"it costs you nothing extra"</strong>", and your price is unaffected."</p>

            <h2 class="legal-h2">"Our promise"</h2>
            <p class="legal-p">"Commissions never change which deals we show or how we rank them. Price data, discounts, and price history are reported as-is from retailers and feeds. We surface genuine drops, not paid placements."</p>

            <h2 class="legal-h2">"Questions"</h2>
            <p class="legal-p">"Email "<a class="legal-link" href=format!("mailto:{CONTACT}")>{CONTACT}</a>" any time."</p>
        </div>
    }
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <div class="footer-inner">
                <div class="footer-brand">"Drip Drop"</div>
                <div class="footer-links">
                    <a href="/privacy">"Privacy"</a>
                    <a href="/terms">"Terms"</a>
                    <a href="/disclosure">"Affiliate Disclosure"</a>
                    <a href=format!("mailto:{CONTACT}")>"Contact"</a>
                </div>
                <div class="footer-note">"Drip Drop earns commission on some outbound links. Prices are for reference — confirm on the retailer's site."</div>
            </div>
        </footer>
    }
}

#[component]
pub fn CookieNotice() -> impl IntoView {
    let (dismissed, set_dismissed) = create_signal(false);
    view! {
        {move || (!dismissed.get()).then(|| view!{
            <div class="cookie-notice">
                <div class="cookie-text">
                    "Drip Drop uses cookieless analytics — we count visits, not people. Clicking through to a retailer may set affiliate cookies so we earn commission. "
                    <a class="cookie-link" href="/privacy">"Privacy Policy"</a>
                </div>
                <button class="cookie-btn" on:click=move |_| set_dismissed.set(true)>"Got it"</button>
            </div>
        })}
    }
}