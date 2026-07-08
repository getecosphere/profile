use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "startDate", skip_serializing_if = "Option::is_none")]
    pub start_date: Option<bson::DateTime>,
    #[serde(rename = "endDate", skip_serializing_if = "Option::is_none")]
    pub end_date: Option<bson::DateTime>,
    #[serde(rename = "currentlyWorking", skip_serializing_if = "Option::is_none")]
    pub currently_working: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub school: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degree: Option<String>,
    #[serde(rename = "fieldOfStudy", skip_serializing_if = "Option::is_none")]
    pub field_of_study: Option<String>,
    #[serde(rename = "startDate", skip_serializing_if = "Option::is_none")]
    pub start_date: Option<bson::DateTime>,
    #[serde(rename = "endDate", skip_serializing_if = "Option::is_none")]
    pub end_date: Option<bson::DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(rename = "issueDate", skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<bson::DateTime>,
    #[serde(rename = "expirationDate", skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<bson::DateTime>,
    #[serde(rename = "credentialUrl", skip_serializing_if = "Option::is_none")]
    pub credential_url: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SocialLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio: Option<String>,
}

/// Profile domain's copy of a user. `id` is the same id auth issued for
/// this user (auth is the credential source of truth); username/email/name/
/// role/avatarUrl/coverPhotoUrl are refreshed from auth on every profile
/// read via AuthClient, everything else here is owned and written locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(rename = "avatarUrl", skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(rename = "coverPhotoUrl", skip_serializing_if = "Option::is_none")]
    pub cover_photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub school: Option<String>,
    #[serde(rename = "whatsappNumber", skip_serializing_if = "Option::is_none")]
    pub whatsapp_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,
    pub role: String,
    #[serde(rename = "platformId", skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiences: Option<Vec<Experience>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education: Option<Vec<Education>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certifications: Option<Vec<Certification>>,
    #[serde(rename = "socialLinks", skip_serializing_if = "Option::is_none")]
    pub social_links: Option<SocialLinks>,
    #[serde(rename = "createdAt")]
    pub created_at: bson::DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: bson::DateTime,
    #[serde(rename = "deletedAt", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<bson::DateTime>,
}

impl User {
    pub fn id_string(&self) -> String {
        self.id.map(|id| id.to_hex()).unwrap_or_default()
    }
}
