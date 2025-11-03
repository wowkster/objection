use std::collections::BTreeSet;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use objection::{
    config::{CachePolicy, Config, CorsConfig, HttpConfig, TlsConfig, TlsKeyConfig, TlsVersion},
    create_server,
};
use serde::Deserialize;
use tracing_subscriber::EnvFilter;
use url::Url;

#[derive(Debug, clap::Parser)]
pub struct Args {
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    /* Initialize Config */

    let args = Args::parse();

    let config = args.config_path.map(parse_and_validate).unwrap_or_default();

    tracing::debug!("using config: {:#?}", config);

    let (_, handle) = create_server(config).await;

    tokio::select! {
        r = handle => r.expect("failed to serve"),
        _ =  tokio::signal::ctrl_c() => {}
    }

    Ok(())
}

pub fn parse_and_validate(path: impl AsRef<Path>) -> Config {
    use clap::error::ErrorKind;

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

    validate_file(file)
}

fn validate_file(file: ConfigFile) -> Config {
    let mut cmd = Args::command();

    let data_directory =
        Path::new(&file.data_directory.expect("missing data directory")).to_path_buf();

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

    Config {
        data_directory,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    data_directory: Option<String>,
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
