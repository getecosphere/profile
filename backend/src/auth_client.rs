use serde::Deserialize;

/// Mirrors auth's UserDto shape exactly (see rwid/auth's src/dto/mod.rs).
/// auth is the only writer of these fields; profile only ever reads them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub cover_photo_url: Option<String>,
    pub role: String,
}

#[derive(Clone)]
pub struct AuthClient {
    http: reqwest::Client,
    base_url: String,
}

impl AuthClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn fetch_by_id(&self, user_id: &str) -> Option<Identity> {
        self.fetch(&format!("{}/auth/users/{}", self.base_url.trim_end_matches('/'), user_id))
            .await
    }

    pub async fn fetch_by_username(&self, username: &str) -> Option<Identity> {
        self.fetch(&format!(
            "{}/auth/users/username/{}",
            self.base_url.trim_end_matches('/'),
            username
        ))
        .await
    }

    async fn fetch(&self, url: &str) -> Option<Identity> {
        let mut request = self.http.get(url);
        if let Some(request_id) = crate::request_id::current() {
            request = request.header(crate::request_id::HEADER_NAME, request_id);
        }
        match request.send().await {
            Ok(response) if response.status().is_success() => response.json::<Identity>().await.ok(),
            Ok(response) if response.status() == reqwest::StatusCode::NOT_FOUND => None,
            Ok(response) => {
                tracing::warn!(url, status = %response.status(), "unexpected response from auth service");
                None
            }
            Err(err) => {
                tracing::error!(url, error = %err, "failed to reach auth service");
                None
            }
        }
    }
}
