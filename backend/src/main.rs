mod api;
mod auth;
mod db;
mod scraper;
mod alerts;
mod pricing;

use anyhow::Result;
use axum::{Router, http::Method};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().max_connections(10).connect(&db_url).await?;
    sqlx::migrate!("../migrations").run(&pool).await?;
    tracing::info!("DB migrations applied");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    let app = Router::new()
        .nest("/api", api::router(pool.clone()))
        .layer(cors);

    let pool_clone = pool.clone();
    tokio::spawn(async move { scraper::run_scheduler(pool_clone).await; });

    let addr = "0.0.0.0:3000";
    tracing::info!("Drip Drop listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
