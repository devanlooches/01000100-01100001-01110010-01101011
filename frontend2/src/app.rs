use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};

#[cfg(not(feature = "ssr"))]
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "ssr"))]
#[wasm_bindgen(module = "/three.js")]
extern "C" {
    #[wasm_bindgen(js_name = initScene)]
    fn init_scene(canvas_id: &str, container_id: &str);
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/dark-matter-simulator.css"/>
        <Title text="Dark Matter Simulator"/>

        <Router>
            <main>
                <Routes fallback=move || "Not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(not(feature = "ssr"))]
#[wasm_bindgen(module = "/three.js")]
extern "C" {
    #[wasm_bindgen(js_name = listenForKey)]
    fn listen_for_key(key: &str, callback: &Closure<dyn Fn()>);
}

#[component]
fn HomePage() -> impl IntoView {
    let splash_visible = RwSignal::new(true);
    let settings_open = RwSignal::new(false);

    #[cfg(not(feature = "ssr"))]
    {
        let closure = Closure::new(move || {
            settings_open.update(|open| *open = !*open);
        });
        listen_for_key("o", &closure);
        closure.forget();
    }

    view! {

        <DarkMatterScene/>

        <div
                    class="splash"
                    class:splash-hidden=move || !splash_visible.get()
                    on:click=move |_| splash_visible.set(false)
                >
                    <h1 class="splash-title">"Dark Matter Simulator"</h1>
                    <p class="splash-sub">"Click to begin"</p>
                </div>

        <div class="ui-overlay">
            <h1 class="title">"Dark Matter Simulator"</h1>
            <div class="settings-pane" class:settings-open=settings_open>
                <h2>"Settings"</h2>
                <p class="settings-hint">"Press O to close"</p>
            </div>
        </div>
    }
}

#[component]
fn DarkMatterScene() -> impl IntoView {
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    #[cfg(not(feature = "ssr"))]
    {
        let canvas_ref = canvas_ref.clone();
        Effect::new(move |_| {
            if let Some(_) = canvas_ref.get() {
                init_scene("scene-canvas", "scene-container");
            }
        });
    }

    view! {
        <div id="scene-container" class="container">
                <canvas id="scene-canvas" node_ref=canvas_ref></canvas>
        </div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
