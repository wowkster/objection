//! The backing database for Objection uses a single buckets table to store all the bucket definitions and an "objects" table for each bucket

use bucket::Bucket;
use serde::{Deserialize, Serialize};

pub mod bucket;
pub mod object;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::Display,
    Serialize,
    Deserialize,
    sqlx::Type,
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
#[sqlx(rename_all = "lowercase")]
pub enum CachePolicy {
    Cache,
    NoCache,
}
