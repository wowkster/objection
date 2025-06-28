//! The backing database for Objection uses a single buckets table to store all the bucket definitions and an "objects" table for each bucket

use bucket::Bucket;
use serde::{Deserialize, Serialize};

use crate::Database;

pub mod bucket;
pub mod object;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::EnumString, strum::Display, Serialize, Deserialize,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum CachePolicy {
    Cache,
    NoCache,
}

pub async fn create_tables(db: &Database) -> sqlite::Result<()> {
    Bucket::create_table(db).await?;

    Ok(())
}
