use axum::{extract::{Path, State}, Json};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn get_one(State(_p): State<PgPool>, Path(_id): Path<Uuid>) -> Json<serde_json::Value> { Json(serde_json::json!(null)) }
pub async fn create(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
pub async fn update(State(_p): State<PgPool>, Path(_id): Path<Uuid>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
pub async fn delete(State(_p): State<PgPool>, Path(_id): Path<Uuid>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
pub async fn deals(State(_p): State<PgPool>, Path(_slug): Path<String>) -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn register(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
pub async fn login(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!({"token":""})) }
pub async fn me(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn update_preferences(State(_p): State<PgPool>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
pub async fn public_view(State(_p): State<PgPool>, Path(_slug): Path<String>) -> Json<serde_json::Value> { Json(serde_json::json!({})) }
pub async fn list_for_item(State(_p): State<PgPool>, Path(_id): Path<Uuid>) -> Json<serde_json::Value> { Json(serde_json::json!([])) }
pub async fn upvote(State(_p): State<PgPool>, Path(_id): Path<Uuid>) -> Json<serde_json::Value> { Json(serde_json::json!({"ok":true})) }
