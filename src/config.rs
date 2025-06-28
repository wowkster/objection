use std::{
    collections::{BTreeSet, HashSet},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    time::Duration,
};

use axum::http::{HeaderName, Method};
use clap::{error::ErrorKind, CommandFactory};
use serde::Deserialize;
use url::{Origin, Url};

use crate::{models::CachePolicy, Args};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    http: Option<PartialHttpConfig>,
    tls: Option<PartialTlsConfig>,
    cors: Option<PartialCorsConfig>,
    cache_control: Option<PartialCacheControlConfig>,
    access_control: Option<PartialAccessControlConfig>,
    ip_filter: Option<PartialIpFilterConfig>,
    content_types: Option<PartialContentTypesConfig>,
    rate_limiting: Option<PartialRateLimitingConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialHttpConfig {
    host: Option<Ipv4Addr>,
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialTlsConfig {
    tls_versions: Option<BTreeSet<String>>,
    private_key: Option<String>,
    public_key: Option<String>,
    private_key_file: Option<PathBuf>,
    public_key_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialCorsConfig {
    allow_origins: Option<BTreeSet<String>>,
    allow_methods: Option<BTreeSet<String>>,
    allow_headers: Option<BTreeSet<String>>,
    allow_credentials: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialCacheControlConfig {
    default_policy: Option<CachePolicy>,
    default_max_age: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialAccessControlConfig {
    enable_access_tokens: Option<bool>,
    enable_local_host_auth_bypass: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialIpFilterConfig {
    whitelist: Option<BTreeSet<String>>,
    blacklist: Option<BTreeSet<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialContentTypesConfig {
    whitelist: Option<BTreeSet<String>>,
    blacklist: Option<BTreeSet<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartialRateLimitingConfig {
    default_period: Option<String>,
    default_burst_size: Option<u32>,
}

#[derive(Debug, Default)]
pub struct Config {
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

impl Config {
    pub fn parse_and_validate(path: impl AsRef<Path>) -> Self {
        let mut cmd = Args::command();

        let contents = match std::fs::read_to_string(path.as_ref()) {
            Ok(value) => value,
            Err(e) => cmd
                .error(
                    ErrorKind::Io,
                    format!(
                        "Failed to read configuration file '{}': {}",
                        path.as_ref().display(),
                        e
                    ),
                )
                .exit(),
        };
        let file = match toml::from_str::<ConfigFile>(&contents) {
            Ok(value) => value,
            Err(e) => cmd
                .error(
                    ErrorKind::ValueValidation,
                    format!(
                        "Failed to parse configuration file '{}': {}",
                        path.as_ref().display(),
                        e
                    ),
                )
                .exit(),
        };

        Self::validate_file(file)
    }

    fn validate_file(file: ConfigFile) -> Self {
        let mut cmd = Args::command();

        let http = file
            .http
            .map(|http| HttpConfig {
                host: http.host.unwrap_or_else(|| HttpConfig::default().host),
                port: http.port.unwrap_or_else(|| HttpConfig::default().port),
            })
            .unwrap_or_default();

        let tls = file.tls.map(|tls| {
            let tls_versions = match tls.tls_versions {
                Some(versions) => versions
                    .into_iter()
                    .map(|v| {
                        v.parse::<TlsVersion>().unwrap_or_else(|_| {
                            cmd.error(
                                ErrorKind::ValueValidation,
                                format!("Invalid TLS version '{}'", v),
                            )
                            .exit()
                        })
                    })
                    .collect(),
                None => BTreeSet::from([TlsVersion::V1_1, TlsVersion::V1_2, TlsVersion::V1_3]),
            };

            let keys = match (
                tls.private_key,
                tls.public_key,
                tls.private_key_file,
                tls.public_key_file,
            ) {
                (None, None, Some(private_key_file), Some(public_key_file)) => 
                    TlsKeyConfig::File { private_key_file, public_key_file },
                (Some(private_key), Some(public_key), None, None) => 
                    TlsKeyConfig::String { private_key, public_key },
                _ => cmd
                    .error(
                        ErrorKind::ValueValidation,
                        "Invalid TLS configuration. Must specify either 'private-key' and 'public-key' or 'private-key-file' and 'public-key-file'",
                    )
                    .exit(),
            };

            TlsConfig {
                tls_versions,
                keys,
            }
        });

        let cors = file.cors.map(|cors| CorsConfig {
            allow_origins: cors
                .allow_origins
                .map(|origins| {
                    origins
                        .into_iter()
                        .map(|o| {
                            o.parse::<Url>()
                                .unwrap_or_else(|_| {
                                    cmd.error(
                                        ErrorKind::ValueValidation,
                                        format!("Invalid CORS origin '{}'", o),
                                    )
                                    .exit()
                                })
                                .origin()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            allow_methods: cors
                .allow_methods
                .map(|methods| {
                    methods
                        .into_iter()
                        .map(|m| {
                            m.parse().unwrap_or_else(|_| {
                                cmd.error(
                                    ErrorKind::ValueValidation,
                                    format!("Invalid CORS HTTP method '{}'", m),
                                )
                                .exit()
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            allow_headers: cors
                .allow_headers
                .map(|headers| {
                    headers
                        .into_iter()
                        .map(|h| {
                            h.parse().unwrap_or_else(|_| {
                                cmd.error(
                                    ErrorKind::ValueValidation,
                                    format!("Invalid CORS HTTP header '{}'", h),
                                )
                                .exit()
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            allow_credentials: cors.allow_credentials.unwrap_or_default(),
        });

        let cache_control = file
            .cache_control
            .map(|_| todo!("Validate cache control config"))
            .unwrap_or_default();
        let access_control = file
            .access_control
            .map(|_| todo!("Validate access control config"))
            .unwrap_or_default();
        let ip_filter = file
            .ip_filter
            .map(|_| todo!("Validate ip filter config"))
            .unwrap_or_default();
        let content_types = file
            .content_types
            .map(|_| todo!("Validate content type filter config"))
            .unwrap_or_default();
        let rate_limiting = file
            .rate_limiting
            .map(|_| todo!("Validate rate limiting config"))
            .unwrap_or_default();

        Self {
            http,
            tls,
            cors,
            cache_control,
            access_control,
            ip_filter,
            content_types,
            rate_limiting,
        }
    }
}
