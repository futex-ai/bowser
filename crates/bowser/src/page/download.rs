//! Browser-native download helpers.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chromiumoxide::cdp::browser_protocol::browser::{
    SetDownloadBehaviorBehavior, SetDownloadBehaviorParams,
};

use crate::error::{Error, Result};
use crate::model::DownloadResult;

use super::types::LivePage;

const DOWNLOAD_POLL_INTERVAL: Duration = Duration::from_millis(100);

impl LivePage {
    pub(super) async fn download_impl(
        &self,
        url: &str,
        destination: &Path,
    ) -> Result<DownloadResult> {
        self.prepare().await?;
        validate_destination(destination)?;
        let temp_dir = download_temp_dir()?;
        fs::create_dir_all(&temp_dir).map_err(|err| Error::io("create download directory", err))?;
        let result = self.download_to_temp(url, destination, &temp_dir).await;
        let _ = fs::remove_dir_all(&temp_dir);
        result
    }

    async fn download_to_temp(
        &self,
        url: &str,
        destination: &Path,
        temp_dir: &Path,
    ) -> Result<DownloadResult> {
        let params = SetDownloadBehaviorParams::builder()
            .behavior(SetDownloadBehaviorBehavior::Allow)
            .download_path(temp_dir.display().to_string())
            .events_enabled(true)
            .build()
            .map_err(|reason| Error::Download { reason })?;
        self.page
            .execute(params)
            .await
            .map_err(|err| Error::Download {
                reason: format!("failed to configure Chrome download behavior: {err}"),
            })?;
        let url_json = serde_json::to_string(url).map_err(|err| Error::Download {
            reason: format!("failed to encode download URL: {err}"),
        })?;
        let _ = self
            .page
            .evaluate(format!("window.location.href = {url_json};"))
            .await;
        let downloaded = wait_for_download(temp_dir, self.config.timeout).await?;
        let filename = downloaded
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToString::to_string)
            .ok_or_else(|| Error::Download {
                reason: "downloaded file did not have a valid filename".to_string(),
            })?;
        move_download(&downloaded, destination)?;
        let bytes = fs::metadata(destination)
            .map_err(|err| Error::io("read downloaded file metadata", err))?
            .len();
        Ok(DownloadResult {
            path: destination.to_path_buf(),
            filename,
            bytes,
        })
    }
}

fn validate_destination(destination: &Path) -> Result<()> {
    if destination.file_name().is_none() {
        return Err(Error::DownloadDestination {
            path: destination.display().to_string(),
            reason: "path must include a filename".to_string(),
        });
    }
    if destination.is_dir() {
        return Err(Error::DownloadDestination {
            path: destination.display().to_string(),
            reason: "path names a directory".to_string(),
        });
    }
    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(Error::DownloadDestination {
            path: destination.display().to_string(),
            reason: "parent directory does not exist".to_string(),
        });
    }
    Ok(())
}

fn download_temp_dir() -> Result<PathBuf> {
    Ok(std::env::temp_dir()
        .join("bowser-downloads")
        .join(uuid::Uuid::new_v4().to_string()))
}

async fn wait_for_download(temp_dir: &Path, timeout: Duration) -> Result<PathBuf> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(path) = completed_download(temp_dir)? {
            return Ok(path);
        }
        tokio::time::sleep(DOWNLOAD_POLL_INTERVAL).await;
    }
    Err(Error::DownloadTimeout {
        seconds: timeout.as_secs(),
    })
}

fn completed_download(temp_dir: &Path) -> Result<Option<PathBuf>> {
    let entries =
        fs::read_dir(temp_dir).map_err(|err| Error::io("read download directory", err))?;
    let mut candidate = None;
    for entry in entries {
        let entry = entry.map_err(|err| Error::io("read download entry", err))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("crdownload") {
            return Ok(None);
        }
        if entry
            .file_type()
            .map_err(|err| Error::io("read download entry type", err))?
            .is_file()
        {
            candidate = Some(path);
        }
    }
    Ok(candidate)
}

fn move_download(source: &Path, destination: &Path) -> Result<()> {
    if fs::rename(source, destination).is_ok() {
        return Ok(());
    }
    fs::copy(source, destination).map_err(|err| Error::io("copy downloaded file", err))?;
    fs::remove_file(source).map_err(|err| Error::io("remove temporary download file", err))
}
