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

/// Server function that runs a TensorFlow model on input data via a Python script.
/// 
/// Workflow:
/// 1. Loads the input .npy file from disk
/// 2. Writes it to "user_input.npy"
/// 3. Calls the Python script: python3 run_model.py
/// 4. Reads the output from "output.npy" that the Python script generates
/// 5. Returns the results as NpyData
///
/// # Arguments
/// * `input_npy_path` - Path to the input .npy file (e.g., "run0100_dm.npy")
/// * `model_path` - Path to the model file (not used by simplified version but kept for API compatibility)
/// * `temp_output_path` - Not used in file-based version
///
/// # Returns
/// The inference output as NpyData (flattened array + shape)
#[server]
pub async fn run_model(
    input_npy_path: String,
    model_path: String,
    temp_output_path: Option<String>,
) -> Result<NpyData, ServerFnError> {
    use std::process::Command;
    use std::path::Path;
    use npyz::NpyFile;

    println!("[run_model] ========================================");
    println!("[run_model] Called with:");
    println!("[run_model]   input_npy_path: {}", input_npy_path);
    println!("[run_model]   model_path: {}", model_path);
    println!("[run_model]   temp_output_path: {:?}", temp_output_path);
    println!("[run_model] ========================================");

    // Step 1: Read input file
    println!("[run_model] STEP 1: Reading input file");
    if !Path::new(&input_npy_path).exists() {
        let err_msg = format!("Input file not found: {}", input_npy_path);
        println!("[run_model] ERROR: {}", err_msg);
        return Err(ServerFnError::new(err_msg));
    }

    let input_bytes = std::fs::read(&input_npy_path)
        .map_err(|e| {
            let err_msg = format!("Failed to read input file: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;
    println!("[run_model] Read {} bytes from {}", input_bytes.len(), input_npy_path);

    // Step 2: Write to user_input.npy
    println!("[run_model] STEP 2: Writing to user_input.npy");
    std::fs::write("user_input.npy", &input_bytes)
        .map_err(|e| {
            let err_msg = format!("Failed to write user_input.npy: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;
    println!("[run_model] Successfully wrote user_input.npy");

    // Step 3: Execute Python script
    println!("[run_model] STEP 3: Executing python3 run_model.py");
    println!("[run_model] ========================================");
    
    let output = Command::new("python3")
        .arg("run_model.py")
        .output()
        .map_err(|e| {
            let err_msg = format!("Failed to execute python script: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    println!("[run_model] Python script output:");
    
    if !stdout.is_empty() {
        println!("[run_model] --- STDOUT ---");
        println!("{}", stdout);
    }
    
    if !stderr.is_empty() {
        println!("[run_model] --- STDERR ---");
        println!("{}", stderr);
    }
    
    if stdout.is_empty() && stderr.is_empty() {
        println!("[run_model] (No output captured)");
    }
    println!("[run_model] ========================================");

    if !output.status.success() {
        let err_msg = format!("Python script failed with exit code: {}", output.status);
        println!("[run_model] ERROR: {}", err_msg);
        return Err(ServerFnError::new(err_msg));
    }
    println!("[run_model] Python script executed successfully");

    // Step 4: Read output.npy
    println!("[run_model] STEP 4: Reading output.npy");
    if !Path::new("output.npy").exists() {
        let err_msg = "Python script did not create output.npy file".to_string();
        println!("[run_model] ERROR: {}", err_msg);
        return Err(ServerFnError::new(err_msg));
    }

    let output_bytes = std::fs::read("output.npy")
        .map_err(|e| {
            let err_msg = format!("Failed to read output.npy: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;
    println!("[run_model] Read {} bytes from output.npy", output_bytes.len());

    // Step 5: Parse output.npy
    println!("[run_model] STEP 5: Parsing output.npy");
    let npy = NpyFile::new(&output_bytes[..])
        .map_err(|e| {
            let err_msg = format!("Failed to parse output.npy: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;

    let shape = npy.shape().to_vec();
    println!("[run_model] Parsed shape: {:?}", shape);

    let data: Vec<f32> = npy
        .into_vec::<f32>()
        .map_err(|e| {
            let err_msg = format!("Failed to read output.npy data as f32: {}", e);
            println!("[run_model] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;

    println!("[run_model] SUCCESS: Loaded {} f32 values with shape {:?}", data.len(), shape);
    println!("[run_model] ========================================");
    Ok(NpyData { data, shape })
}

/// Server function that loads a .npy file and returns it as JSON-serialisable data.
/// `run_id` is a placeholder for a future API parameter (e.g. "run0100_dm").
///
/// This function can optionally run inference via `run_model` if a model path is provided.
#[server]
pub async fn load_npy(run_id: String) -> Result<NpyData, ServerFnError> {
    use npyz::NpyFile;

    println!("[load_npy] Called with run_id: {}", run_id);

    // First, try to read the .npy file from disk
    let path = format!("{run_id}.npy");
    println!("[load_npy] Attempting to read from disk: {}", path);
    
    let bytes = match std::fs::read(&path) {
        Ok(data) => {
            println!("[load_npy] Successfully read {} bytes from disk", data.len());
            data
        },
        Err(e) => {
            println!("[load_npy] Failed to read from disk: {}", e);
            // Fallback: try to fetch from API (if available)
            let api_url = format!("http://localhost:8000/api/simulations/{run_id}/npy");
            println!("[load_npy] Attempting to fetch from API: {}", api_url);
            match reqwest::get(&api_url).await {
                Ok(resp) if resp.status().is_success() => {
                    println!("[load_npy] API request successful");
                    resp
                        .bytes()
                        .await
                        .map_err(|e| {
                            let err_msg = format!("Failed to read response body: {}", e);
                            println!("[load_npy] ERROR: {}", err_msg);
                            ServerFnError::new(err_msg)
                        })?
                        .to_vec()
                },
                Ok(resp) => {
                    let err_msg = format!("API returned non-success status: {}", resp.status());
                    println!("[load_npy] ERROR: {}", err_msg);
                    return Err(ServerFnError::new(err_msg));
                },
                Err(e) => {
                    let err_msg = format!("Failed to read {}: {}, and API request failed: {}", path, e, e);
                    println!("[load_npy] ERROR: {}", err_msg);
                    return Err(ServerFnError::new(err_msg));
                }
            }
        }
    };

    // Parse the .npy file
    println!("[load_npy] Parsing .npy file from {} bytes", bytes.len());
    let npy = NpyFile::new(&bytes[..])
        .map_err(|e| {
            let err_msg = format!("Failed to parse npy: {}", e);
            println!("[load_npy] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;

    let shape = npy.shape().to_vec();
    println!("[load_npy] Parsed shape: {:?}", shape);

    let data: Vec<f32> = npy
        .into_vec::<f32>()
        .map_err(|e| {
            let err_msg = format!("Failed to read npy data as f32: {}", e);
            println!("[load_npy] ERROR: {}", err_msg);
            ServerFnError::new(err_msg)
        })?;

    println!("[load_npy] SUCCESS: Loaded {} f32 values with shape {:?}", data.len(), shape);
    Ok(NpyData { data, shape })
}

#[cfg(not(feature = "ssr"))]
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "ssr"))]
#[wasm_bindgen(module = "/three.js")]
extern "C" {
    #[wasm_bindgen(js_name = initScene)]
    fn init_scene(canvas_id: &str, container_id: &str);

    #[wasm_bindgen(js_name = listenForKey)]
    fn listen_for_key(key: &str, callback: &Closure<dyn Fn()>);
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

#[component]
fn HomePage() -> impl IntoView {
    let splash_visible = RwSignal::new(true);
    let settings_open = RwSignal::new(false);

    // NEW: About overlay open/close
    let about_open = RwSignal::new(false);

    #[cfg(not(feature = "ssr"))]
    {
        // O toggles settings
        let closure = Closure::new(move || {
            settings_open.update(|open| *open = !*open);
        });
        listen_for_key("o", &closure);
        closure.forget();

        // I toggles About overlay
        let about_toggle = Closure::new(move || {
            about_open.update(|v| *v = !*v);
        });
        listen_for_key("i", &about_toggle);
        about_toggle.forget();
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
                "01100100 01100001 01110010 01101011 DARK"
            </h1>

            <p class="main-hint" class:hidden=move || splash_visible.get()>
                "Press O to open menu · Press I for About"
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
                    <p class="input-note">"Please select a number between 50 and 500"</p>
                </div>
                <p class="settings-hint">"Press O to close"</p>
            </div>
        </div>


       <div class="about-overlay" class:open=move || about_open.get()>
           <div class="about-panel">
               <h1 class="about-title">"About"</h1>
               <p class="about-sub">"The team behind Dark Matter Simulator"</p>

               <div class="more-info">
                   <p class="more-info-text">
                        "Our project presents an interactive visualization of dark matter dispersal patterns, "
                        "The methodology implemented in this simulation follows the UNet-based neural network "
                        "approach detailed in the research by Wang et al. (2024)."
                   </p>
                   <p class="citation">
                           "Wang, Z., Shi, F., Yang, X., Li, Q., Liu, Y., & Li, X. (2024). "
                           <em>"Mapping the large-scale density field of dark matter using artificial intelligence."</em>
                           " SCIENCE CHINA Physics, Mechanics & Astronomy, 67(1), 219513. "
                           <a href="https://doi.org/10.1007/s11433-023-2192-9" target="_blank" rel="noopener noreferrer">
                               "DOI: 10.1007/s11433-023-2192-9"
                           </a>
                      </p>
               </div>

               <div class="team-grid">
                   <div class="team-card">
                       <div class="team-name">"Miguel Angel"</div>
                       <div class="team-text">"Frontend & UI"</div>
                   </div>
                   <div class="team-card">
                       <div class="team-name">"Eric"</div>
                       <div class="team-text">"ML Training"</div>
                   </div>

                   <div class="team-card">
                       <div class="team-name">"Isaac"</div>
                       <div class="team-text">"Pipeline, Frontend & Visualization"</div>
                   </div>
                   <div class="team-card">
                       <div class="team-name">"Erin"</div>
                       <div class="team-text">"Backend & Data"</div>
                  </div>

                  <div class="team-card">
                       <div class="team-name">"Yann"</div>
                       <div class="team-text">"Frontend & Ml implementation"</div>
                    </div>
               </div>

               <button class="back-btn" on:click=move |_| about_open.set(false)>
                   "Back"
               </button>

                <p class="about-hint">"Press I to close"</p>
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
            if canvas_ref.get().is_some() {
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
