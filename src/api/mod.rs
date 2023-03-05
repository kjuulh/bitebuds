pub mod events;

use leptos::*;

#[cfg(feature = "ssr")]
pub fn register() {
    events::GetUpcomingEvents::register();
    events::GetFullEvent::register();
}
