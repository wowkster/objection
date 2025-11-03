use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::CachePolicy;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bucket {
    uuid: Uuid,
    name: Box<str>,
    #[serde(flatten)]
    #[sqlx(flatten)]
    settings: BucketSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, FromRow)]
pub struct BucketSettings {
    pub default_cache_policy: Option<CachePolicy>,
    pub access_logging: bool,
}

impl Bucket {
    pub async fn new(
        db: &sqlx::SqlitePool,
        name: impl Into<String>,
        settings: BucketSettings,
    ) -> sqlx::Result<Self> {
        let name: String = name.into();

        let bucket: Bucket = sqlx::query_as("INSERT INTO buckets VALUES (?, ?, ?, ?);")
            .bind(Uuid::new_v4())
            .bind(name)
            .bind(settings.default_cache_policy)
            .bind(settings.access_logging)
            .fetch_one(db)
            .await?;

        Ok(bucket)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn settings(&self) -> &BucketSettings {
        &self.settings
    }

    pub async fn find_all(db: &sqlx::SqlitePool) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as("SELECT * FROM buckets;").fetch_all(db).await
    }

    pub async fn find_by_name(db: &sqlx::SqlitePool, name: &str) -> Result<Option<Self>, ()> {
        todo!()
    }

    pub async fn find_by_uuid(db: &sqlx::SqlitePool, uuid: Uuid) -> Result<Option<Self>, ()> {
        todo!()
    }

    pub async fn update_settings(&mut self, db: &sqlx::SqlitePool, settings: BucketSettings) {
        todo!()
    }

    pub async fn delete(self, db: &sqlx::SqlitePool) {}

    pub async fn export_backup(&self) -> BucketBackup {
        todo!()
    }

    pub async fn import_backup(&self, backup: BucketBackup) {
        todo!()
    }
}

pub struct BucketBackup {}
