// Imports

const THREE = window.THREE;
if (!THREE) {
    console.error('THREE not loaded');
}

const OrbitControls = window.OrbitControls;
if (!OrbitControls) {
    console.error('OrbitControls not loaded');
}

// Config

const BG_COLOUR = new THREE.Color(0x000000);
const CUBE_COLOUR = new THREE.Color(0xff5500);
const CUBE_OPACITY_COEFFICIENT = 0.05;
const CUBE_RENDER_SIZE = 5;
const MIN_ZOOM = 1;
const MAX_ZOOM = 15;
const DEFUALT_ZOOM = 10;
const ZOOM_SPEED = 5;
const CAMERA_DAMPING = 0.05;
const DENSITY_MAX = 1000;

// Global Variables

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

// Public Functions

// Initialize 

export function initScene(canvasId, containerId) {

    if (initialized) return;

    // initialize container & canvas
    container = document.getElementById(containerId);
    canvas = document.getElementById(canvasId);
    if (!container || !canvas) {
        console.error('Could not find container or canvas');
        return;
    }

    // initialize scene
    scene = new THREE.Scene();
    scene.background = BG_COLOUR;

    // initialize camera
    camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100);
    camera.position.set(0, 0, DEFUALT_ZOOM);

    // initialize renderer
    renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;

    // initialize controls
    controls = new OrbitControls(camera, renderer.domElement);
    controls.enablePan = false;
    controls.enableDamping = true;
    controls.dampingFactor = CAMERA_DAMPING;
    controls.minDistance = MIN_ZOOM;
    controls.maxDistance = MAX_ZOOM;
    controls.zoomSpeed = ZOOM_SPEED;

    // initialize cube & cubelets with random opacities
    initializeCube(64);
    randomizeOpacities();

    initialized = true;
    animating = false;

    // watches for window being resized during runtime
    new ResizeObserver(resize).observe(container);
}

// Randomize Opacities

export function randomizeOpacities() {

    for (let i = 0; i < cubeletCount; i++) {
        opacities[i] = Math.random() * CUBE_OPACITY_COEFFICIENT;
    }
    geometry.setAttribute('opacity', new THREE.InstancedBufferAttribute(opacities, 1));
}

// Set Opacities from Density Array

export function setOpacitiesFromDensities(array) {

    // Convert all densities to opacities
    for (let i = 0; i < array.length; i++) {
        array[i] = array[i] * CUBE_OPACITY_COEFFICIENT / DENSITY_MAX;
    }
    setOpacities(array);
}

// Keypress Listener

export function listenForKey(key, callback) {
    window.addEventListener('keydown', (ev) => {
        if (ev.key === key) {
            callback();
        }
    });
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
    let mat = new THREE.MeshStandardMaterial({
        color: CUBE_COLOUR,
        transparent: true,
        depthWrite: false,
    });


    // apply custom material shader to allow for opacity changes
    mat.onBeforeCompile = (shader) => {
        shader.vertexShader = `
        attribute float opacity;
        varying float vOpacity;
        ${shader.vertexShader}
      `.replace(
            `void main() {`,
            `void main() {
          vOpacity = opacity;`
        );
        shader.fragmentShader = `
        varying float vOpacity;
        ${shader.fragmentShader}
      `.replace(
            `#include <opaque_fragment>`,
            `#include <opaque_fragment>
          gl_FragColor.a *= vOpacity;`
        );
    };

    // initialize opacities
    geometry.setAttribute('opacity', new THREE.InstancedBufferAttribute(opacities, 1));

    // generate mesh of cubelets
    const mesh = new THREE.InstancedMesh(geometry, mat, cubeletCount);

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
                    (cubeletSize * (x + 0.5)) - (CUBE_RENDER_SIZE / 2),
                    (cubeletSize * (y + 0.5)) - (CUBE_RENDER_SIZE / 2),
                    (cubeletSize * (z + 0.5)) - (CUBE_RENDER_SIZE / 2)
                );
                // update position matrix
                dummy.updateMatrix();
                // push position matrix to cubelet
                mesh.setMatrixAt(i, dummy.matrix);
                i++;
            }
        }
    }

    // add cubelets to scene
    scene.add(mesh);

    // add soft lighting to scene
    scene.add(new THREE.AmbientLight(0xffffff, 0.5));
}

// Set Opacities from Array

function setOpacities(array) {

    // verify input array has a matching length
    if (array.length != cubeLength * cubeLength * cubeLength) {
        console.error('Array size does not match initialized size');
        return;
    }

    // assign array of opacities to array of densities
    for (let i = 0; i < cubeletCount; i++) {
        opacities[i] = array[i];
    }

    geometry.setAttribute('opacity', new THREE.InstancedBufferAttribute(opacities, 1));
}

// Dynamically Resize Container to Window

function resize() {

    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w === 0 || h === 0) return;

    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);

    if (!animating) {
        animating = true;
        animate();
    }

}

// Animation Loop

function animate() {
    requestAnimationFrame(animate);
    controls.update();
    renderer.render(scene, camera);
}