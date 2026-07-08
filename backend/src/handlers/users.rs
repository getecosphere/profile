use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    auth_extractor::AuthUser,
    date_parse::parse_date,
    dto::{
        AddSkillQuery, CertificationRequest, EducationRequest, ExperienceRequest, SearchQuery,
        SocialLinksRequest, UpdateUserRequest, UserDto,
    },
    error::AppResult,
    models::user::{Certification, Education, Experience},
    repo,
    state::AppState,
};

const PROFILE_ROLES: &[&str] = &["OWNER", "MENTOR", "MEMBER"];

pub async fn get_all_users(State(state): State<AppState>) -> AppResult<Json<Vec<UserDto>>> {
    let users = repo::users::find_all(&state).await?;
    Ok(Json(users.iter().map(UserDto::from).collect()))
}

pub async fn get_user_by_username(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> AppResult<Json<UserDto>> {
    let user = repo::users::sync_from_auth_by_username(&state, &username).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> AppResult<Json<UserDto>> {
    let user = repo::users::sync_from_auth(&state, &user_id).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(name) = req.name {
        user.name = name;
    }
    if req.headline.is_some() {
        user.headline = req.headline;
    }
    if req.bio.is_some() {
        user.bio = req.bio;
    }
    if req.location.is_some() {
        user.location = req.location;
    }
    if req.website.is_some() {
        user.website = req.website;
    }
    if req.interests.is_some() {
        user.interests = req.interests;
    }
    if req.whatsapp_number.is_some() {
        user.whatsapp_number = req.whatsapp_number;
    }
    if req.province.is_some() {
        user.province = req.province;
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn search_users(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<Vec<UserDto>>> {
    let users = repo::users::search_by_name_in_platform(&state, &q.platform_id, &q.query).await?;
    Ok(Json(users.iter().map(UserDto::from).collect()))
}

pub async fn get_users_by_platform(
    State(state): State<AppState>,
    Path(platform_id): Path<String>,
) -> AppResult<Json<Vec<UserDto>>> {
    let users = repo::users::find_by_platform_id(&state, &platform_id).await?;
    Ok(Json(users.iter().map(UserDto::from).collect()))
}

pub async fn add_experience(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Json(req): Json<ExperienceRequest>,
) -> AppResult<(StatusCode, Json<UserDto>)> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    let experience = Experience {
        id: Uuid::new_v4().to_string(),
        title: req.title,
        company: req.company,
        location: req.location,
        description: req.description,
        start_date: parse_date(&req.start_date),
        end_date: parse_date(&req.end_date),
        currently_working: Some(req.currently_working.unwrap_or(false)),
    };
    user.experiences.get_or_insert_with(Vec::new).push(experience);
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok((StatusCode::CREATED, Json(UserDto::from(&user))))
}

pub async fn update_experience(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, experience_id)): Path<(String, String)>,
    Json(req): Json<ExperienceRequest>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(experiences) = user.experiences.as_mut() {
        if let Some(e) = experiences.iter_mut().find(|e| e.id == experience_id) {
            if req.title.is_some() {
                e.title = req.title;
            }
            if req.company.is_some() {
                e.company = req.company;
            }
            if req.location.is_some() {
                e.location = req.location;
            }
            if req.description.is_some() {
                e.description = req.description;
            }
            if req.start_date.is_some() {
                e.start_date = parse_date(&req.start_date);
            }
            if req.end_date.is_some() {
                e.end_date = parse_date(&req.end_date);
            }
            if req.currently_working.is_some() {
                e.currently_working = req.currently_working;
            }
        }
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn delete_experience(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, experience_id)): Path<(String, String)>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(experiences) = user.experiences.as_mut() {
        experiences.retain(|e| e.id != experience_id);
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn add_education(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Json(req): Json<EducationRequest>,
) -> AppResult<(StatusCode, Json<UserDto>)> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    let education = Education {
        id: Uuid::new_v4().to_string(),
        school: req.school,
        degree: req.degree,
        field_of_study: req.field_of_study,
        start_date: parse_date(&req.start_date),
        end_date: parse_date(&req.end_date),
        description: req.description,
    };
    user.education.get_or_insert_with(Vec::new).push(education);
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok((StatusCode::CREATED, Json(UserDto::from(&user))))
}

pub async fn update_education(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, education_id)): Path<(String, String)>,
    Json(req): Json<EducationRequest>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(education) = user.education.as_mut() {
        if let Some(e) = education.iter_mut().find(|e| e.id == education_id) {
            if req.school.is_some() {
                e.school = req.school;
            }
            if req.degree.is_some() {
                e.degree = req.degree;
            }
            if req.field_of_study.is_some() {
                e.field_of_study = req.field_of_study;
            }
            if req.start_date.is_some() {
                e.start_date = parse_date(&req.start_date);
            }
            if req.end_date.is_some() {
                e.end_date = parse_date(&req.end_date);
            }
            if req.description.is_some() {
                e.description = req.description;
            }
        }
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn delete_education(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, education_id)): Path<(String, String)>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(education) = user.education.as_mut() {
        education.retain(|e| e.id != education_id);
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn add_skill(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Query(q): Query<AddSkillQuery>,
) -> AppResult<(StatusCode, Json<UserDto>)> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    let skills = user.skills.get_or_insert_with(Vec::new);
    if !skills.contains(&q.skill) {
        skills.push(q.skill);
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok((StatusCode::CREATED, Json(UserDto::from(&user))))
}

pub async fn remove_skill(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, skill)): Path<(String, String)>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(skills) = user.skills.as_mut() {
        skills.retain(|s| s != &skill);
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn add_certification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Json(req): Json<CertificationRequest>,
) -> AppResult<(StatusCode, Json<UserDto>)> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    let certification = Certification {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        issuer: req.issuer,
        issue_date: parse_date(&req.issue_date),
        expiration_date: parse_date(&req.expiration_date),
        credential_url: req.credential_url,
    };
    user.certifications.get_or_insert_with(Vec::new).push(certification);
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok((StatusCode::CREATED, Json(UserDto::from(&user))))
}

pub async fn update_certification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, certification_id)): Path<(String, String)>,
    Json(req): Json<CertificationRequest>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(certifications) = user.certifications.as_mut() {
        if let Some(c) = certifications.iter_mut().find(|c| c.id == certification_id) {
            if req.name.is_some() {
                c.name = req.name;
            }
            if req.issuer.is_some() {
                c.issuer = req.issuer;
            }
            if req.issue_date.is_some() {
                c.issue_date = parse_date(&req.issue_date);
            }
            if req.expiration_date.is_some() {
                c.expiration_date = parse_date(&req.expiration_date);
            }
            if req.credential_url.is_some() {
                c.credential_url = req.credential_url;
            }
        }
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn delete_certification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((user_id, certification_id)): Path<(String, String)>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    if let Some(certifications) = user.certifications.as_mut() {
        certifications.retain(|c| c.id != certification_id);
    }
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}

pub async fn update_social_links(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<String>,
    Json(req): Json<SocialLinksRequest>,
) -> AppResult<Json<UserDto>> {
    auth.require_role(PROFILE_ROLES)?;
    let mut user = repo::users::require_entity_by_id(&state, &user_id).await?;

    let mut links = user.social_links.unwrap_or_default();
    if req.linkedin.is_some() {
        links.linkedin = req.linkedin;
    }
    if req.twitter.is_some() {
        links.twitter = req.twitter;
    }
    if req.github.is_some() {
        links.github = req.github;
    }
    if req.portfolio.is_some() {
        links.portfolio = req.portfolio;
    }
    user.social_links = Some(links);
    user.updated_at = bson::DateTime::now();

    repo::users::save(&state, &user).await?;
    Ok(Json(UserDto::from(&user)))
}
