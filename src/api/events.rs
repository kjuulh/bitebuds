use std::path::PathBuf;

use cfg_if::cfg_if;
use leptos::*;
use serde::{Deserialize, Serialize};

use domain::{Event, EventOverview};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use services::EventStore;
        use lazy_static::lazy_static;

        lazy_static! {
            static ref EVENTSTORE: EventStore = EventStore::new(PathBuf::from("articles"));
        }
        async fn get_upcoming_events_fn() -> Result<UpcomingEventsOverview, ServerFnError> {
            let mut events: Vec<EventOverview> = EVENTSTORE
                .get_upcoming_events()
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?
                .iter()
                .filter(|d| d.time.ge(&chrono::Utc::now().date_naive()))
                .map(|data| data.clone().into())
                .collect();

            events.sort_by(|a, b| a.time.cmp(&b.time))          ;

            Ok(UpcomingEventsOverview { events })
        }
        async fn get_full_event_fn(event_id: uuid::Uuid) -> Result<Option<Event>, ServerFnError> {
            let event = EVENTSTORE
                .get_event(event_id)
                .await
                .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            Ok(event)
        }

        pub async fn boostrap() -> Result<(), ServerFnError> {
            EVENTSTORE.bootstrap().await.map_err(|e| ServerFnError::ServerError(e.to_string()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpcomingEventsOverview {
    pub events: Vec<EventOverview>,
}

#[server(GetUpcomingEvents, "/api")]
pub async fn get_upcoming_events() -> Result<UpcomingEventsOverview, ServerFnError> {
    get_upcoming_events_fn().await
}

#[server(GetFullEvent, "/api")]
pub async fn get_full_event(event_id: uuid::Uuid) -> Result<Option<Event>, ServerFnError> {
    get_full_event_fn(event_id).await
}
