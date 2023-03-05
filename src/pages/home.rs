use leptos::*;

use crate::api;
use crate::components::day::{Day, DayProps};

#[component]
pub fn HomePage(cx: Scope) -> impl IntoView {
    let events = create_resource(
        cx,
        || (),
        |_| async { api::events::get_upcoming_events().await },
    );

    let events_view = move || {
        events.with(cx, |events| {
            events.clone().map(|event_overview| {
                event_overview
                    .events
                    .iter()
                    .enumerate()
                    .map(|(index, event)| {
                        view! {
                            cx,
                            <Day
                                event=event.clone()
                                next={Some(index == 0)}
                                last={
                                    if event_overview.events.len() - 1 == index {
                                        Some(true)
                                    } else {
                                        None
                                    }
                                }
                            />
                        }
                    })
                    .collect::<Vec<_>>()
            })
        })
    };

    view! {
    cx,
    <div class="space-y-4 pt-8">
        <Suspense fallback=move || view! {cx, <p>"Loading events..."</p>}>
            <ul class="days flex flex-col">
                {events_view}
            </ul>
        </Suspense>
    </div>
    }
}
