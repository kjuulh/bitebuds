use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::pages::home::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! { cx,
        <Stylesheet id="leptos" href="/pkg/ssr_modes.css" />
        <Title text="Bitebuds" />

        <script defer="true" data-domain="bitebuds.front.kjuulh.io" src="https://plausible.front.kjuulh.io/js/script.js"/>

        <Router>
            <div class="app grid lg:grid-cols-[25%,50%,25%] sm:grid-cols-[10%,80%,10%] grid-cols-[5%,90%,5%]">
                <main class="main col-start-2">
                    <div class="pt-4">
                        <h1 class="font-semibold text-xl tracking-wide">"Bitebuds"</h1>
                        <Routes>
                            <Route path="" view=|cx| view! { cx, <HomePage /> }/>
                        </Routes>
                    </div>
                </main>
            </div>
        </Router>
    }
}
