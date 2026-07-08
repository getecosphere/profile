use serde::{Deserialize, Serialize};

use crate::models::user::{Certification, Education, Experience, SocialLinks, User};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceDto {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currently_working: Option<bool>,
}

impl From<&Experience> for ExperienceDto {
    fn from(e: &Experience) -> Self {
        ExperienceDto {
            id: e.id.clone(),
            title: e.title.clone(),
            company: e.company.clone(),
            location: e.location.clone(),
            description: e.description.clone(),
            start_date: e.start_date.map(|d| d.to_chrono()),
            end_date: e.end_date.map(|d| d.to_chrono()),
            currently_working: e.currently_working,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationDto {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub school: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degree: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_of_study: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl From<&Education> for EducationDto {
    fn from(e: &Education) -> Self {
        EducationDto {
            id: e.id.clone(),
            school: e.school.clone(),
            degree: e.degree.clone(),
            field_of_study: e.field_of_study.clone(),
            start_date: e.start_date.map(|d| d.to_chrono()),
            end_date: e.end_date.map(|d| d.to_chrono()),
            description: e.description.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificationDto {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_url: Option<String>,
}

impl From<&Certification> for CertificationDto {
    fn from(c: &Certification) -> Self {
        CertificationDto {
            id: c.id.clone(),
            name: c.name.clone(),
            issuer: c.issuer.clone(),
            issue_date: c.issue_date.map(|d| d.to_chrono()),
            expiration_date: c.expiration_date.map(|d| d.to_chrono()),
            credential_url: c.credential_url.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialLinksDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portfolio: Option<String>,
}

impl From<&SocialLinks> for SocialLinksDto {
    fn from(s: &SocialLinks) -> Self {
        SocialLinksDto {
            linkedin: s.linkedin.clone(),
            twitter: s.twitter.clone(),
            github: s.github.clone(),
            portfolio: s.portfolio.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub school: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whatsapp_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interests: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiences: Option<Vec<ExperienceDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education: Option<Vec<EducationDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certifications: Option<Vec<CertificationDto>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_links: Option<SocialLinksDto>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<&User> for UserDto {
    fn from(u: &User) -> Self {
        UserDto {
            id: u.id_string(),
            name: u.name.clone(),
            username: u.username.clone(),
            email: u.email.clone(),
            headline: u.headline.clone(),
            avatar_url: u.avatar_url.clone(),
            cover_photo_url: u.cover_photo_url.clone(),
            bio: u.bio.clone(),
            location: u.location.clone(),
            website: u.website.clone(),
            school: u.school.clone(),
            whatsapp_number: u.whatsapp_number.clone(),
            province: u.province.clone(),
            role: u.role.clone(),
            platform_id: u.platform_id.clone(),
            interests: u.interests.clone(),
            experiences: u
                .experiences
                .as_ref()
                .map(|es| es.iter().map(ExperienceDto::from).collect()),
            education: u
                .education
                .as_ref()
                .map(|es| es.iter().map(EducationDto::from).collect()),
            skills: u.skills.clone(),
            certifications: u
                .certifications
                .as_ref()
                .map(|cs| cs.iter().map(CertificationDto::from).collect()),
            social_links: u.social_links.as_ref().map(SocialLinksDto::from),
            created_at: u.created_at.to_chrono(),
            updated_at: u.updated_at.to_chrono(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    #[serde(default)]
    pub interests: Option<Vec<String>>,
    pub whatsapp_number: Option<String>,
    pub province: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "platformId")]
    pub platform_id: String,
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceRequest {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub currently_working: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationRequest {
    pub school: Option<String>,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificationRequest {
    pub name: Option<String>,
    pub issuer: Option<String>,
    pub issue_date: Option<String>,
    pub expiration_date: Option<String>,
    pub credential_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SocialLinksRequest {
    pub linkedin: Option<String>,
    pub twitter: Option<String>,
    pub github: Option<String>,
    pub portfolio: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddSkillQuery {
    pub skill: String,
}

#[derive(Debug, Deserialize)]
pub struct AddTagQuery {
    pub name: String,
}
