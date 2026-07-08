use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use crate::{dto::AddTagQuery, error::AppResult, repo, state::AppState};

pub async fn get_schools(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    Ok(Json(repo::tags::get_all_sorted(&state, "schools").await?))
}

pub async fn add_school(
    State(state): State<AppState>,
    Query(q): Query<AddTagQuery>,
) -> AppResult<(StatusCode, String)> {
    repo::tags::add_if_missing(&state, "schools", &q.name).await?;
    Ok((StatusCode::CREATED, q.name))
}

pub async fn get_interests(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    Ok(Json(repo::tags::get_all_sorted(&state, "interests").await?))
}

pub async fn add_interest(
    State(state): State<AppState>,
    Query(q): Query<AddTagQuery>,
) -> AppResult<(StatusCode, String)> {
    repo::tags::add_if_missing(&state, "interests", &q.name).await?;
    Ok((StatusCode::CREATED, q.name))
}

pub async fn get_skills(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    Ok(Json(repo::tags::get_all_sorted(&state, "skills").await?))
}

pub async fn add_skill_tag(
    State(state): State<AppState>,
    Query(q): Query<AddTagQuery>,
) -> AppResult<(StatusCode, String)> {
    repo::tags::add_if_missing(&state, "skills", &q.name).await?;
    Ok((StatusCode::CREATED, q.name))
}
