use serde::Serialize;
use sqlite::{Row, Value};
use uuid::Uuid;

use crate::Database;

use super::CachePolicy;

#[derive(Debug)]
pub struct Bucket {
    uuid: Uuid,
    name: Box<str>,
    settings: BucketSettings,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BucketSettings {
    pub default_cache_policy: Option<CachePolicy>,
    pub access_logging: bool,
}

impl Bucket {
    pub async fn create_table(db: &Database) -> sqlite::Result<()> {
        tokio::task::block_in_place(|| {
            db.execute(indoc::indoc! {"
                CREATE TABLE IF NOT EXISTS buckets (
                    uuid TEXT PRIMARY KEY UNIQUE NOT NULL,
                    name TEXT UNIQUE NOT NULL,
                    `settings.default_cache_policy` TEXT,
                    `settings.access_logging` BOOLEAN NOT NULL
                )
            "})
        })
    }

    pub async fn new(
        db: &Database,
        name: impl Into<String>,
        settings: BucketSettings,
    ) -> sqlite::Result<Self> {
        let name: String = name.into();

        let bucket = Self {
            uuid: Uuid::new_v4(),
            name: name.into(),
            settings,
        };

        tokio::task::block_in_place(|| {
            let mut stmt = db.prepare(indoc::indoc! {"
                INSERT INTO buckets VALUES (?, ?, ?, ?);
            "})?;

            stmt.bind::<&[(_, Value)]>(&[
                (1, bucket.uuid().to_string().into()),
                (2, bucket.name().into()),
                (
                    3,
                    bucket
                        .settings
                        .default_cache_policy
                        .map_or(Value::Null, |cp| cp.to_string().into()),
                ),
                (4, (bucket.settings.access_logging as i64).into()),
            ])?;

            stmt.next()?;

            Ok(bucket)
        })
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

    pub async fn find_all(db: &Database) -> sqlite::Result<Vec<Self>> {
        tokio::task::block_in_place(|| {
            db.prepare("SELECT * FROM buckets;")
                .unwrap()
                .into_iter()
                .map(|row| row.map(Into::into))
                .collect::<Result<Vec<_>, _>>()
        })
    }

    pub async fn find_by_name(db: &Database, name: &str) -> Result<Option<Self>, ()> {
        todo!()
    }

    pub async fn find_by_uuid(db: &Database, uuid: Uuid) -> Result<Option<Self>, ()> {
        todo!()
    }

    pub async fn update_settings(&mut self, db: &Database, settings: BucketSettings) {
        todo!()
    }

    pub async fn delete(self, db: &Database) {}

    pub async fn export_backup(&self) -> BucketBackup {
        todo!()
    }

    pub async fn import_backup(&self, backup: BucketBackup) {
        todo!()
    }
}

impl From<Row> for Bucket {
    fn from(row: Row) -> Self {
        Self {
            uuid: row
                .read::<&str, _>("uuid")
                .parse()
                .expect("Failed to parse uuid"),
            name: row.read::<&str, _>("name").into(),
            settings: BucketSettings {
                default_cache_policy: row
                    .read::<Option<&str>, _>("settings.default_cache_policy")
                    .map(|v| {
                        v.parse()
                            .expect("Failed to parse settings.default_cache_policy")
                    }),
                access_logging: row.read::<i64, _>("settings.access_logging") == 1,
            },
        }
    }
}

pub struct BucketBackup {}
