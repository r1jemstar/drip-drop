use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id:            Uuid,
    pub sku:           String,
    pub name:          String,
    pub brand_id:      Uuid,
    pub category:      String,
    pub current_price: f64,
    pub was_price:     f64,
    pub drop_percent:  f64,
    pub affiliate_url: String,
    pub image_url:     String,
    pub sizes:         Vec<String>,
    pub in_stock:      bool,
    pub expires_at:    Option<DateTime<Utc>>,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub price:       f64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brand {
    pub id:         Uuid,
    pub name:       String,
    pub slug:       String,
    pub awin_id:    Option<String>,
    pub cj_id:      Option<String>,
    pub commission: f64,
    pub active:     bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:              Uuid,
    pub email:           String,
    pub display_name:    String,
    pub is_premium:      bool,
    pub excluded_brands: Vec<Uuid>,
    pub created_at:      DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id:           Uuid,
    pub user_id:      Uuid,
    pub item_id:      Uuid,
    pub target_price: Option<f64>,
    pub active:       bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id:       Uuid,
    pub label:    String,
    pub tag_type: TagType,
    pub upvotes:  i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagType { System, Community, Personal }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleBoard {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub name:       String,
    pub is_shared:  bool,
    pub share_slug: Option<String>,
    pub item_ids:   Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}
