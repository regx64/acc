//! Object storage for test data, images and backups (R2 in production,
//! MinIO or a local directory in development).

use std::sync::Arc;

use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::ObjectStore;

/// Builds a store from `STORAGE_URL`:
/// - `file:///var/lib/acc/objects` for a local directory
/// - `s3://bucket` for S3-compatible storage; credentials and endpoint come
///   from the usual `AWS_*` variables (`AWS_ENDPOINT`, `AWS_ACCESS_KEY_ID`,
///   `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`, `AWS_ALLOW_HTTP`).
pub fn from_url(url: &str) -> anyhow::Result<Arc<dyn ObjectStore>> {
    if let Some(path) = url.strip_prefix("file://") {
        std::fs::create_dir_all(path)?;
        Ok(Arc::new(LocalFileSystem::new_with_prefix(path)?))
    } else if let Some(bucket) = url.strip_prefix("s3://") {
        let s3 = AmazonS3Builder::from_env()
            .with_bucket_name(bucket)
            .build()?;
        Ok(Arc::new(s3))
    } else {
        anyhow::bail!("unsupported STORAGE_URL: {url}")
    }
}

/// Key of a test file. `batch` is a fresh random id per upload, so keys are
/// never overwritten and caches keyed on them never go stale.
pub fn testcase_key(problem_id: i32, batch: &str, idx: i32, ext: &str) -> String {
    format!("problems/{problem_id}/{batch}/{idx}.{ext}")
}
