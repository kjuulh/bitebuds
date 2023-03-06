use chrono::Datelike;
use leptos::*;

use crate::api::events::*;
use domain::{EventOverview, Image};

#[component]
pub fn Day(
    cx: Scope,
    event: EventOverview,
    next: Option<bool>,
    last: Option<bool>,
) -> impl IntoView {
    let (expanded, set_expanded) = create_signal(cx, false);
    let day = event.time.weekday().to_string();
    let timestamp = event.time.format("%Y-%m-%d").to_string();

    view! {
        cx,
        <div class="sm:grid grid-cols-[1fr,4fr] gap-4 space-y-4 sm:space-y-0">
            <div class="relative">
                {if !last.unwrap_or(false) {
                    view! {
                        cx,
                        <div class="bg-gray-300 absolute top-3 left-[3px] h-full w-0.5 hidden sm:block z-0"/>
                    }.into_view(cx)
                } else {
                    view! {cx, <div></div>}.into_view(cx)
                }}
                <div class="col-start-1 flex space-x-2">
                    <div class={format!("hidden sm:block w-2 h-2 rounded-full mt-2.5 z-10 {}", if next.unwrap_or(false) {"bg-orange-600"} else { "bg-gray-300"})} />
                    <div class="inline-block">
                        <span class={format!("text-md font-medium {}", if next.unwrap_or(false) {"text-orange-600"} else {"text-gray-700"})}>
                            {day}
                        </span>
                        <p class="text-xs font-normal text-gray-500">
                            {timestamp}
                        </p>
                    </div>
                </div>
            </div>
            <div class="col-start-2 transition-all sm:pb-6">
                {move || if expanded() == true {
                    view! {
                        cx,
                        <DayContentExpanded event_id=event.id.clone()/>
                    }.into_view(cx)
                } else {
                    view! {
                        cx,
                        <DayContentCollapsed event=event.clone() setter=set_expanded />
                    }.into_view(cx)
                }}
            </div>
        </div>
        <div class="divider block sm:hidden h-0.5 w-full bg-gray-300 my-6 rounded-full" />
    }
}

#[component]
fn DayContentExpanded(cx: Scope, event_id: uuid::Uuid) -> impl IntoView {
    let full_event = create_resource(cx, move || (), move |_| get_full_event(event_id));

    let image = |cx: Scope, image: Option<Image>| {
        if let Some(image) = image { 
            view! {
                cx, 
                <img src={image.url} alt=image.alt class="object-cover max-h-[250px] " />
            }.into_view(cx)
        } else {
            view! {cx, <div></div>}.into_view(cx)
        }
    };

    let event_view = move || full_event.with(cx, |event| {
        event.clone().map(|event| {event.map(|event| view! {
            cx, 
    <article class="day-content space-x-3 min-h-[150px] flex flex-col">
        {image(cx, event.cover_image)}
        <div class="day-content__body space-y-2 pt-6">
            <h2 class="font-semibold text-xl text-orange-600">{event.name}</h2>

            {
                event.description.map(|d| view! {cx, 
                    <p class="font-normal sm:px-6 text">
                        {d}
                    </p>
                    })  
            }

            {
                event.recipe_id.map(|_r| {view! {cx, 
                    <h3 class="font-medium text-lg pt-2 text-orange-600">"Recipe"</h3>

                    <ol class="px-10">
                        <li class="list-item list-decimal">
                            "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur
                            cupidatat."
                        </li>
                        <li class="list-item list-decimal">
                            "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur
                            cupidatat."
                        </li>
                    </ol>

                    <h3 class="font-medium text-lg pt-2 text-orange-600">"References"</h3>
                    <ul class="px-10">
                        <li class="list-item list-decimal">
                            <a href={r"https://google.com"}>
                    "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat"</a>
                        </li>
                    </ul>

                    <h3 class="font-medium text-lg pt-2 text-orange-600">"Images"</h3>
                    <div class="day-content__images grid grid-cols-3 gap-4 mx-4 pt-2">
                        <img
                            src={r"https://images.unsplash.com/photo-1677856217391-838a585c8290?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=2070&q=80"}
                            alt="no alt text in sight"
                            class="object-cover"
                        />
                        <img
                            src={r"https://images.unsplash.com/photo-1677856217391-838a585c8290?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=2070&q=80"}
                            alt="no alt text in sight"
                            class="object-cover"
                        />
                        <img
                            src={r"https://images.unsplash.com/photo-1677856217391-838a585c8290?ixlib=rb-4.0.3&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=2070&q=80"}
                            alt="no alt text in sight"
                            class="object-cover"
                        />
                    </div>
                }})     

            }
        </div>
    </article>

    <div class="pb-10" />
        })})
    });

    view! {
        cx,
        <Suspense fallback=move || view! {cx, <p>"Loading events..."</p>}>
                {event_view}
        </Suspense>
    }
}

#[component]
fn DayContentCollapsed(
    cx: Scope,
    setter: WriteSignal<bool>,
    event: EventOverview,
) -> impl IntoView {
    let image = event.cover_image.clone();

    view! {
    cx,
    <article class="day-content sm:grid grid-cols-[30%,70%] sm:space-x-6 flex flex-col">
        {if let Some(image) = image {
                view! {
                    cx,
                    <div class="content-start justify-start">
                        <img src={image.url} alt=image.alt class="object-cover place-self-center w-full max-w-full max-h-[150px] sm:max-h-full " />
                    </div>
                }.into_view(cx)
            } else {
                    view!{cx, <div></div>}.into_view(cx)
                }}
        <div class="day-content__body flex flex-col">
            <h2 class="font-semibold text-lg text-orange-600">{event.name}</h2>
            {if let Some(mut description) = event.description.clone() {
                description.truncate(120);
                view! {cx,
                    <p class="font-normal text-sm">
                       {
                            if description.len() == 120 {
                                format!("{description}...")
                            } else {
                                description
                            }
                        }
                    </p>
                }.into_view(cx)
            } else {
                    view! {cx, <div></div>}.into_view(cx)
                }}
            <div class="flex-grow" />
            <button
            on:click=move |_| setter.update(|value| *value = !*value)
                class="transition-all h-3 w-20 bg-gray-200 hover:bg-gray-300 self-center rounded-b-[4rem] rounded-t-[1rem] mt-3"
            />
        </div>
    </article>
    }
}
