use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use mime::Mime;
use uuid::Uuid;

use super::CachePolicy;

pub struct Object {
    bucket: Uuid,
    hash: Box<str>,
    path: Box<str>,
    expires_at: Option<DateTime<Utc>>,
    content_type: Option<Mime>,
    cache_policy: CachePolicy,
    tags: BTreeSet<Box<str>>,
}

// objection.buckets
// objection.objects_40823429834283235245328734223434
