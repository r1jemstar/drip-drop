pub mod deals;
pub mod brands;
pub mod users;
pub mod alerts;
pub mod boards;
pub mod tags;
pub mod admin;

use axum::{Router, routing::{get, post, put, delete}};
use sqlx::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/admin/ingest",    post(admin::run_ingest))
        .route("/admin/products",      post(admin::add_product))
        .route("/admin/products/price",put(admin::update_price))
        .route("/admin/products/batch",post(admin::add_batch))
        .route("/deals",               get(deals::list))
        .route("/deals/:id",           get(deals::get_one))
        .route("/deals/:id/history",   get(deals::price_history))
        .route("/deals/:id/compare",   get(deals::compare_prices))
        .route("/brands",              get(brands::list))
        .route("/brands/:slug/deals",  get(brands::deals))
        .route("/users/register",      post(users::register))
        .route("/users/login",         post(users::login))
        .route("/users/me",            get(users::me))
        .route("/users/me/preferences",put(users::update_preferences))
        .route("/alerts",              get(alerts::list).post(alerts::create))
        .route("/alerts/:id",          delete(alerts::delete))
        .route("/boards",              get(boards::list).post(boards::create))
        .route("/boards/:id",          get(boards::get_one).put(boards::update))
        .route("/boards/:slug/public", get(boards::public_view))
        .route("/tags/item/:item_id",  get(tags::list_for_item))
        .route("/tags",                post(tags::create))
        .route("/tags/:id/upvote",     post(tags::upvote))
        .with_state(pool)
}
