use std::{
    collections::{BTreeSet, HashSet},
    net::Ipv4Addr,
    path::PathBuf,
    time::Duration,
};

use axum::http::{HeaderName, Method};
use serde::Deserialize;
use url::Origin;

pub use crate::models::CachePolicy;

#[derive(Debug, Default)]
pub struct Config {
    pub data_directory: PathBuf,
    pub http: HttpConfig,
    pub tls: Option<TlsConfig>,
    pub cors: Option<CorsConfig>,
    pub cache_control: CacheControlConfig,
    pub access_control: AccessControlConfig,
    pub ip_filter: Option<IpFilterConfig>,
    pub content_types: Option<ContentTypesConfig>,
    pub rate_limiting: Option<RateLimitingConfig>,
}

#[derive(Debug)]
pub struct HttpConfig {
    pub host: Ipv4Addr,
    pub port: u16,
}

impl HttpConfig {
    pub fn random_port() -> Self {
        Self {
            host: Ipv4Addr::UNSPECIFIED,
            port: 0,
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: Ipv4Addr::UNSPECIFIED,
            port: 2048,
        }
    }
}

#[derive(Debug)]
pub struct TlsConfig {
    pub tls_versions: BTreeSet<TlsVersion>,
    pub keys: TlsKeyConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumString)]
pub enum TlsVersion {
    #[strum(serialize = "1.1")]
    V1_1,
    #[strum(serialize = "1.2")]
    V1_2,
    #[strum(serialize = "1.3")]
    V1_3,
}

#[derive(Debug)]
pub enum TlsKeyConfig {
    String {
        private_key: String,
        public_key: String,
    },
    File {
        private_key_file: PathBuf,
        public_key_file: PathBuf,
    },
}

#[derive(Debug)]
pub struct CorsConfig {
    pub allow_origins: HashSet<Origin>,
    pub allow_methods: HashSet<Method>,
    pub allow_headers: HashSet<HeaderName>,
    pub allow_credentials: bool,
}

#[derive(Debug)]
pub struct CacheControlConfig {
    pub default_policy: CachePolicy,
    pub default_max_age: u64,
}

impl Default for CacheControlConfig {
    fn default() -> Self {
        Self {
            default_policy: CachePolicy::NoCache,
            default_max_age: 3_600,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AccessControlConfig {
    pub enable_access_tokens: bool,
    pub enable_local_host_auth_bypass: bool,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        Self {
            enable_access_tokens: true,
            enable_local_host_auth_bypass: false,
        }
    }
}

#[derive(Debug)]
pub enum IpFilterConfig {
    Whitelist(BTreeSet<cidr::IpCidr>),
    Blacklist(BTreeSet<cidr::IpCidr>),
}

#[derive(Debug)]
pub enum ContentTypesConfig {
    Whitelist(BTreeSet<mime::Mime>),
    Blacklist(BTreeSet<mime::Mime>),
}

#[derive(Debug, Deserialize)]
pub struct RateLimitingConfig {
    pub default_period: Duration,
    pub default_burst_size: u32,
}
