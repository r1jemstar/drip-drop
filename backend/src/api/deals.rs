use axum::{extract::{Path, Query, State}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// ── Response shape sent to the frontend ──
#[derive(Debug, Serialize)]
pub struct DealResponse {
    pub id:            Uuid,
    pub sku:           String,
    pub name:          String,
    pub brand:         String,
    pub category:      String,
    pub current_price: f64,
    pub was_price:     f64,
    pub drop_percent:  f64,
    pub currency:      String,
    pub affiliate_url: String,
    pub image_url:     Option<String>,
    pub sizes:         Vec<String>,
    pub in_stock:      bool,
    pub region:        String,
}

#[derive(Debug, Deserialize)]
pub struct DealQuery {
    pub region:   Option<String>,   // 'GB' | 'US' | 'CA' — defaults to GB
    pub category: Option<String>,   // 'womenswear' etc.
    pub min_drop: Option<f64>,      // e.g. 30 = 30%+ off
    pub sort:     Option<String>,   // 'drop' | 'price_asc' | 'price_desc'
    pub limit:    Option<i64>,
}

pub async fn list(
    State(pool): State<PgPool>,
    Query(q): Query<DealQuery>,
) -> Result<Json<Vec<DealResponse>>, (StatusCode, String)> {
    let region   = q.region.unwrap_or_else(|| "GB".to_string());
    let category = q.category;
    let min_drop = q.min_drop.unwrap_or(0.0);
    let limit    = q.limit.unwrap_or(50).min(100);

    // ORDER BY based on sort param
    let order = match q.sort.as_deref() {
        Some("price_asc")  => "i.current_price ASC",
        Some("price_desc") => "i.current_price DESC",
        _                   => "i.drop_percent DESC",  // default
    };

    let sql = format!(
        r#"
        SELECT
            i.id, i.sku, i.name, b.name AS brand, i.category::text AS category,
            i.current_price::float8, i.was_price::float8, i.drop_percent::float8,
            i.currency, i.affiliate_url,i.image_url, i.sizes, i.in_stock, i.region
        FROM items i
        JOIN brands b ON b.id = i.brand_id
        WHERE i.region = $1
          AND i.drop_percent >= $2
          AND ($3::text IS NULL OR i.category::text = $3)
        ORDER BY {order}
        LIMIT $4
        "#
    );

    let rows = sqlx::query_as::<_, (Uuid, String, String, String, String, f64, f64, f64, String, String, Option<String>, Vec<String>, bool, String)>(&sql)
        .bind(&region)
        .bind(min_drop)
        .bind(&category)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let deals = rows.into_iter().map(|r| DealResponse {
        id: r.0, sku: r.1, name: r.2, brand: r.3, category: r.4,
        current_price: r.5, was_price: r.6, drop_percent: r.7,
        currency: r.8, affiliate_url: r.9, image_url: r.10,
        sizes: r.11, in_stock: r.12, region: r.13,
    }).collect();

    Ok(Json(deals))
}

pub async fn get_one(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<DealResponse>>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, (Uuid, String, String, String, String, f64, f64, f64, String, String, Option<String>, Vec<String>, bool, String)>(
        r#"
        SELECT i.id, i.sku, i.name, b.name AS brand, i.category::text,
                i.current_price::float8, i.was_price::float8, i.drop_percent::float8,
                i.currency, i.affiliate_url, i.image_url, i.sizes, i.in_stock, i.region
        FROM items i JOIN brands b ON b.id = i.brand_id
        WHERE i.id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(row.map(|r| DealResponse {
        id: r.0, sku: r.1, name: r.2, brand: r.3, category: r.4,
        current_price: r.5, was_price: r.6, drop_percent: r.7,
        currency: r.8, affiliate_url: r.9, image_url: r.10,
        sizes: r.11, in_stock: r.12, region: r.13,
    })))
}

#[derive(Debug, Serialize)]
pub struct PricePointResponse {
    pub price:       f64,
    pub recorded_at: String,
}

pub async fn price_history(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<PricePointResponse>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (f64, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT price::float8, recorded_at
        FROM price_history
        WHERE item_id = $1
        ORDER BY recorded_at ASC
        "#
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let points = rows.into_iter().map(|(price, ts)| PricePointResponse {
        price,
        recorded_at: ts.to_rfc3339(),
    }).collect();

    Ok(Json(points))
}

pub async fn compare_prices(
    State(_pool): State<PgPool>,
    Path(_id): Path<Uuid>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "prices": [] }))
}

