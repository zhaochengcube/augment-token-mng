use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::data::subscription::models::Subscription;
use tokio_postgres::Row;

/// 订阅数据库映射器
pub struct SubscriptionMapper;

impl AccountDbMapper<Subscription> for SubscriptionMapper {
    fn from_row(row: &Row) -> Result<Subscription, StorageError> {
        Ok(Subscription {
            id: row.get(0),
            website: row.get(1),
            website_url: row.get(2),
            start_date: row.get(3),
            duration_months: row.get(4),
            expiry_date: row.get(5),
            cost: row.get(6),
            tag: row.get(7),
            tag_color: row.get(8),
            notes: row.get(9),
            created_at: row.get(10),
            updated_at: row.get(11),
            version: row.get(12),
            deleted: row.get(13),
        })
    }

    fn select_columns() -> &'static str {
        "id, website, website_url, start_date, duration_months, expiry_date, cost, tag, tag_color, notes, created_at, updated_at, version, deleted"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO subscriptions
            (id, website, website_url, start_date, duration_months, expiry_date, cost, tag, tag_color, notes, created_at, updated_at, version, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (id) DO UPDATE SET
            website = EXCLUDED.website,
            website_url = EXCLUDED.website_url,
            start_date = EXCLUDED.start_date,
            duration_months = EXCLUDED.duration_months,
            expiry_date = EXCLUDED.expiry_date,
            cost = EXCLUDED.cost,
            tag = EXCLUDED.tag,
            tag_color = EXCLUDED.tag_color,
            notes = EXCLUDED.notes,
            updated_at = EXCLUDED.updated_at,
            version = EXCLUDED.version,
            deleted = EXCLUDED.deleted
        "#
    }

    fn to_params(subscription: &Subscription, version: i64) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        vec![
            Box::new(subscription.id.clone()),
            Box::new(subscription.website.clone()),
            Box::new(subscription.website_url.clone()),
            Box::new(subscription.start_date.clone()),
            Box::new(subscription.duration_months),
            Box::new(subscription.expiry_date.clone()),
            Box::new(subscription.cost),
            Box::new(subscription.tag.clone()),
            Box::new(subscription.tag_color.clone()),
            Box::new(subscription.notes.clone()),
            Box::new(subscription.created_at),
            Box::new(subscription.updated_at),
            Box::new(version),
            Box::new(subscription.deleted),
        ]
    }
}

