use bson::{doc, oid::ObjectId};
use mongodb::Collection;

use crate::{error::AppError, models::user::User, state::AppState};

fn collection(state: &AppState) -> Collection<User> {
    state.db.collection("users")
}

pub async fn find_by_id_local(state: &AppState, id: &str) -> Result<Option<User>, AppError> {
    let Ok(oid) = ObjectId::parse_str(id) else {
        return Ok(None);
    };
    Ok(collection(state)
        .find_one(doc! { "_id": oid, "deletedAt": null }, None)
        .await?)
}

/// Pulls the latest username/email/name/role/avatarUrl/coverPhotoUrl from
/// auth and upserts the local profile row. Called on every public profile
/// read so avatar/cover/role changes made through auth show up promptly --
/// there's no push-based sync, this is the only mechanism.
pub async fn sync_from_auth(state: &AppState, user_id: &str) -> Result<User, AppError> {
    let identity = state
        .auth_client
        .fetch_by_id(user_id)
        .await
        .ok_or_else(|| AppError::NotFound(format!("User not found with id: {user_id}")))?;
    upsert_from_identity(state, user_id, identity).await
}

pub async fn sync_from_auth_by_username(state: &AppState, username: &str) -> Result<User, AppError> {
    let identity = state
        .auth_client
        .fetch_by_username(username)
        .await
        .ok_or_else(|| AppError::NotFound(format!("User not found with username: {username}")))?;
    let user_id = identity.id.clone();
    upsert_from_identity(state, &user_id, identity).await
}

async fn upsert_from_identity(
    state: &AppState,
    user_id: &str,
    identity: crate::auth_client::Identity,
) -> Result<User, AppError> {
    let oid = ObjectId::parse_str(user_id)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("auth returned a non-ObjectId user id")))?;
    let now = bson::DateTime::now();

    let existing = collection(state).find_one(doc! { "_id": oid }, None).await?;
    let user = match existing {
        Some(mut user) => {
            user.username = identity.username;
            user.email = identity.email;
            user.name = identity.name;
            user.role = identity.role;
            user.avatar_url = identity.avatar_url;
            user.cover_photo_url = identity.cover_photo_url;
            user.updated_at = now;
            collection(state).replace_one(doc! { "_id": oid }, &user, None).await?;
            user
        }
        None => {
            let user = User {
                id: Some(oid),
                username: identity.username,
                email: identity.email,
                name: identity.name,
                headline: None,
                avatar_url: identity.avatar_url,
                cover_photo_url: identity.cover_photo_url,
                bio: None,
                location: None,
                website: None,
                school: None,
                whatsapp_number: None,
                province: None,
                role: identity.role,
                platform_id: None,
                interests: None,
                experiences: None,
                education: None,
                skills: None,
                certifications: None,
                social_links: None,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            collection(state).insert_one(&user, None).await?;
            user
        }
    };

    Ok(user)
}

/// Local-only lookup for profile-editing operations: a missing row is
/// hydrated from auth once (so editing a profile you've never touched
/// before doesn't 404), but avatar/cover/role freshness isn't re-checked
/// on every edit the way sync_from_auth does for reads.
pub async fn require_entity_by_id(state: &AppState, id: &str) -> Result<User, AppError> {
    match find_by_id_local(state, id).await? {
        Some(user) => Ok(user),
        None => sync_from_auth(state, id).await,
    }
}

pub async fn save(state: &AppState, user: &User) -> Result<(), AppError> {
    let oid = user
        .id
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("user has no id")))?;
    collection(state).replace_one(doc! { "_id": oid }, user, None).await?;
    Ok(())
}

pub async fn find_by_platform_id(state: &AppState, platform_id: &str) -> Result<Vec<User>, AppError> {
    let mut cursor = collection(state)
        .find(doc! { "platformId": platform_id, "deletedAt": null }, None)
        .await?;
    let mut out = Vec::new();
    use futures_util::StreamExt;
    while let Some(user) = cursor.next().await {
        out.push(user?);
    }
    Ok(out)
}

pub async fn search_by_name_in_platform(
    state: &AppState,
    platform_id: &str,
    query: &str,
) -> Result<Vec<User>, AppError> {
    let mut cursor = collection(state)
        .find(
            doc! {
                "platformId": platform_id,
                "name": { "$regex": query, "$options": "i" },
                "deletedAt": null,
            },
            None,
        )
        .await?;
    let mut out = Vec::new();
    use futures_util::StreamExt;
    while let Some(user) = cursor.next().await {
        out.push(user?);
    }
    Ok(out)
}

pub async fn find_all(state: &AppState) -> Result<Vec<User>, AppError> {
    let mut cursor = collection(state).find(doc! { "deletedAt": null }, None).await?;
    let mut out = Vec::new();
    use futures_util::StreamExt;
    while let Some(user) = cursor.next().await {
        out.push(user?);
    }
    Ok(out)
}
