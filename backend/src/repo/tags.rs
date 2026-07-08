use bson::{doc, Document};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, state::AppState};

/// Schools, interests, and skills are all the same shape in the Java
/// version (a bare name-list taxonomy: GET all sorted, POST add-if-missing)
/// -- one generic implementation instead of three copies.
#[derive(Debug, Serialize, Deserialize)]
struct Tag {
    name: String,
    #[serde(rename = "createdAt")]
    created_at: bson::DateTime,
    #[serde(rename = "updatedAt")]
    updated_at: bson::DateTime,
}

pub async fn get_all_sorted(state: &AppState, collection_name: &str) -> Result<Vec<String>, AppError> {
    let collection: mongodb::Collection<Document> = state.db.collection(collection_name);
    let mut cursor = collection.find(doc! {}, None).await?;
    let mut names = Vec::new();
    use futures_util::StreamExt;
    while let Some(doc) = cursor.next().await {
        if let Ok(name) = doc?.get_str("name") {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

pub async fn add_if_missing(state: &AppState, collection_name: &str, name: &str) -> Result<(), AppError> {
    let collection: mongodb::Collection<Tag> = state.db.collection(collection_name);
    if collection.find_one(doc! { "name": name }, None).await?.is_some() {
        return Ok(());
    }
    let now = bson::DateTime::now();
    collection
        .insert_one(
            &Tag {
                name: name.to_string(),
                created_at: now,
                updated_at: now,
            },
            None,
        )
        .await?;
    Ok(())
}
