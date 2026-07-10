use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn process_drops(pool: &PgPool, dropped_item_ids: Vec<Uuid>) -> Result<()> {
    for item_id in dropped_item_ids {
        // SELECT u.email, a.target_price, i.name, i.current_price, i.affiliate_url
        // FROM alerts a JOIN users u ON u.id = a.user_id JOIN items i ON i.id = a.item_id
        // WHERE a.item_id = $1 AND a.active = true
        // AND (a.target_price IS NULL OR i.current_price <= a.target_price)
        // Then send email via Lettre
        tracing::info!("Alert fired for item {item_id}");
    }
    Ok(())
}
