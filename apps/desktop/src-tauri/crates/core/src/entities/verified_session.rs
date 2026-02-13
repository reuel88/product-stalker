use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Verified session entity
///
/// Stores verified browser sessions (after manual CAPTCHA solving) for reuse.
/// Sessions are domain-scoped and have an expiration date.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "verified_sessions")]
pub struct Model {
    /// Unique identifier (UUID v4)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Domain this session is valid for (e.g., "www.example.com")
    pub domain: String,

    /// JSON-encoded cookies from the verified session
    #[sea_orm(column_type = "Text")]
    pub cookies_json: String,

    /// User agent used during verification
    pub user_agent: String,

    /// Session expiration timestamp
    pub expires_at: DateTimeUtc,

    /// Creation timestamp
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
