//! Feed ingest engine — runs every 6h via tokio-cron-scheduler.
//! Downloads AWIN JSONL, diffs prices, writes to DB, triggers alerts.
use anyhow::Result;
use sqlx::PgPool;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn run_scheduler(pool: PgPool) {
    let sched = JobScheduler::new().await.expect("scheduler init");
    let p = pool.clone();
    sched.add(Job::new_async("0 0 */6 * * *", move |_, _| {
        let pool = p.clone();
        Box::pin(async move {
            if let Err(e) = ingest_all_feeds(&pool).await {
                tracing::error!("Ingest error: {e}");
            }
        })
    }).unwrap()).await.unwrap();
    sched.start().await.unwrap();
    loop { tokio::time::sleep(std::time::Duration::from_secs(3600)).await; }
}

async fn ingest_all_feeds(pool: &PgPool) -> Result<()> {
    // TODO:
    // 1. SELECT id, awin_id FROM brands WHERE active = true AND awin_id IS NOT NULL
    // 2. For each: GET https://productdata.awin.com/datafeed/...
    // 3. Stream JSONL line by line via reqwest
    // 4. Deserialise each line into AwinProduct struct
    // 5. INSERT INTO items ... ON CONFLICT (sku, brand_id) DO UPDATE SET ...
    // 6. If current_price != excluded_price: INSERT INTO price_history
    // 7. Call alerts::process_drops(pool, dropped_ids).await
    tracing::info!("Ingest run complete");
    Ok(())
}
