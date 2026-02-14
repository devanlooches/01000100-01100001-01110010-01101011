let initialized = false;

export function initScene(canvasId, containerId) {
    if (initialized) return;

    const THREE = window.THREE;
    if (!THREE) {
        console.error('THREE not loaded');
        return;
    }

    const container = document.getElementById(containerId);
    const canvasEl = document.getElementById(canvasId);
    if (!container || !canvasEl) {
        console.error('Could not find container or canvas');
        return;
    }

    initialized = true;

    const renderer = new THREE.WebGLRenderer({ canvas: canvasEl, antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
    camera.position.z = 3;

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshNormalMaterial();
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    let animating = false;

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

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    new ResizeObserver(resize).observe(container);
}

export function listenForKey(key, callback) {
    window.addEventListener('keydown', (ev) => {
        if (ev.key === key) {
            callback();
        }
    });
}
