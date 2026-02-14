use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};
use serde::{Deserialize, Serialize};

/// The parsed contents of a .npy file: a flat data buffer plus its shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpyData {
    /// Flattened array values (row-major / C order).
    pub data: Vec<f32>,
    /// Original shape, e.g. [100, 3] for 100 particles × 3 coords.
    pub shape: Vec<u64>,
}

/// Server function that loads a .npy file and returns it as JSON-serialisable data.
/// `run_id` is a placeholder for a future API parameter (e.g. "run0100_dm").
#[server]
pub async fn load_npy(run_id: String) -> Result<NpyData, ServerFnError> {
    use npyz::NpyFile;

    // TODO: update the base URL once the Django API is deployed.
    let api_url = format!("http://localhost:8000/api/simulations/{run_id}/npy");

    let bytes = match reqwest::get(&api_url).await {
        Ok(resp) if resp.status().is_success() => resp
            .bytes()
            .await
            .map_err(|e| ServerFnError::new(format!("failed to read response body: {e}")))?
            .to_vec(),
        _ => {
            // Fallback: read from local disk while the API is not yet available.
            let path = format!("{run_id}.npy");
            std::fs::read(&path)
                .map_err(|e| ServerFnError::new(format!("failed to read {path}: {e}")))?
        }
    };

    let npy = NpyFile::new(&bytes[..])
        .map_err(|e| ServerFnError::new(format!("failed to parse npy: {e}")))?;

    let shape = npy.shape().to_vec();

    let data: Vec<f32> = npy
        .into_vec::<f32>()
        .map_err(|e| ServerFnError::new(format!("failed to read npy data as f32: {e}")))?;

    Ok(NpyData { data, shape })
}

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
            <h1 class="title">
                "01100100011000010111001001101011(DARK)"
            </h1>

            <p class="main-hint" class:hidden=move || splash_visible.get()>
                "Press O to open menu"
            </p>

            <div class="settings-pane" class:settings-open=settings_open>
                <h2>"Settings"</h2>
                <div class="input-group">
                        <label for="star-count">"Galaxy Count"</label>
                        <input
                            id="star-count"
                            type="number"
                            min="50"
                            max="500"
                            value="500"
                        />
                 </div>
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
