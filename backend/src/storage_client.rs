use crate::error::{AppError, AppResult};
use reqwest::multipart::{Form, Part};

/// Result of proxying a file upload to the storage LXS.
#[derive(Debug, Clone)]
pub struct StoredUpload {
    pub key: String,
    pub content_url: String,
}

/// Thin client for the storage LXS (photos/storage domain). Profile is the
/// *writer* of avatar/cover URLs; it proxies the bytes to storage and stores
/// the resulting content URL on the local user row. This keeps image
/// processing, S3 and file serving entirely inside the storage domain.
#[derive(Clone)]
pub struct StorageClient {
    http: reqwest::Client,
    base_url: String,
}

impl StorageClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
        }
    }

    fn available(&self) -> bool {
        !self.base_url.is_empty()
    }

    fn objects_url(&self) -> String {
        format!("{}/storage/objects", self.base_url.trim_end_matches('/'))
    }

    fn content_url_for(&self, key: &str) -> String {
        format!("{}/storage/content/{}", self.base_url.trim_end_matches('/'), key)
    }

    /// Upload a file to the storage LXS under namespace/reference_id, return
    /// the object key and a public content URL.
    pub async fn upload(
        &self,
        owner_id: &str,
        namespace: &str,
        reference_id: &str,
        file_name: Option<&str>,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> AppResult<StoredUpload> {
        if !self.available() {
            return Err(AppError::ServiceUnavailable(
                "Avatar upload requires the storage LXS (STORAGE_BASE_URL is not set).".to_string(),
            ));
        }
        let mut part = Part::bytes(bytes)
            .file_name(file_name.unwrap_or("upload").to_string())
            .mime_str(content_type.unwrap_or("application/octet-stream"))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid part mime: {e}")))?;
        // mime_str needs a valid MIME; fall back if the caller passed garbage.
        if content_type.is_none() {
            part = part.mime_str("application/octet-stream")
                .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid part mime: {e}")))?;
        }
        let form = Form::new()
            .text("owner_id", owner_id.to_string())
            .text("namespace", namespace.to_string())
            .text("reference_id", reference_id.to_string())
            .part("file", part);

        let resp = self
            .http
            .post(self.objects_url())
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage upload request failed: {e}")))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "storage upload failed (HTTP {}): {}",
                status,
                text.chars().take(200).collect::<String>()
            )));
        }
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage upload bad response: {e}")))?;
        let key = parsed
            .get("key")
            .and_then(|k| k.as_str())
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("storage upload missing key")))?;
        Ok(StoredUpload {
            key: key.to_string(),
            content_url: self.content_url_for(key),
        })
    }

    /// Delete an object from storage (owner-scoped). Non-fatal on 404.
    pub async fn delete(&self, owner_id: &str, key: &str) -> AppResult<()> {
        if !self.available() || key.is_empty() {
            return Ok(());
        }
        let url = format!("{}/storage/objects/{}", self.base_url.trim_end_matches('/'), key);
        let resp = self
            .http
            .delete(&url)
            .query(&[("owner_id", owner_id)])
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("storage delete request failed: {e}")))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        if !resp.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "storage delete failed (HTTP {})",
                resp.status()
            )));
        }
        Ok(())
    }
}
