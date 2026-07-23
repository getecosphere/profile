use std::{sync::Arc, time::Duration};

use axum::{
    http::HeaderValue,
    response::Response,
    routing::{get, post, put},
    Router,
};
use tower::ServiceBuilder;
use tower_governor::{governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::{handlers, state::AppState};

const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

pub fn build_router(state: AppState) -> Router {
    let origins: Vec<_> = state
        .config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
            axum::http::Method::HEAD,
        ]))
        .allow_headers(AllowHeaders::mirror_request())
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    let general_governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(state.config.rate_limit_general_replenish_secs)
            .burst_size(state.config.rate_limit_general_burst)
            .finish()
            .expect("valid governor config"),
    );
    spawn_governor_cleanup(general_governor_config.clone());

    let api_routes = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/users", get(handlers::users::get_all_users))
        .route("/users/search", get(handlers::users::search_users))
        .route(
            "/users/username/:username",
            get(handlers::users::get_user_by_username),
        )
        .route(
            "/users/platform/:platform_id",
            get(handlers::users::get_users_by_platform),
        )
        .route(
            "/users/:user_id",
            get(handlers::users::get_user_by_id).put(handlers::users::update_user),
        )
        .route(
            "/users/:user_id/experience",
            post(handlers::users::add_experience),
        )
        .route(
            "/users/:user_id/experience/:experience_id",
            put(handlers::users::update_experience).delete(handlers::users::delete_experience),
        )
        .route(
            "/users/:user_id/education",
            post(handlers::users::add_education),
        )
        .route(
            "/users/:user_id/education/:education_id",
            put(handlers::users::update_education).delete(handlers::users::delete_education),
        )
        .route("/users/:user_id/skills", post(handlers::users::add_skill))
        .route(
            "/users/:user_id/skills/:skill",
            axum::routing::delete(handlers::users::remove_skill),
        )
        .route(
            "/users/:user_id/certifications",
            post(handlers::users::add_certification),
        )
        .route(
            "/users/:user_id/certifications/:certification_id",
            put(handlers::users::update_certification).delete(handlers::users::delete_certification),
        )
        .route(
            "/users/:user_id/social-links",
            put(handlers::users::update_social_links),
        )
        .route(
            "/schools",
            get(handlers::tags::get_schools).post(handlers::tags::add_school),
        )
        .route(
            "/interests",
            get(handlers::tags::get_interests).post(handlers::tags::add_interest),
        )
        .route(
            "/skills",
            get(handlers::tags::get_skills).post(handlers::tags::add_skill_tag),
        )
        .layer(GovernorLayer {
            config: general_governor_config,
        })
        .layer(
            ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
                .layer(axum::middleware::map_response(security_headers)),
        )
        .with_state(state);

    // Mirrors the estate's other services' `server.servlet.context-path: /api`.
    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(axum::middleware::from_fn(crate::request_id::propagate))
}

/// Same response headers as every other service in the estate.
async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "cache-control",
        HeaderValue::from_static("no-cache, no-store, max-age=0, must-revalidate"),
    );
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

/// The keyed rate-limit store grows one entry per distinct client key seen;
/// without periodic cleanup that's unbounded memory growth from an attacker
/// cycling source IPs.
fn spawn_governor_cleanup(
    config: Arc<tower_governor::governor::GovernorConfig<SmartIpKeyExtractor, governor::middleware::NoOpMiddleware>>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            config.limiter().retain_recent();
        }
    });
}
