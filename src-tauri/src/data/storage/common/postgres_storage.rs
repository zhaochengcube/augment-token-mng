use super::traits::{AccountStorage, StorageError, SyncableAccount};
use crate::database::DatabaseManager;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio_postgres::Row;

/// 平台特定的数据库映射器 trait
pub trait AccountDbMapper<T: SyncableAccount>: Send + Sync + 'static {
    /// 从数据库行构建账号对象
    fn from_row(row: &Row) -> Result<T, StorageError>;

    /// 查询字段列表
    fn select_columns() -> &'static str;

    /// 生成 INSERT SQL
    fn insert_sql() -> &'static str;

    /// 获取 INSERT 参数
    fn to_params(
        account: &T,
        version: i64,
    ) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>;
}

/// 通用 PostgreSQL 存储
pub struct GenericPostgreSQLStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    db_manager: Arc<DatabaseManager>,
    _phantom: PhantomData<(T, M)>,
}

impl<T, M> GenericPostgreSQLStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            db_manager,
            _phantom: PhantomData,
        }
    }

    async fn get_pool(&self) -> Result<Arc<crate::database::DbPool>, StorageError> {
        self.db_manager
            .get_pool()
            .ok_or_else(|| "Database not connected".into())
    }

    pub async fn get_next_version(&self) -> Result<i64, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!("SELECT nextval('{}')", T::sequence_name());
        let rows = client.query(&sql, &[]).await?;

        if let Some(row) = rows.first() {
            Ok(row.get(0))
        } else {
            Err("Failed to get next version".into())
        }
    }

    pub async fn get_max_version(&self) -> Result<i64, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!("SELECT COALESCE(MAX(version), 0) FROM {}", T::table_name());
        let rows = client.query(&sql, &[]).await?;

        if let Some(row) = rows.first() {
            Ok(row.get(0))
        } else {
            Ok(0)
        }
    }

    pub async fn load_accounts_since_version(
        &self,
        since_version: i64,
    ) -> Result<Vec<T>, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!(
            "SELECT {} FROM {} WHERE version > $1 AND deleted IS NOT TRUE ORDER BY version",
            M::select_columns(),
            T::table_name()
        );
        let rows = client.query(&sql, &[&since_version]).await?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(M::from_row(&row)?);
        }
        Ok(accounts)
    }

    pub async fn load_tombstones_since_version(
        &self,
        since_version: i64,
    ) -> Result<Vec<String>, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!(
            "SELECT id FROM {} WHERE deleted IS TRUE AND version > $1",
            T::table_name()
        );
        let rows = client.query(&sql, &[&since_version]).await?;

        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    pub async fn save_account_with_version(&self, account: &T) -> Result<i64, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let new_version = self.get_next_version().await?;

        let params = M::to_params(account, new_version);
        let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

        client.execute(M::insert_sql(), &params_refs).await?;
        Ok(new_version)
    }

    pub async fn delete_account_with_tombstone(
        &self,
        account_id: &str,
    ) -> Result<i64, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let new_version = self.get_next_version().await?;

        let sql = format!(
            "UPDATE {} SET deleted = TRUE, version = $2, updated_at = $3 WHERE id = $1",
            T::table_name()
        );
        let rows_affected = client
            .execute(
                &sql,
                &[&account_id, &new_version, &chrono::Utc::now().timestamp()],
            )
            .await?;

        if rows_affected == 0 {
            Ok(0)
        } else {
            Ok(new_version)
        }
    }
}

#[async_trait::async_trait]
impl<T, M> AccountStorage<T> for GenericPostgreSQLStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    async fn save_account(&self, account: &T) -> Result<(), StorageError> {
        self.save_account_with_version(account).await?;
        Ok(())
    }

    async fn load_accounts(&self) -> Result<Vec<T>, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!(
            "SELECT {} FROM {} WHERE deleted IS NOT TRUE ORDER BY created_at DESC",
            M::select_columns(),
            T::table_name()
        );
        let rows = client.query(&sql, &[]).await?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(M::from_row(&row)?);
        }
        Ok(accounts)
    }

    async fn get_account(&self, account_id: &str) -> Result<Option<T>, StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!(
            "SELECT {} FROM {} WHERE id = $1 AND deleted IS NOT TRUE",
            M::select_columns(),
            T::table_name()
        );
        let rows = client.query(&sql, &[&account_id]).await?;

        if let Some(row) = rows.first() {
            Ok(Some(M::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    async fn update_account(&self, account: &T) -> Result<(), StorageError> {
        self.save_account(account).await
    }

    async fn delete_account(&self, account_id: &str) -> Result<bool, StorageError> {
        let version = self.delete_account_with_tombstone(account_id).await?;
        Ok(version > 0)
    }

    async fn clear_all_accounts(&self) -> Result<(), StorageError> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let sql = format!("DELETE FROM {}", T::table_name());
        client.execute(&sql, &[]).await?;
        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        match T::platform_name() {
            "antigravity" => "antigravity_postgres",
            "windsurf" => "windsurf_postgres",
            _ => "generic_postgres",
        }
    }

    async fn is_available(&self) -> bool {
        self.db_manager.is_connected()
    }
}
