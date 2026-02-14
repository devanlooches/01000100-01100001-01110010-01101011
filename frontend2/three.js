let initialized = false;

export function initScene(canvasId, containerId) {
  if (initialized) return;

  const THREE = window.THREE;
  const OrbitControls = window.OrbitControls;
  if (!THREE) {
    console.error('THREE not loaded');
    return;
  }
  if (!OrbitControls) {
    console.error('OrbitControls not loaded');
    return;
  }

  const container = document.getElementById(containerId);
  const canvasEl = document.getElementById(canvasId);
  if (!container || !canvasEl) {
    console.error('Could not find container or canvas');
    return;
  }

  initialized = true;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x000000);

  const camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100);
  const size = 5;
  camera.position.set(0, 0, size * 2);

  const renderer = new THREE.WebGLRenderer({ canvas: canvasEl, antialias: true });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.enablePan = false;
  controls.minDistance = size / 3;
  controls.maxDistance = size * 3;
  controls.zoomSpeed = 5.0;

  function generateGrid(n) {
    const cubeSize = size / n;
    const geom = new THREE.BoxGeometry(cubeSize, cubeSize, cubeSize);
    const mat = new THREE.MeshStandardMaterial({
      color: 0xff5500,
      transparent: true,
      depthWrite: false,
    });

    mat.onBeforeCompile = (shader) => {
      shader.vertexShader = `
        attribute float instanceAlpha;
        varying float vInstanceAlpha;
        ${shader.vertexShader}
      `.replace(
        `void main() {`,
        `void main() {
          vInstanceAlpha = instanceAlpha;`
      );

      shader.fragmentShader = `
        varying float vInstanceAlpha;
        ${shader.fragmentShader}
      `.replace(
        `#include <opaque_fragment>`,
        `#include <opaque_fragment>
          gl_FragColor.a *= vInstanceAlpha;`
      );
    };

    const count = n * n * n;
    const alphas = new Float32Array(count);
    for (let i = 0; i < count; i++) {
      alphas[i] = Math.random() / 20;
    }

    geom.setAttribute('instanceAlpha', new THREE.InstancedBufferAttribute(alphas, 1));

    const mesh = new THREE.InstancedMesh(geom, mat, count);
    mesh.castShadow = true;
    mesh.receiveShadow = true;

    const dummy = new THREE.Object3D();
    const color = new THREE.Color();

    let i = 0;
    for (let x = 0; x < n; x++) {
      for (let y = 0; y < n; y++) {
        for (let z = 0; z < n; z++) {
          dummy.position.set(
            (cubeSize * (x + 0.5)) - (size / 2),
            (cubeSize * (y + 0.5)) - (size / 2),
            (cubeSize * (z + 0.5)) - (size / 2)
          );
          dummy.updateMatrix();
          mesh.setMatrixAt(i, dummy.matrix);

          color.setHex(0xffffff);
          mesh.setColorAt(i, color);

          i++;
        }
      }
    }
    scene.add(mesh);
    scene.add(new THREE.AmbientLight(0xffffff, 0.5));
  }

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
    controls.update();
    renderer.render(scene, camera);
  }

  generateGrid(64);
  new ResizeObserver(resize).observe(container);
}

export function listenForKey(key, callback) {
  window.addEventListener('keydown', (ev) => {
    if (ev.key === key) {
      callback();
    }
  });
}
