use objection::{
    config::{Config, HttpConfig},
    create_server,
};
use s3::creds::Credentials;
use tempdir::TempDir;
use tokio_util::task::AbortOnDropHandle;

/// An ephemeral testing server which binds to a random port and uses a tmp
/// directory for object storage.
struct TestServer {
    /// Used to configure the S3 request region
    region: s3::Region,
    /// Temporary directory used for object storage
    _data_directory: TempDir,
    /// Handle to this server
    _handle: AbortOnDropHandle<()>,
}

async fn create_test_server() -> TestServer {
    let data_directory =
        tempdir::TempDir::new("objection-testing").expect("Failed to create temporary directory");

    let config = Config {
        data_directory: data_directory.path().to_owned(),
        http: HttpConfig::random_port(),
        ..Default::default()
    };

    let (addr, join_handle) = create_server(config).await;

    TestServer {
        region: s3::Region::Custom {
            region: "us-east-1".into(),
            endpoint: format!("http://127.0.0.1:{}", addr.port()),
        },
        _data_directory: data_directory,
        _handle: AbortOnDropHandle::new(join_handle),
    }
}

#[tokio::test]
pub async fn list_buckets_anonymous_empty() {
    let server = create_test_server().await;

    let buckets = s3::Bucket::list_buckets(server.region, Credentials::anonymous().unwrap())
        .await
        .unwrap();

    assert_eq!(buckets.buckets.bucket.len(), 0);
}
