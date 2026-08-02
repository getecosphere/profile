mod common;

use serde_json::{json, Value};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

async fn mock_identity(auth: &MockServer, user_id: &str, name: &str, avatar_url: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/auth/users/{user_id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": user_id,
            "name": name,
            "username": format!("user-{user_id}"),
            "email": format!("{user_id}@example.com"),
            "avatarUrl": avatar_url,
            "coverPhotoUrl": null,
            "role": "member",
        })))
        .mount(auth)
        .await;
}

/// The first time this service is asked about a user it has no local row
/// for, it should hydrate one from auth rather than 404ing.
#[tokio::test]
async fn lazy_hydrates_a_user_never_seen_before() {
    let auth = MockServer::start().await;
    mock_identity(
        &auth,
        "c9f95b04ca88de84b41374c5",
        "Alice",
        "https://cdn.example.com/alice-v1.png",
    )
    .await;
    let app = common::spawn(&auth.uri()).await;

    let res = app
        .http
        .get(app.url("/users/c9f95b04ca88de84b41374c5"))
        .send()
        .await
        .expect("get user request");
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.expect("user body");
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["avatarUrl"], "https://cdn.example.com/alice-v1.png");
}

/// A user auth doesn't know about either (never registered, or a typo'd
/// id) should 404, not silently create a placeholder row.
#[tokio::test]
async fn unknown_user_is_not_hydrated() {
    let auth = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/auth/users/nonexistent"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&auth)
        .await;
    let app = common::spawn(&auth.uri()).await;

    let res = app
        .http
        .get(app.url("/users/nonexistent"))
        .send()
        .await
        .expect("get user request");
    assert_eq!(res.status(), 404);
}

/// Every GET refreshes from auth unconditionally, so avatar/name changes
/// made through auth show up immediately on the next read here -- not
/// just on first contact.
#[tokio::test]
async fn every_read_refreshes_identity_from_auth() {
    let auth = MockServer::start().await;
    mock_identity(
        &auth,
        "0c1f56722f454f263fba645d",
        "Bob",
        "https://cdn.example.com/bob-v1.png",
    )
    .await;
    let app = common::spawn(&auth.uri()).await;

    let first: Value = app
        .http
        .get(app.url("/users/0c1f56722f454f263fba645d"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first["avatarUrl"], "https://cdn.example.com/bob-v1.png");

    // auth's identity changes (new avatar uploaded there).
    auth.reset().await;
    mock_identity(
        &auth,
        "0c1f56722f454f263fba645d",
        "Bob",
        "https://cdn.example.com/bob-v2.png",
    )
    .await;

    let second: Value = app
        .http
        .get(app.url("/users/0c1f56722f454f263fba645d"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        second["avatarUrl"], "https://cdn.example.com/bob-v2.png",
        "profile should reflect the latest auth identity, not a stale cache"
    );
}

#[tokio::test]
async fn updating_a_profile_requires_authentication() {
    let auth = MockServer::start().await;
    mock_identity(
        &auth,
        "783ca098cbc4bda85af30e11",
        "Carol",
        "https://cdn.example.com/carol.png",
    )
    .await;
    let app = common::spawn(&auth.uri()).await;

    // Hydrate first via a GET so the row exists.
    app.http
        .get(app.url("/users/783ca098cbc4bda85af30e11"))
        .send()
        .await
        .unwrap();

    let unauthenticated = app
        .http
        .put(app.url("/users/783ca098cbc4bda85af30e11"))
        .json(&json!({ "bio": "Hacked bio" }))
        .send()
        .await
        .expect("update user request");
    assert_eq!(unauthenticated.status(), 401);

    let token = common::mint_token(
        "783ca098cbc4bda85af30e11",
        "user-783ca098cbc4bda85af30e11",
        "MEMBER",
    );
    let authenticated = app
        .http
        .put(app.url("/users/783ca098cbc4bda85af30e11"))
        .bearer_auth(token)
        .json(&json!({ "bio": "Rustacean since 2024" }))
        .send()
        .await
        .expect("update user request");
    assert_eq!(authenticated.status(), 200);
    let updated: Value = authenticated.json().await.unwrap();
    assert_eq!(updated["bio"], "Rustacean since 2024");
    // Identity fields stay auth's, unaffected by the profile edit.
    assert_eq!(updated["name"], "Carol");
}

#[tokio::test]
async fn experience_can_be_added_with_lenient_date_formats() {
    let auth = MockServer::start().await;
    mock_identity(
        &auth,
        "898c918d3eb05a43760dbd2b",
        "Dave",
        "https://cdn.example.com/dave.png",
    )
    .await;
    let app = common::spawn(&auth.uri()).await;
    app.http
        .get(app.url("/users/898c918d3eb05a43760dbd2b"))
        .send()
        .await
        .unwrap();
    let token = common::mint_token(
        "898c918d3eb05a43760dbd2b",
        "user-898c918d3eb05a43760dbd2b",
        "MEMBER",
    );

    let res = app
        .http
        .post(app.url("/users/898c918d3eb05a43760dbd2b/experience"))
        .bearer_auth(&token)
        .json(&json!({
            "title": "Software Engineer",
            "company": "Acme",
            "startDate": "01/03/2022",
            "endDate": "2023-06-15",
        }))
        .send()
        .await
        .expect("add experience request");
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    let experiences = body["experiences"].as_array().expect("experiences array");
    assert_eq!(experiences.len(), 1);
    assert_eq!(experiences[0]["title"], "Software Engineer");
}

#[tokio::test]
async fn school_taxonomy_add_and_list() {
    let auth = MockServer::start().await;
    let app = common::spawn(&auth.uri()).await;

    let before: Vec<String> = app
        .http
        .get(app.url("/schools"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!before.iter().any(|s| s == "Institut Teknologi Bandung"));

    let add_res = app
        .http
        .post(app.url("/schools"))
        .query(&[("name", "Institut Teknologi Bandung")])
        .send()
        .await
        .expect("add school request");
    assert_eq!(add_res.status(), 201);

    let after: Vec<String> = app
        .http
        .get(app.url("/schools"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(after.iter().any(|s| s == "Institut Teknologi Bandung"));
}
