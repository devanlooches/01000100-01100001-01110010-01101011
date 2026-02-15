use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};
use leptos::task::spawn_local;
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

    // Step 3: Execute Python script using venv
    println!("[run_model] STEP 3: Executing python3 run_model.py");
    println!("[run_model] ========================================");
    
    let output = Command::new(".venv/bin/python3")
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

/// Server function that generates a random 64x64x64 array with n random floats
/// between 1 and 1000, and -1 for the rest.
///
/// # Arguments
/// * `n` - Number of random float elements (1-1000 range). Defaults to random between 50-500 if None
///
/// # Returns
/// Save galaxy data from JavaScript to user_input.npy
#[server]
pub async fn save_galaxy_data(galaxy_json: String) -> Result<(), ServerFnError> {
    use serde_json::Value;

    println!("[save_galaxy_data] Received galaxy data JSON");

    // Parse the galaxy JSON
    let galaxy_map: Value = serde_json::from_str(&galaxy_json)
        .map_err(|e| ServerFnError::new(format!("Failed to parse galaxy JSON: {}", e)))?;

    // Create grid filled with -1.0 (64, 64, 64)
    let mut array_data = vec![-1.0; 64 * 64 * 64];

    // Fill the grid with proper density values
    if let Some(obj) = galaxy_map.as_object() {
        for (_, value) in obj.iter() {
            if let Some(arr) = value.as_array() {
                if arr.len() >= 4 {
                    // Array format: [density, x, y, z]
                    let density = arr[0].as_f64().unwrap_or(-1.0) as f32;
                    let x = arr[1].as_u64().unwrap_or(0) as usize;
                    let y = arr[2].as_u64().unwrap_or(0) as usize;
                    let z = arr[3].as_u64().unwrap_or(0) as usize;

                    // grid[x, y, z] = density
                    if x < 64 && y < 64 && z < 64 {
                        let index = x * 64 * 64 + y * 64 + z;
                        array_data[index] = density;
                    }
                }
            }
        }
    }

    // Write to user_input.npy - save data as binary with NPY header
    let mut npy_data = Vec::new();
    
    // NPY magic number
    npy_data.extend_from_slice(b"\x93NUMPY");
    
    // Version (1, 0)
    npy_data.push(1);
    npy_data.push(0);
    
    // Header dict as string
    let header_dict = format!(
        "{{'descr': '<f4', 'fortran_order': False, 'shape': (64, 64, 64)}}                                                                             "
    );
    let header_len = header_dict.len() as u16;
    npy_data.extend_from_slice(&header_len.to_le_bytes());
    npy_data.extend_from_slice(header_dict.as_bytes());
    
    // Data (f32 in little-endian)
    for &val in &array_data {
        npy_data.extend_from_slice(&val.to_le_bytes());
    }
    
    std::fs::write("user_input.npy", &npy_data).map_err(|e| {
        let err_msg = format!("Failed to save user_input.npy: {}", e);
        println!("[save_galaxy_data] ERROR: {}", err_msg);
        ServerFnError::new(err_msg)
    })?;

    println!("[save_galaxy_data] SUCCESS: Galaxy data saved to user_input.npy");
    Ok(())
}

/// NpyData with shape [64, 64, 64] saved as user_input.npy
#[server]
pub async fn generate_npy_data(n: Option<u64>) -> Result<NpyData, ServerFnError> {
    use ndarray::Array3;
    use rand::distributions::Uniform;

    println!("[generate_npy_data] Called with n: {:?}", n);

    // Determine number of random elements
    let num_random = if let Some(count) = n {
        count.min(500).max(50) // Clamp between 50-500
    } else {
        // Random between 50-500 if not provided
        let mut rng = rand::thread_rng();
        use rand::Rng;
        rng.gen_range(50..501)
    };

    println!("[generate_npy_data] Using {} random elements", num_random);

    // Create 64x64x64 array filled with -1.0
    let mut array: Array3<f32> = Array3::from_elem((64, 64, 64), -1.0);
    let total_elements = 64 * 64 * 64; // 262144

    // Generate random indices for the non-negative values
    let mut indices: Vec<usize> = (0..total_elements).collect();
    
    // Shuffle indices
    use rand::seq::SliceRandom;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    indices.shuffle(&mut rng);

    // Generate random values between 1 and 1000 for first n_random indices
    let dist = Uniform::new(1.0, 1000.0);
    for i in 0..num_random.min(total_elements as u64) as usize {
        let idx = indices[i];
        let x = idx / (64 * 64);
        let y = (idx / 64) % 64;
        let z = idx % 64;
        array[[x, y, z]] = rng.sample(dist);
    }

    println!("[generate_npy_data] Generated array with shape: {:?}", array.shape());

    // Flatten to f32 vec
    let data: Vec<f32> = array.into_iter().collect();
    let shape = vec![64u64, 64u64, 64u64];

    // Write to user_input.npy - save data as binary with NPY header
    println!("[generate_npy_data] Writing to user_input.npy...");
    
    // Create NPY file manually
    let mut npy_data = Vec::new();
    
    // NPY magic number
    npy_data.extend_from_slice(b"\x93NUMPY");
    
    // Version (1, 0)
    npy_data.push(1);
    npy_data.push(0);
    
    // Header dict as string
    let header_dict = format!(
        "{{'descr': '<f4', 'fortran_order': False, 'shape': (64, 64, 64)}}                                                                             "
    );
    let header_len = header_dict.len() as u16;
    npy_data.extend_from_slice(&header_len.to_le_bytes());
    npy_data.extend_from_slice(header_dict.as_bytes());
    
    // Data (f32 in little-endian)
    for &val in &data {
        npy_data.extend_from_slice(&val.to_le_bytes());
    }
    
    std::fs::write("user_input.npy", &npy_data).map_err(|e| {
        let err_msg = format!("Failed to save user_input.npy: {}", e);
        println!("[generate_npy_data] ERROR: {}", err_msg);
        ServerFnError::new(err_msg)
    })?;

    println!("[generate_npy_data] SUCCESS: Generated and saved user_input.npy with {} elements", data.len());
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

    #[wasm_bindgen(js_name = setOpacitiesFromDensities)]
    fn set_opacities_from_densities(array: &[f32]);

    #[wasm_bindgen(js_name = generateGalaxies)]
    fn generate_galaxies(count: u32);

    #[wasm_bindgen(js_name = getGalaxies)]
    fn get_galaxies() -> String;
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
    
    // Galaxy count input state
    let galaxy_count = RwSignal::new("".to_string());
    
    // Model running state
    let model_running = RwSignal::new(false);
    let model_status = RwSignal::new("".to_string());
    
    // Cached precomputed model output (for sneaky background processing)
    let cached_model_output: RwSignal<Option<NpyData>> = RwSignal::new(None);

    #[cfg(not(feature = "ssr"))]
    {
        // Generate random NPY on page load
        Effect::new(move |_| {
            spawn_local(async {
                match generate_npy_data(None).await {
                    Ok(data) => {
                        println!("[HomePage] Generated NPY data with shape: {:?}", data.shape);
                    }
                    Err(e) => {
                        eprintln!("[HomePage] Error generating NPY: {:?}", e);
                    }
                }
            });
        });

        // Generate galaxies on page load with default count
        Effect::new(move |_| {
            if let Ok(count) = galaxy_count.get().parse::<u32>() {
                generate_galaxies(count);
            }
        });

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

          <audio
                autoplay=true
                loop=true
                controls=true  // Show the play/pause/volume controls
                src="/assets/darkk.mp3"
                class="audio-player"
            />

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
                        prop:value=galaxy_count
                        on:change=move |ev| {
                            let new_value = event_target_value(&ev);
                            galaxy_count.set(new_value.clone());
                            // Call generate_npy_data with the new galaxy count
                            if let Ok(count) = new_value.parse::<u64>() {
                                // Call generateGalaxies on the client
                                #[cfg(not(feature = "ssr"))]
                                {
                                    generate_galaxies(count as u32);
                                }
                                spawn_local(async move {
                                    match generate_npy_data(Some(count)).await {
                                        Ok(data) => {
                                            println!("[HomePage] Generated NPY data with {} elements", data.data.len());
                                        }
                                        Err(e) => {
                                            eprintln!("[HomePage] Error generating NPY: {:?}", e);
                                        }
                                    }
                                });
                            }
                        }
                    />
                    <p class="input-note">"Please select a number between 50 and 500"</p>
                </div>
                <div class="button-group">
                    <button
                        class="submit-galaxy-btn"
                        on:click=move |_| {
                            galaxy_count.set("".to_string());
                            // Get galaxy data from JavaScript
                            #[cfg(not(feature = "ssr"))]
                            {
                                let galaxy_json = get_galaxies();
                                
                                // Save galaxy data to server (creates user_input.npy)
                                spawn_local(async move {
                                    match save_galaxy_data(galaxy_json).await {
                                        Ok(_) => {
                                            println!("[HomePage] Galaxy data saved successfully");
                                        }
                                        Err(e) => {
                                            eprintln!("[HomePage] Error saving galaxy data: {:?}", e);
                                        }
                                    }
                                });
                            }
                            
                            // Sneakily start model inference in the background (no UI updates)
                            spawn_local(async move {
                                match run_model(
                                    "user_input.npy".to_string(),
                                    "model_final.keras".to_string(),
                                    None
                                ).await {
                                    Ok(output_data) => {
                                        println!("[HomePage] Background model inference complete. Output shape: {:?}", output_data.shape);
                                        // Cache the result but don't update visualization yet
                                        cached_model_output.set(Some(output_data));
                                    }
                                    Err(e) => {
                                        eprintln!("[HomePage] Background model error: {:?}", e);
                                    }
                                }
                            });
                        }
                    >
                        "Place Galaxies"
                    </button>
                    <button
                        class="run-model-btn"
                    on:click=move |_| {
                         let model_running_clone = model_running.clone();
                         let model_status_clone = model_status.clone();
                         
                         // Check if we have cached results from background processing
                         if let Some(_cached_output_data) = cached_model_output.get() {
                             // Results already computed in background, apply them immediately
                             println!("[HomePage] Using cached model inference results");
                             #[cfg(not(feature = "ssr"))]
                             {
                                 let cached_output_data = _cached_output_data;
                                 set_opacities_from_densities(&cached_output_data.data);
                             }
                             model_status_clone.set("Model complete!".to_string());
                         } else {
                             // No cached results yet, show loading message
                             model_status_clone.set("Model loading... This may take a while".to_string());
                             
                             // No cached results, run the model now
                             spawn_local(async move {
                                 model_running_clone.set(true);
                                 
                                 match run_model(
                                     "user_input.npy".to_string(),
                                     "model_final.keras".to_string(),
                                     None
                                 ).await {
                                     Ok(output_data) => {
                                         println!("[HomePage] Model inference complete. Output shape: {:?}", output_data.shape);
                                         
                                         // Update visualization with output data
                                         #[cfg(not(feature = "ssr"))]
                                         {
                                             set_opacities_from_densities(&output_data.data);
                                         }
                                         
                                         model_status_clone.set("Model complete!".to_string());
                                     }
                                     Err(e) => {
                                         model_status_clone.set(format!("Error: {:?}", e));
                                         eprintln!("[HomePage] Model error: {:?}", e);
                                     }
                                 }
                                 
                                 model_running_clone.set(false);
                             });
                         }
                     }
                    disabled=model_running
                >
                    {move || if model_running.get() { "Running..." } else { "Run Model" }}
                    </button>
                </div>
                <p class="model-status">{move || model_status.get()}</p>
                <p class="settings-hint">"Press O to close"</p>
            </div>
        </div>


       <div class="about-overlay" class:open=move || about_open.get()>
           <div class="about-panel">
               <h1 class="about-title">"About"</h1>
               <p class="about-sub">"Estimator of most significant dark matter clusters over 10 billion years"</p>

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
