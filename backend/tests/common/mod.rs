//! Shared integration-test harness. See rwid/auth/backend's copy of this
//! file for the fuller rationale on the real-server/direct-AppConfig
//! pattern. This domain additionally depends on `auth` over HTTP
//! (`AuthClient`), so `spawn` takes a base URL for it -- tests point that
//! at a `wiremock::MockServer` rather than a real auth instance, keeping
//! the test self-contained.
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use profile_service::{config::AppConfig, routes, state::AppState};
use serde::Serialize;
use std::net::SocketAddr;
use uuid::Uuid;

pub const TEST_JWT_SECRET: &str =
    "this-is-a-64-byte-or-longer-test-secret-for-hs512-signing-parity-check!!";

pub struct TestApp {
    pub base_url: String,
    pub http: reqwest::Client,
    pub db: mongodb::Database,
}

impl TestApp {
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        let db = self.db.clone();
        tokio::spawn(async move {
            let _ = db.drop(None).await;
        });
    }
}

pub async fn spawn(auth_base_url: &str) -> TestApp {
    // Best-effort: main.rs normally does this, but tests never call
    // main.rs. Without a subscriber, tracing::warn!/error! calls inside
    // the service silently go nowhere, hiding the real cause behind a
    // bare 500 when something fails.
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let run_id = Uuid::new_v4().simple().to_string();
    let db_name = format!("profile_test_{run_id}");
    let mongodb_uri = format!("mongodb://localhost:27017/{db_name}");

    let client = mongodb::Client::with_uri_str(&mongodb_uri)
        .await
        .expect("connect to local test MongoDB (is `mongod` running on localhost:27017?)");
    let db = client.default_database().expect("db name in URI");

    let config = AppConfig {
        mongodb_uri,
        jwt_secret: TEST_JWT_SECRET.to_string(),
        server_port: 0,
        cors_allowed_origins: vec!["http://localhost:3000".to_string()],
        auth_base_url: auth_base_url.to_string(),
    };

    let state = AppState::new(db.clone(), config);
    let app = routes::build_router(state);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind test listener");
    let addr = listener.local_addr().expect("listener local addr");
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("test server crashed");
    });

    TestApp {
        base_url: format!("http://{addr}/api"),
        http: reqwest::Client::new(),
        db,
    }
}

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    username: String,
    role: String,
    iat: i64,
    exp: i64,
}

/// Mints a JWT with the same HS512/claims shape auth actually issues.
pub fn mint_token(user_id: &str, username: &str, role: &str) -> String {
    let now = chrono::Utc::now();
    let claims = TestClaims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(1)).timestamp(),
    };
    encode(
        &Header::new(Algorithm::HS512),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("encode test jwt")
}
