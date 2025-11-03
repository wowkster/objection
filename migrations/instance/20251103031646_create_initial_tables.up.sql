CREATE TABLE IF NOT EXISTS buckets (
    uuid TEXT PRIMARY KEY UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,

    default_cache_policy TEXT,
    access_logging BOOLEAN NOT NULL
)
