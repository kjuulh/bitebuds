use lazy_static::lazy_static;
use leptos::*;
use serde::{Deserialize, Serialize};

use domain::{Event, EventOverview, Image};

lazy_static! {
    static ref EVENTS: Vec<Event> = vec![
        Event {
            cover_image: Some(Image {
                id: uuid::Uuid::new_v4(),
                url: "https://images.unsplash.com/photo-1513104890138-7c749659a591?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=400&q=80".into(),
                alt: "some-alt".into(),
                metadata: None,
            }),
            id: uuid::Uuid::new_v4(),
            name: "Pizza".into(),
            description: Some("Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.".into()),
            time: chrono::Utc::now()
                .checked_add_days(chrono::Days::new(1))
                .unwrap()
                .date_naive(),
            recipe_id: None,
            images: vec![],
            metadata: None,
        },
        Event {
            cover_image: Some(Image {
                id: uuid::Uuid::new_v4(),
                url: "https://images.unsplash.com/photo-1513104890138-7c749659a591?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=400&q=80".into(),
                alt: "some-alt".into(),
                metadata: None,
            }),
            id: uuid::Uuid::new_v4(),
            name: "Kød boller".into(),
            description: Some("Lorem ipsum dolor sit amet, officia excepteur ex fugiat reprehenderit enim labore culpa sint ad nisi Lorem pariatur mollit ex esse exercitation amet. Nisi anim cupidatat excepteur officia. Reprehenderit nostrud nostrud ipsum Lorem est aliquip amet voluptate voluptate dolor minim nulla est proident. Nostrud officia pariatur ut officia. Sit irure elit esse ea nulla sunt ex occaecat reprehenderit commodo officia dolor Lorem duis laboris cupidatat officia voluptate. Culpa proident adipisicing id nulla nisi laboris ex in Lorem sunt duis officia eiusmod. Aliqua reprehenderit commodo ex non excepteur duis sunt velit enim. Voluptate laboris sint cupidatat ullamco ut ea consectetur et est culpa et culpa duis.".into()),
            time: chrono::Utc::now()
                .checked_add_days(chrono::Days::new(4))
                .unwrap()
                .date_naive(),
            recipe_id: None,
            images: vec![],
            metadata: None,
        },
        Event {
            cover_image: Some(Image {
                id: uuid::Uuid::new_v4(),
                url: "https://images.unsplash.com/photo-1513104890138-7c749659a591?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=400&q=80".into(),
                alt: "some-alt".into(),
                metadata: None,
            }),
            id: uuid::Uuid::new_v4(),
            name: "Pizza".into(),
            description: Some("description".into()),
            time: chrono::Utc::now()
                .checked_sub_days(chrono::Days::new(2))
                .unwrap()
                .date_naive(),
            recipe_id: None,
            images: vec![],
            metadata: None,
        },
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpcomingEventsOverview {
    pub events: Vec<EventOverview>,
}

#[server(GetUpcomingEvents, "/api")]
pub async fn get_upcoming_events() -> Result<UpcomingEventsOverview, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    get_upcoming_events_fn().await
}

#[cfg(feature = "ssr")]
async fn get_upcoming_events_fn() -> Result<UpcomingEventsOverview, ServerFnError> {
    let current_time = chrono::Utc::now();

    let es = services::EventStore::default();

    let events = es
        .get_upcoming_events()
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .iter()
        .map(|data| data.clone().into())
        .collect();

    Ok(UpcomingEventsOverview { events })
}

#[server(GetFullEvent, "/api")]
pub async fn get_full_event(event_id: uuid::Uuid) -> Result<Option<Event>, ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    get_full_event_fn(event_id).await
}

#[cfg(feature = "ssr")]
async fn get_full_event_fn(event_id: uuid::Uuid) -> Result<Option<Event>, ServerFnError> {
    let event = EVENTS
        .iter()
        .find(|data| data.id == event_id)
        .map(|d| d.clone());

    Ok(event)
}
