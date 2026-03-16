use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::data::bookmark::models::Bookmark;
use tokio_postgres::Row;

/// 书签数据库映射器
pub struct BookmarkMapper;

impl AccountDbMapper<Bookmark> for BookmarkMapper {
    fn from_row(row: &Row) -> Result<Bookmark, StorageError> {
        Ok(Bookmark {
            id: row.get(0),
            name: row.get(1),
            url: row.get(2),
            description: row.get(3),
            tag: row.get(4),
            tag_color: row.get(5),
            created_at: row.get(6),
            updated_at: row.get(7),
            version: row.get(8),
            deleted: row.get(9),
        })
    }

    fn select_columns() -> &'static str {
        "id, name, url, description, tag, tag_color, created_at, updated_at, version, deleted"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO bookmarks
            (id, name, url, description, tag, tag_color, created_at, updated_at, version, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            url = EXCLUDED.url,
            description = EXCLUDED.description,
            tag = EXCLUDED.tag,
            tag_color = EXCLUDED.tag_color,
            updated_at = EXCLUDED.updated_at,
            version = EXCLUDED.version,
            deleted = EXCLUDED.deleted
        "#
    }

    fn to_params(
        bookmark: &Bookmark,
        version: i64,
    ) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        vec![
            Box::new(bookmark.id.clone()),
            Box::new(bookmark.name.clone()),
            Box::new(bookmark.url.clone()),
            Box::new(bookmark.description.clone()),
            Box::new(bookmark.tag.clone()),
            Box::new(bookmark.tag_color.clone()),
            Box::new(bookmark.created_at),
            Box::new(bookmark.updated_at),
            Box::new(version),
            Box::new(bookmark.deleted),
        ]
    }
}
