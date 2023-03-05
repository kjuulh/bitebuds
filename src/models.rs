use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata(HashMap<String, String>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: uuid::Uuid,
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    pub id: uuid::Uuid,
    pub url: String,
    pub alt: String,
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub id: uuid::Uuid,
    pub cover_image: Option<Image>,
    pub name: String,
    pub description: Option<String>,
    pub time: chrono::DateTime<chrono::Utc>,
    pub recipe_id: Option<uuid::Uuid>,
    pub images: Vec<Image>,
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventOverview {
    pub id: uuid::Uuid,
    pub cover_image: Option<Image>,
    pub name: String,
    pub description: Option<String>,
    pub time: chrono::DateTime<chrono::Utc>,
}

impl From<Event> for EventOverview {
    fn from(value: Event) -> Self {
        Self {
            id: value.id,
            cover_image: value.cover_image,
            name: value.name,
            description: value.description,
            time: value.time,
        }
    }
}
