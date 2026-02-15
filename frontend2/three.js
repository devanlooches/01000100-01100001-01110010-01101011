// Imports

const THREE = window.THREE;
if (!THREE) {
  console.error("THREE not loaded");
}

const OrbitControls = window.OrbitControls;
if (!OrbitControls) {
  console.error("OrbitControls not loaded");
}

// Config

const BG_COLOUR = new THREE.Color(0x000000);
const GALAXY_COLOUR = new THREE.Color(0x00ffff);
const DENSITY_COLOUR = new THREE.Color(0xff5500);
const STAR_COLOUR = new THREE.Color(0xffffff);
const CUBE_OPACITY_COEFFICIENT = 0.05;
const CUBE_RENDER_SIZE = 5;
const MIN_ZOOM = 0;
const MAX_ZOOM = 15;
const DEFUALT_ZOOM = 10;
const ZOOM_SPEED = 5;
const CAMERA_DAMPING = 0.05;
const DENSITY_MAX = 1000;
const GALAXY_MAX = 500;
const GALAXY_MIN = 50;
const STAR_COUNT = 1000;
const STAR_DISTANCE = 100;
const STAR_RANGE = 100;
const STAR_SIZE = 0.2;

// Global Variables

// Galaxy selection
let raycaster;
let mouse;
let highlightMesh;
const HIGHLIGHT_COLOUR = 0xff0000;
let hoveredInstanceId = null; // What the mouse is over NOW
let selectedInstanceId = null; // What the user clicked on
let selectionMesh; // The yellow box for the selected item
const SELECTION_COLOUR = 0xffff00; // Yellow
const MOVE_SPEED = 0.5;

let initialized = false;
let cubeLength;
let opacities;
let cubeletCount;
let geometry;
let scene;
let cubeletSize;
let animating;
let container;
let camera;
let renderer;
let controls;
let canvas;
let galaxies;
let material;
let mesh;

// Public Functions

// Initialize

export function initScene(canvasId, containerId) {
  console.log("Initializing Scenes...");
  if (initialized) return;
  console.log("Initializing Scene..."); // Debug: Confirm function starts

  // 1. Initialize Container & Canvas
  container = document.getElementById(containerId);
  canvas = document.getElementById(canvasId);
  if (!container || !canvas) {
    console.error("Could not find container or canvas");
    return;
  }

  // 2. Initialize Scene, Camera, Renderer
  scene = new THREE.Scene();
  scene.background = BG_COLOUR;

  camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100);
  camera.position.set(0, 0, DEFUALT_ZOOM);

  renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;

  controls = new OrbitControls(camera, renderer.domElement);
  controls.enablePan = false;
  controls.enableDamping = true;
  controls.dampingFactor = CAMERA_DAMPING;
  controls.minDistance = MIN_ZOOM;
  controls.maxDistance = MAX_ZOOM;
  controls.zoomSpeed = ZOOM_SPEED;

  initializeCube(64);
  randomizeOpacities();
  generateStars();

  //Initialize raycast
  raycaster = new THREE.Raycaster();
  mouse = new THREE.Vector2();

  // Red Hover Box (Keep this)
  const highlightGeo = new THREE.BoxGeometry(
    cubeletSize * 1.05,
    cubeletSize * 1.05,
    cubeletSize * 1.05,
  );
  const highlightMat = new THREE.MeshBasicMaterial({
    color: HIGHLIGHT_COLOUR,
    wireframe: true,
    transparent: true,
    opacity: 0.8,
  });
  highlightMesh = new THREE.Mesh(highlightGeo, highlightMat);
  highlightMesh.visible = false;
  scene.add(highlightMesh);

  // yellow selection box
  const selectGeo = new THREE.BoxGeometry(
    cubeletSize * 1.1,
    cubeletSize * 1.1,
    cubeletSize * 1.1,
  );
  const selectMat = new THREE.MeshBasicMaterial({
    color: SELECTION_COLOUR,
    wireframe: true,
    transparent: true,
    opacity: 0.9,
    depthTest: false, // Always visible
  });
  selectionMesh = new THREE.Mesh(selectGeo, selectMat);
  selectionMesh.visible = false;
  scene.add(selectionMesh);

  // mouse move listeners
  window.addEventListener("mousemove", onMouseMove, false);

  initialized = true;
  animating = false;
  console.log("Scene Initialized Successfully!");
}

// Randomize Opacities

export function randomizeOpacities() {
  for (let i = 0; i < cubeletCount; i++) {
    opacities[i] = Math.random() * CUBE_OPACITY_COEFFICIENT;
  }
  geometry.setAttribute(
    "opacity",
    new THREE.InstancedBufferAttribute(opacities, 1),
  );
  mesh.material.color.set(DENSITY_COLOUR);
}

// Set Opacities from Density Array

export function setOpacitiesFromDensities(array) {
  // Convert all densities to opacities
  for (let i = 0; i < array.length; i++) {
    if (array[i] < 0) array[i] = 0;
    if (array[i] > DENSITY_MAX) array = DENSITY_MAX;
    array[i] = (array[i] * CUBE_OPACITY_COEFFICIENT) / DENSITY_MAX;
  }
  setOpacities(array);
  showGalaxies();
}

// Keypress Listener

export function listenForKey(key, callback) {
  window.addEventListener("keydown", (ev) => {
    console.log(ev.key);
    if (ev.key === key) {
      callback();
    }
    if (ev.key === "g") {
    }
    if (ev.key === "h") {
    }
  });
}

// Generate Galaxies

export function generateGalaxies(galaxyCount) {
  if (galaxyCount > GALAXY_MAX) galaxyCount = GALAXY_MAX;
  if (galaxyCount < GALAXY_MIN) galaxyCount = GALAXY_MIN;
  galaxies = new Array(cubeletCount).fill(false);
  setOpacities(new Float32Array(cubeletCount).fill(0));
  for (let i = 0; i < galaxyCount; ) {
    let index = Math.floor(Math.random() * cubeletCount);
    if (!galaxies[index]) {
      galaxies[index] = true;
      opacities[index] = Math.random();
      i++;
    }
  }
  geometry.setAttribute(
    "opacity",
    new THREE.InstancedBufferAttribute(opacities, 1),
  );
  mesh.material.color.set(GALAXY_COLOUR);
}

export function getGalaxies() {
  let data = {};
  let index = 0;
  for (let i = 0; i < cubeletCount; i++) {
    if (galaxies[i]) {
      data[index] = [opacities[i] * DENSITY_MAX, getX(i), getY(i), getZ(i)];
      index++;
    }
  }
  return JSON.stringify(data);
}

// Private Functions

// Initialize Cube

function initializeCube(size) {
  // set cube variables
  cubeLength = size;
  cubeletCount = Math.pow(cubeLength, 3);
  opacities = new Float32Array(cubeletCount);
  cubeletSize = CUBE_RENDER_SIZE / cubeLength;

  // generate cubelet geometry and material
  geometry = new THREE.BoxGeometry(cubeletSize, cubeletSize, cubeletSize);
  material = new THREE.MeshStandardMaterial({
    color: DENSITY_COLOUR,
    transparent: true,
    depthWrite: false,
  });

  // apply custom material shader to allow for opacity changes
  material.onBeforeCompile = (shader) => {
    shader.vertexShader = `
        attribute float opacity;
        varying float vOpacity;
        ${shader.vertexShader}
      `.replace(
      `void main() {`,
      `void main() {
          vOpacity = opacity;`,
    );
    shader.fragmentShader = `
        varying float vOpacity;
        ${shader.fragmentShader}
      `.replace(
      `#include <opaque_fragment>`,
      `#include <opaque_fragment>
          gl_FragColor.a *= vOpacity;`,
    );
  };

  // initialize opacities
  geometry.setAttribute(
    "opacity",
    new THREE.InstancedBufferAttribute(opacities, 1),
  );

  // generate mesh of cubelets
  mesh = new THREE.InstancedMesh(geometry, material, cubeletCount);

  // initialize meshes
  mesh.castShadow = true;
  mesh.receiveShadow = true;

  // initialize dummy object used for position generation
  const dummy = new THREE.Object3D();

  // generate dummy (x, y, z) positions and apply to cubelets
  let i = 0;
  for (let x = 0; x < cubeLength; x++) {
    for (let y = 0; y < cubeLength; y++) {
      for (let z = 0; z < cubeLength; z++) {
        dummy.position.set(
          // offset positions by cube size to centre cube
          cubeletSize * (x + 0.5) - CUBE_RENDER_SIZE / 2,
          cubeletSize * (y + 0.5) - CUBE_RENDER_SIZE / 2,
          cubeletSize * (z + 0.5) - CUBE_RENDER_SIZE / 2,
        );
        // update position matrix
        dummy.updateMatrix();
        // push position matrix to cubelet
        mesh.setMatrixAt(i, dummy.matrix);
        i++;
      }
    }
  }
  mesh.instanceMatrix.needsUpdate = true;
  mesh.computeBoundingSphere();

  // add cubelets to scene
  scene.add(mesh);

  // add soft lighting to scene
  scene.add(new THREE.AmbientLight(0xffffff, 5));
}

// Set Opacities from Array

function setOpacities(array) {
  // verify input array has a matching length
  if (array.length != cubeLength * cubeLength * cubeLength) {
    console.error("Array size does not match initialized size");
    return;
  }

  // assign array of opacities to array of densities
  for (let i = 0; i < cubeletCount; i++) {
    opacities[i] = array[i];
  }

  geometry.setAttribute(
    "opacity",
    new THREE.InstancedBufferAttribute(opacities, 1),
  );
  mesh.material.color.set(DENSITY_COLOUR);
}

// Dynamically Resize Container to Window

// Mouse listener
export function onMouseMove(event) {
  // Calculate mouse position in normalized device coordinates
  // (-1 to +1) for both components
  const rect = canvas.getBoundingClientRect();
  mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
  mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
}

// Animation Loop

function animate() {
  requestAnimationFrame(animate);

  // FIX: Only run logic if the scene is ready
  if (initialized && mesh) {
    raycaster.setFromCamera(mouse, camera);
    const intersections = raycaster.intersectObject(mesh);

    if (intersections.length > 0) {
      const validHit = intersections.find(
        (hit) => !galaxies || galaxies[hit.instanceId],
      );

      if (validHit) {
        // STORE THIS ID GLOBALLY so click/keys can use it
        hoveredInstanceId = validHit.instanceId;

        // Move Highlight
        const matrix = new THREE.Matrix4();
        mesh.getMatrixAt(hoveredInstanceId, matrix);
        highlightMesh.position.setFromMatrixPosition(matrix);
        highlightMesh.visible = true;
      } else {
        highlightMesh.visible = false;
        hoveredInstanceId = null;
      }
    } else {
      highlightMesh.visible = false;
      hoveredInstanceId = null;
    }
  }

  // NEW: If we have a selected galaxy, keep the yellow box snapped to it
  // (This handles the visual update when the cube moves)
  if (selectedInstanceId !== null) {
    const matrix = new THREE.Matrix4();
    mesh.getMatrixAt(selectedInstanceId, matrix);
    selectionMesh.position.setFromMatrixPosition(matrix);
    selectionMesh.visible = true;
  }

  controls.update();
  renderer.render(scene, camera);
}

// Convert (x, y, z) into index

function getIndex(x, y, z) {
  return x * cubeLength * cubeLength + y * cubeLength + z;
}

// Get (x, y, z) from index

function getZ(index) {
  return Math.floor(index / (cubeLength * cubeLength)) % cubeLength;
}

function getY(index) {
  return Math.floor(index / cubeLength) % cubeLength;
}

function getX(index) {
  return index % cubeLength;
}

// Generate Decorative Stars

function generateStars() {
  let i = 0;
  const dummy = new THREE.Object3D();
  const starGeometry = new THREE.BoxGeometry(STAR_SIZE, STAR_SIZE, STAR_SIZE);
  const starMaterial = new THREE.MeshStandardMaterial({
    color: STAR_COLOUR,
    transparent: true,
    depthWrite: false,
  });
  const starMesh = new THREE.InstancedMesh(
    starGeometry,
    starMaterial,
    STAR_COUNT,
  );
  const starOpacities = new Float32Array(STAR_COUNT);

  for (let i = 0; i < STAR_COUNT; i++) {
    let sector = Math.floor(Math.random() * 7);
    let x = sector < 5 && sector != 3;
    let y = sector < 2 || sector == 3 || sector == 5;
    let z = (sector < 4 && sector != 1) || sector == 6;
    dummy.position.set(
      Math.random() * STAR_RANGE +
        (x ? (Math.random > 0.5 ? 1 : -1) : 0) * STAR_DISTANCE -
        CUBE_RENDER_SIZE / 2,
      Math.random() * STAR_RANGE +
        (y ? (Math.random > 0.5 ? 1 : -1) : 0) * STAR_DISTANCE -
        CUBE_RENDER_SIZE / 2,
      Math.random() * STAR_RANGE +
        (z ? (Math.random > 0.5 ? 1 : -1) : 0) * STAR_DISTANCE +
        CUBE_RENDER_SIZE / 2,
    );
    dummy.updateMatrix();
    starMesh.setMatrixAt(i, dummy.matrix);
    starOpacities[i] = Math.random();
  }
  starGeometry.setAttribute(
    "opacity",
    new THREE.InstancedBufferAttribute(starOpacities, 1),
  );

  scene.add(starMesh);
}

function showGalaxies() {
  for (let i = 0; i < galaxies.length; i++) {
    if (galaxies[i]) {
      opacities[i] = 1;
    }
  }
  geometry.attributes.opacity.needsUpdate = true;
}
