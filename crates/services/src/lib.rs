use cached::proc_macro::once;
use domain::{Event, Image, Metadata};
use gitevents_sdk::events::EventResponse;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawImage {
    pub url: String,
    pub alt: String,
    pub metadata: Option<Metadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawEvent {
    #[serde(alias = "coverImage")]
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
    use chrono::NaiveDate;
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

struct InnerEventStore {
    url: Option<String>,
    pub path: PathBuf,
    events: Arc<tokio::sync::RwLock<Vec<Event>>>,
    url_path: Option<String>,
}

#[derive(Clone)]
pub struct EventStore {
    inner: Arc<InnerEventStore>,
}

impl EventStore {
    pub fn new(path: PathBuf) -> Self {
        let article_repo_url = std::env::var("BITE_ARTICLE_REPO_URL")
            .map(|a| (a != "").then(|| a))
            .unwrap_or(None);
        let article_repo_path = std::env::var("BITE_ARTICLE_REPO_PATH")
            .map(|a| (a != "").then(|| a))
            .unwrap_or(None);
        Self {
            inner: Arc::new(InnerEventStore {
                url: article_repo_url,
                url_path: article_repo_path,
                path,
                events: Default::default(),
            }),
        }
    }

    pub async fn bootstrap(&self) -> eyre::Result<()> {
        tracing::info!("boostrapping event_store");
        //let mut event_path = self.inner.path.clone();
        //event_path.push("events");

        //let events = fetch_events(event_path.clone()).await?;

        //let mut e = self.inner.events.write().await;
        //*e = events;

        if let Some(repo_url) = self.inner.url.clone() {
            tracing::info!(repo_url = repo_url, "subscribing to repo");
            let inner = self.inner.clone();

            tokio::task::spawn(async move {
                gitevents_sdk::builder::Builder::new()
                    .set_generic_git_url(repo_url)
                    .set_scheduler_opts(&gitevents_sdk::cron::SchedulerOpts {
                        duration: std::time::Duration::from_secs(30),
                    })
                    .action(move |req| {
                        let inner = inner.clone();

                        async move {
                            tracing::info!("updating articles");
                            let mut event_path = req.git.path.clone();
                            event_path.push(inner.url_path.as_ref().unwrap());

                            tracing::debug!(
                                path = event_path.display().to_string(),
                                "reading from"
                            );

                            let events = fetch_events(event_path).await.unwrap();

                            let mut e = inner.events.write().await;
                            *e = events.clone();

                            Ok(EventResponse {})
                        }
                    })
                    .execute()
                    .await
                    .unwrap();
            });
        }

        Ok(())
    }

    pub async fn get_upcoming_events(&self) -> eyre::Result<Vec<Event>> {
        let events = self.inner.events.read().await.clone();

        Ok(events)
    }

    pub async fn get_event(&self, event_id: uuid::Uuid) -> eyre::Result<Option<Event>> {
        let events = self.inner.events.read().await;

        let event = events.iter().find(|e| e.id == event_id);

        Ok(event.map(|e| e.clone()))
    }
}

pub async fn fetch_events(event_path: PathBuf) -> eyre::Result<Vec<Event>> {
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

impl Default for EventStore {
    fn default() -> Self {
        Self {
            inner: Arc::new(InnerEventStore {
                url: Default::default(),
                path: PathBuf::from("articles"),
                events: Default::default(),
                url_path: Some("articles/events".into()),
            }),
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
