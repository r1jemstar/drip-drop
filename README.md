# Drip Drop

A fashion price tracker built in Rust, front to back. Axum on the server, Leptos compiled to WebAssembly in the browser, Postgres underneath. No JavaScript framework anywhere in the stack.

Live: https://drip-drop-2vk.pages.dev
API: https://drip-drop-api.onrender.com

## The idea

Most "sale" prices are lies. A retailer sets an RRP nobody ever paid, marks it down, and calls it 40% off. Drip Drop ignores what the retailer claims and works out the discount from prices it has actually observed. If an item has never been cheaper than it is today, that's a real drop. If it's been sitting at the same price for three months with a "was" tag on it, Drip Drop shows no discount at all.

That's the whole product. Everything else is plumbing.

## Not trusting the input

The affiliate feeds arrive with `saving` and `savings_percent` columns already filled in. Ingesting them would have been one line and would have made the site look far busier on day one — hundreds of products with confident discount badges.

They're deliberately discarded. Those numbers come from the party that benefits from them being large, and there's no way to verify them.

Instead, every recorded price is compared against the highest price actually seen for that item over the previous 90 days, falling back to the stored RRP only when there is no history at all. The discount is derived from observed data. The consequence is that a fresh catalogue shows almost no discounts, because nothing has been watched long enough to know. Drops appear over the following days as history accumulates and they become provable.

This is the single most important decision in the codebase, and it cost the product its most flattering first impression.

## Price recording

History is only written when a price genuinely changes. Re-running the same feed ten times in a day produces one history row, not ten. That keeps the history table meaningful and makes ingest safely repeatable — a retry, a duplicate cron fire, or a manual run during debugging can't corrupt the record.

## Feed ingest

Affiliate feeds are inconsistent. Different networks, different formats, different column names for the same field, prices formatted for whichever locale the merchant happens to sit in. The ingest layer absorbs that.

It handles CSV, TSV, JSONL, JSON and XML, detected by sniffing content rather than trusting the file extension, and transparently decompresses gzip. Each field carries around eight aliases, so `search_price`, `sale_price` and `current_price` all resolve to the same value. Money parsing copes with `34.99`, `1.299,00`, `1,299.00` and bare numbers by determining which separator is actually the decimal one.

Malformed rows are skipped rather than aborting the run, but never silently. Skips are counted by reason and returned in the report, so a feed dropping 90% of rows because the wrong columns were selected looks nothing like one dropping four junk rows. Silent tolerance and hard failure are both wrong here; the useful behaviour is to keep going and say precisely what was lost.

Categories are normalised onto a fixed enum. Brands are created on demand with generated slugs.

## Scheduling

Feeds live in a table carrying their own cadence. A feed can run every six hours, daily, or weekly, and can be paused without a deploy. One endpoint ingests whatever is currently due; a GitHub Actions cron hits it every three hours and each feed fires only when its own interval has elapsed. The job fails loudly if any feed reports errors, so a broken feed surfaces immediately instead of rotting for weeks.

## Frontend

Leptos compiled to WebAssembly. Reactive signals and resources drive the UI, with region detection through a Cloudflare geo function and currency switching to match.

The price history chart is hand-built rather than pulled from a charting library. Geometry is computed in Rust and rendered as native SVG nodes so event handlers actually attach. An earlier version generated the SVG as a string and injected it through `inner_html` — visually identical, and impossible to make interactive, because injected markup has no reactive nodes to bind to. Rewriting it as real elements is what made the crosshair and per-point tooltip possible.

Beyond the catalogue there's a trending view ranked by real drop percentage, a stateful outfit builder, and a brands view counted live from the data rather than hardcoded.

## Security and secrets

Admin endpoints that write to the database are guarded by a shared token compared against an environment variable. The token lives in the Render environment and in GitHub Secrets, never in the repository or in the workflow file.

During development an affiliate API key was exposed in a feed URL while debugging. It was rotated immediately rather than left to expire. Feed URLs now live in the database and in secrets, not in shell history or committed files.

## Running it

Backend needs `DATABASE_URL` and `INGEST_TOKEN`. Migrations run automatically on boot.

```
cd backend
cargo run
```

Frontend needs `trunk`:

```
cd frontend
trunk serve
```

Production build and deploy:

```
cd frontend
cargo clean
trunk build --release
npx wrangler pages deploy dist --project-name drip-drop
```

`cargo clean` is needed before release builds on Windows due to a stale cache issue.

To register a feed, insert a row into `feeds` with its URL, region and interval. To trigger ingest manually:

```
POST /api/admin/ingest/due
x-ingest-token: <token>
```

## Privacy

Analytics are Cloudflare Web Analytics — cookieless, aggregate only. Page views, referrers, countries. No accounts, no profiles, no first-party tracking cookies. The only cookies involved are the affiliate ones set by the retailer on click-through, disclosed on every page.

This was a choice, not a default. Conventional analytics would have given more detail about individual visitors. Counting engagement doesn't require identifying anyone, so it doesn't.

## Deployment

Backend on Render, frontend on Cloudflare Pages, Postgres on Neon. Backend auto-deploys on push. The frontend needs a `_redirects` file at the root of `dist` so deep links to routed pages resolve instead of returning 404.

## Current state

634 products from live affiliate feeds, 207 detected price drops, 100% parse rate on production ingest runs. Approved AWIN publisher.

Not done: accounts and auth, so price alerts and saved boards remain honest placeholders rather than half-working features. No test suite. The catalogue is Canada-only — the multi-region infrastructure is built and working, the advertiser relationships for GB and US aren't there yet.

## On scraping

Drip Drop doesn't scrape. Everything comes from affiliate product feeds the retailer has agreed to publish.

Scraping would be faster and would unlock the large brands that don't run affiliate programmes at all. Commercial APIs exist that will do it. It breaks retailer terms of service, and a project whose entire premise is that its numbers can be trusted can't be built on data taken from people who didn't agree to give it.

## Where this sits

Drip Drop is the shipping project: something real, deployed, running unattended, with a commercial relationship behind it and data arriving whether or not anyone is watching. It's also a production WebAssembly application — Rust compiled to a browser target, serving real users, which is the delivery model this codebase exists partly to understand properly.

The deeper systems work lives elsewhere in the portfolio, principally in Viper, a compiled language with Cranelift and WebAssembly backends. This project is not that. What it demonstrates is the other half: that the thing gets finished, deployed, secured, and kept running.
