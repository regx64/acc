//! Local cache of test data pulled from object storage.
//!
//! Object keys are never reused (each upload gets a fresh prefix), so a key
//! maps to a fixed file and the cache never goes stale. Files are written to
//! a temp name and renamed into place.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use acc_judge::{Input, TestCase};
use anyhow::Context;
use object_store::path::Path as ObjPath;
use object_store::{ObjectStore, ObjectStoreExt};

pub struct TestcaseCache {
    dir: PathBuf,
    judge_dir: Option<PathBuf>,
    store: Arc<dyn ObjectStore>,
}

impl TestcaseCache {
    pub fn new(dir: PathBuf, judge_dir: Option<PathBuf>, store: Arc<dyn ObjectStore>) -> Self {
        TestcaseCache {
            dir,
            judge_dir,
            store,
        }
    }

    pub async fn get(&self, input_key: &str, output_key: &str) -> anyhow::Result<TestCase> {
        let in_rel = key_path(input_key)?;
        let out_rel = key_path(output_key)?;
        let in_path = self.fetch(&in_rel, input_key).await?;
        let out_path = self.fetch(&out_rel, output_key).await?;

        let expected = tokio::fs::read(&out_path).await?;
        let input = match &self.judge_dir {
            Some(jd) => Input::Path(jd.join(&in_rel).to_string_lossy().into_owned()),
            None => Input::Bytes(tokio::fs::read(&in_path).await?),
        };
        Ok(TestCase { input, expected })
    }

    async fn fetch(&self, rel: &Path, key: &str) -> anyhow::Result<PathBuf> {
        let path = self.dir.join(rel);
        if tokio::fs::try_exists(&path).await? {
            return Ok(path);
        }
        let data = self
            .store
            .get(&ObjPath::from(key))
            .await
            .with_context(|| format!("downloading {key}"))?
            .bytes()
            .await?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let tmp = path.with_extension(format!("tmp{}", std::process::id()));
        tokio::fs::write(&tmp, &data).await?;
        // go-judge reads inputs as its own user.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644)).await?;
        }
        tokio::fs::rename(&tmp, &path).await?;
        Ok(path)
    }
}

/// Relative cache path for an object key, refusing anything that could
/// escape the cache directory.
fn key_path(key: &str) -> anyhow::Result<PathBuf> {
    let ok = !key.is_empty()
        && key
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b));
    anyhow::ensure!(ok, "unsafe object key: {key}");
    Ok(PathBuf::from(key))
}

#[cfg(test)]
mod tests {
    use super::key_path;

    #[test]
    fn keys() {
        assert!(key_path("problems/1000/ab12/1.in").is_ok());
        assert!(key_path("../etc/passwd").is_err());
        assert!(key_path("/abs").is_err());
        assert!(key_path("a//b").is_err());
    }
}
