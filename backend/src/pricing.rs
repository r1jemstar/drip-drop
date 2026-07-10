//! Layer 1 — price recording & drop detection.
//! Pure computation. No external APIs. This is the heart of Drip Drop.

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Record a price for an item, but ONLY if it changed from the last recorded value.
/// This keeps price_history clean (idempotent) and returns whether a drop happened.
pub struct PriceUpdate {
    pub item_id:     Uuid,
    pub new_price:   f64,
    pub dropped:     bool,   // did price go DOWN?
    pub drop_percent: f64,
}

/// Called whenever we see a fresh price for an item (manual add, or later, feed ingest).
pub async fn record_price(pool: &PgPool, item_id: Uuid, new_price: f64) -> Result<PriceUpdate> {
    // 1. Get the most recent recorded price for this item
    let last: Option<(f64,)> = sqlx::query_as(
        r#"
        SELECT price::float8
        FROM price_history
        WHERE item_id = $1
        ORDER BY recorded_at DESC
        LIMIT 1
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?;

    let last_price = last.map(|(p,)| p);

    // 2. Only insert history if price actually changed (idempotency)
    let changed = match last_price {
        Some(p) => (p - new_price).abs() > f64::EPSILON,
        None => true, // first time we've ever seen it
    };

    if changed {
        sqlx::query(
            "INSERT INTO price_history (item_id, price) VALUES ($1, $2)",
        )
        .bind(item_id)
        .bind(new_price)
        .execute(pool)
        .await?;
    }

    // 3. Compute the "real was-price" = highest price seen in last 90 days
    //    (falls back to the item's stored was_price if no history yet)
    let was_row: Option<(f64,)> = sqlx::query_as(
        r#"
        SELECT MAX(price)::float8
        FROM price_history
        WHERE item_id = $1
          AND recorded_at > NOW() - INTERVAL '90 days'
        "#,
    )
    .bind(item_id)
    .fetch_optional(pool)
    .await?;

    let stored_was: (f64,) = sqlx::query_as(
        "SELECT was_price::float8 FROM items WHERE id = $1",
    )
    .bind(item_id)
    .fetch_one(pool)
    .await?;

    // use the higher of (90-day observed high, stored RRP) as the honest baseline
    let observed_high = was_row.and_then(|(m,)| if m > 0.0 { Some(m) } else { None });
    let was = observed_high.map(|h| h.max(stored_was.0)).unwrap_or(stored_was.0);

    // 4. Recalculate drop %
    let drop_percent = if was > 0.0 {
        ((was - new_price) / was * 100.0).max(0.0)
    } else {
        0.0
    };

    let dropped = match last_price {
        Some(p) => new_price < p,
        None => false,
    };

    // 5. Update the item row with fresh current price, was, and drop
    sqlx::query(
        r#"
        UPDATE items
        SET current_price = $2,
            was_price     = $3,
            drop_percent  = $4,
            updated_at    = NOW()
        WHERE id = $1
        "#,
    )
    .bind(item_id)
    .bind(new_price)
    .bind(was)
    .bind(drop_percent)
    .execute(pool)
    .await?;

    Ok(PriceUpdate { item_id, new_price, dropped, drop_percent })
}