use std::path::PathBuf;

use domain::{Event, Image, Metadata};
use serde::{Deserialize, Serialize};

pub struct EventStore {
    pub path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawImage {
    pub url: String,
    pub alt: String,
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawEvent {
    pub cover_image: Option<RawImage>,
    pub name: String,
    pub description: Option<String>,
    #[serde(with = "short_time_stamp")]
    pub time: chrono::NaiveDate,
    pub recipe_id: Option<uuid::Uuid>,
    //pub images: Vec<RawImage>,
    pub metadata: Option<Metadata>,
    #[serde(skip)]
    pub content: String,
}

mod short_time_stamp {
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

impl From<RawEvent> for Event {
    fn from(value: RawEvent) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            cover_image: value.cover_image.map(|ci| ci.into()),
            name: value.name,
            description: value.description,
            time: value.time,
            recipe_id: value.recipe_id,
            images: vec![],
            metadata: value.metadata,
        }
    }
}

impl From<RawImage> for Image {
    fn from(value: RawImage) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            url: value.url,
            alt: value.alt,
            metadata: value.metadata,
        }
    }
}

impl EventStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub async fn get_upcoming_events(&self) -> eyre::Result<Vec<Event>> {
        let mut event_path = self.path.clone();
        event_path.push("events");
        let mut dir = tokio::fs::read_dir(event_path).await?;

        let mut events = vec![];

        while let Ok(Some(entry)) = dir.next_entry().await {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                let file = tokio::fs::read(entry.path()).await?;
                let content = std::str::from_utf8(&file)?;
                if content.starts_with("---\n") {
                    let after_marker = &content[4..];
                    if let Some(marker_end) = after_marker.find("---\n") {
                        let raw_front_matter = &content[4..marker_end + 4];
                        let mut raw_event: RawEvent = serde_yaml::from_str(raw_front_matter)?;
                        raw_event.content = content[marker_end + 4..].to_string();

                        events.push(raw_event.into())
                    }
                }
            }
        }

        Ok(events)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self {
            path: PathBuf::from("articles"),
        }
    }
}

#[cfg(test)]
mod test {
    use domain::Event;

    use crate::RawEvent;

    #[test]
    fn can_parse_event() {
        let raw = r#"coverImage:
  url: "https://cdn-rdb.arla.com/Files/arla-dk/2010638351/0606cf14-3972-4abb-b2c8-faa3249de170.jpg?crop=(0,482,0,-117)&w=1269&h=715&mode=crop&ak=6826258c&hm=f35b5bfe" 
  alt: billede af oksesteg
name: Gammeldags oksesteg
description: |
  God gammeldags oksesteg med en intens og fyldig brun sauce. Gammeldags oksesteg
  er rigtig simremad som gør de fleste glade. Så server en gammeldags oksesteg for
  din gæster... både de unge og de gamle.
time: 2023-03-06"#;

        let raw_event: RawEvent = serde_yaml::from_str(raw).unwrap();
        let _: Event = raw_event.into();
    }
}
