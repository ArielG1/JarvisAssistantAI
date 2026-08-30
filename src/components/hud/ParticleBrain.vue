<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from "vue"
import * as THREE from "three"
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js"
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js"
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js"
import { useHudStore } from "@/stores/hud"
import { STATUS_COLORS } from "@/types/status"

const hud = useHudStore()
const container = ref<HTMLDivElement | null>(null)
let raf = 0
let renderer: THREE.WebGLRenderer
let composer: EffectComposer | null = null
let bloomPass: UnrealBloomPass | null = null

function hexNumber(hex: string): number {
  return parseInt(hex.replace("#", ""), 16)
}

function angDiff(a: number, b: number) {
  let d = a - b
  while (d > Math.PI) d -= 2 * Math.PI
  while (d < -Math.PI) d += 2 * Math.PI
  return d
}
function bump(theta: number, center: number, width: number, amount: number) {
  const d = angDiff(theta, center)
  return amount * Math.exp(-(d * d) / (2 * width * width))
}
function gauss(x: number, center: number, width: number, amount: number) {
  const d = x - center
  return amount * Math.exp(-(d * d) / (2 * width * width))
}
function randomDir() {
  const u = Math.random()
  const v = Math.random()
  const theta = 2 * Math.PI * u
  const phi = Math.acos(2 * v - 1)
  return {
    dx: Math.sin(phi) * Math.cos(theta),
    dy: Math.cos(phi),
    dz: Math.sin(phi) * Math.sin(theta),
  }
}
function brainRadiusDir(dx: number, dy: number, dz: number) {
  const theta = Math.atan2(dz, dx)
  let r =
    1 +
    0.1 * Math.sin(4 * theta + 0.3) +
    0.06 * Math.sin(9 * theta + 1.2) +
    0.04 * Math.sin(15 * theta + 2.0) +
    0.05 * Math.sin(6 * dy * 3.0 + theta)
  r += bump(theta, 0.9, 0.4, 0.16) * Math.max(0, -dy + 0.3)
  r += bump(theta, 2.35, 0.35, 0.1) * Math.max(0, -dy + 0.3)
  r -= gauss(dy, -0.92, 0.1, 0.6)
  const nearMidplane = Math.exp(-(dz * dz) / (2 * 0.05 * 0.05))
  r -= nearMidplane * Math.max(0, dy) * 0.1
  return Math.max(r, 0.2)
}
const RX = 1.55, RY = 1.05, RZ = 1.0
function brainPoint(shell: number) {
  const { dx, dy, dz } = randomDir()
  const r = brainRadiusDir(dx, dy, dz) * shell
  return new THREE.Vector3(dx * r * RX, dy * r * RY, dz * r * RZ)
}
function makeGlowTexture() {
  const c = document.createElement("canvas")
  c.width = c.height = 128
  const g = c.getContext("2d")!
  const grad = g.createRadialGradient(64, 64, 0, 64, 64, 64)
  grad.addColorStop(0, "rgba(255,255,255,1)")
  grad.addColorStop(0.35, "rgba(255,255,255,0.6)")
  grad.addColorStop(1, "rgba(255,255,255,0)")
  g.fillStyle = grad
  g.fillRect(0, 0, 128, 128)
  return new THREE.CanvasTexture(c)
}

onMounted(() => {
  const el = container.value!
  const scene = new THREE.Scene()
  const camera = new THREE.PerspectiveCamera(45, el.clientWidth / el.clientHeight, 0.1, 100)
  camera.position.set(0, 0, 6.4)

  renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.setSize(el.clientWidth, el.clientHeight)
  renderer.setClearColor(0x000000, 0)
  el.appendChild(renderer.domElement)

  try {
    composer = new EffectComposer(renderer)
    composer.addPass(new RenderPass(scene, camera))
    bloomPass = new UnrealBloomPass(new THREE.Vector2(el.clientWidth, el.clientHeight), 1.15, 0.75, 0.12)
    composer.addPass(bloomPass)
  } catch {
    composer = null
    bloomPass = null
  }

  const brainGroup = new THREE.Group()
  scene.add(brainGroup)

  const glowTex = makeGlowTexture()

  const CORE_N = 500
  const corePositions: THREE.Vector3[] = []
  const corePhase = new Float32Array(CORE_N * 3)
  for (let i = 0; i < CORE_N; i++) {
    corePositions.push(brainPoint(0.92 + Math.random() * 0.08))
    corePhase[i * 3] = Math.random() * Math.PI * 2
    corePhase[i * 3 + 1] = Math.random() * Math.PI * 2
    corePhase[i * 3 + 2] = Math.random() * Math.PI * 2
  }
  const coreGeo = new THREE.BufferGeometry().setFromPoints(corePositions)
  const coreMat = new THREE.PointsMaterial({
    map: glowTex,
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    size: 0.09,
    transparent: true,
    opacity: 0.95,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  })
  brainGroup.add(new THREE.Points(coreGeo, coreMat))

  const THRESH = 0.24
  const linePairs: number[] = []
  for (let i = 0; i < CORE_N; i++) {
    for (let j = i + 1; j < CORE_N; j++) {
      if (corePositions[i].distanceTo(corePositions[j]) < THRESH) linePairs.push(i, j)
    }
  }
  const lineGeo = new THREE.BufferGeometry()
  const linePosArray = new Float32Array(linePairs.length * 3)
  for (let k = 0; k < linePairs.length; k += 2) {
    const i = linePairs[k], j = linePairs[k + 1]
    const base = k * 3
    linePosArray[base] = corePositions[i].x
    linePosArray[base + 1] = corePositions[i].y
    linePosArray[base + 2] = corePositions[i].z
    linePosArray[base + 3] = corePositions[j].x
    linePosArray[base + 4] = corePositions[j].y
    linePosArray[base + 5] = corePositions[j].z
  }
  lineGeo.setAttribute("position", new THREE.BufferAttribute(linePosArray, 3))
  const lineMat = new THREE.LineBasicMaterial({
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    transparent: true,
    opacity: 0.18,
    blending: THREE.AdditiveBlending,
  })
  brainGroup.add(new THREE.LineSegments(lineGeo, lineMat))

  const DUST_N = 1800
  const dustPositions: THREE.Vector3[] = []
  const dustPhase = new Float32Array(DUST_N * 3)
  for (let i = 0; i < DUST_N; i++) {
    dustPositions.push(brainPoint(0.5 + Math.random() * 0.9))
    dustPhase[i * 3] = Math.random() * Math.PI * 2
    dustPhase[i * 3 + 1] = Math.random() * Math.PI * 2
    dustPhase[i * 3 + 2] = Math.random() * Math.PI * 2
  }
  const dustGeo = new THREE.BufferGeometry().setFromPoints(dustPositions)
  const dustMat = new THREE.PointsMaterial({
    map: glowTex,
    color: hexNumber(STATUS_COLORS[hud.currentState]),
    size: 0.03,
    transparent: true,
    opacity: 0.5,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  })
  brainGroup.add(new THREE.Points(dustGeo, dustMat))

  scene.add(new THREE.AmbientLight(0xffffff, 0.2))

  let autoRotate = true
  let isDragging = false, lastX = 0, lastY = 0, rotVelY = 0.0025
  el.addEventListener("pointerdown", (e) => {
    isDragging = true
    autoRotate = false
    lastX = e.clientX
    lastY = e.clientY
  })
  window.addEventListener("pointerup", () => { isDragging = false })
  window.addEventListener("pointermove", (e) => {
    if (!isDragging) return
    const dx = e.clientX - lastX, dy = e.clientY - lastY
    brainGroup.rotation.y += dx * 0.005
    brainGroup.rotation.x += dy * 0.005
    rotVelY = dx * 0.0006
    lastX = e.clientX
    lastY = e.clientY
  })

  function onResize() {
    camera.aspect = el.clientWidth / el.clientHeight
    camera.updateProjectionMatrix()
    renderer.setSize(el.clientWidth, el.clientHeight)
    composer?.setSize(el.clientWidth, el.clientHeight)
  }
  window.addEventListener("resize", onResize)

  let tBrain = 0
  let lastStateSeen = hud.currentState
  let turbulence = 0
  function updateStateColors() {
    const c = hexNumber(STATUS_COLORS[hud.currentState])
    coreMat.color.setHex(c)
    lineMat.color.setHex(c)
    dustMat.color.setHex(c)
  }

  function animate() {
    raf = requestAnimationFrame(animate)
    tBrain += 0.016
    if (lastStateSeen !== hud.currentState) {
      updateStateColors()
      lastStateSeen = hud.currentState
    }

    if (autoRotate) brainGroup.rotation.y += 0.0022
    else if (!isDragging) {
      brainGroup.rotation.y += rotVelY
      rotVelY *= 0.96
      if (Math.abs(rotVelY) < 0.0004) autoRotate = true
    }

    const sinceBurst = performance.now() - hud.lastChangeAt
    const burst = sinceBurst < 700 ? 1 - sinceBurst / 700 : 0
    const burstEase = burst * burst

    const breathe = 0.85 + 0.15 * Math.sin(tBrain * 0.6) + burstEase * 0.5
    coreMat.opacity = Math.min(1, 0.75 * breathe + 0.15)
    coreMat.size = 0.09 + burstEase * 0.05
    lineMat.opacity = Math.min(0.9, 0.12 * breathe + 0.05 + burstEase * 0.35)
    dustMat.opacity = Math.min(1, 0.5 + burstEase * 0.4)
    brainGroup.scale.setScalar(1 + burstEase * 0.12)
    if (bloomPass) bloomPass.strength = 1.15 + burstEase * 1.4

    const wantTurbulence = hud.currentState === "pensando" || hud.currentState === "trabajando" ? 1 : 0
    turbulence += (wantTurbulence - turbulence) * 0.05
    const turbAmp = hud.currentState === "trabajando" ? 0.16 : 0.11

    if (turbulence > 0.004) {
      const corePos = coreGeo.attributes.position.array as Float32Array
      for (let i = 0; i < CORE_N; i++) {
        const base = corePositions[i]
        const amp = turbAmp * turbulence
        corePos[i * 3] = base.x + amp * Math.sin(tBrain * 2.3 + corePhase[i * 3])
        corePos[i * 3 + 1] = base.y + amp * Math.sin(tBrain * 2.7 + corePhase[i * 3 + 1])
        corePos[i * 3 + 2] = base.z + amp * Math.sin(tBrain * 3.1 + corePhase[i * 3 + 2])
      }
      coreGeo.attributes.position.needsUpdate = true

      const dustPos = dustGeo.attributes.position.array as Float32Array
      for (let i = 0; i < DUST_N; i++) {
        const base = dustPositions[i]
        const amp = turbAmp * 0.6 * turbulence
        dustPos[i * 3] = base.x + amp * Math.sin(tBrain * 2.1 + dustPhase[i * 3])
        dustPos[i * 3 + 1] = base.y + amp * Math.sin(tBrain * 2.5 + dustPhase[i * 3 + 1])
        dustPos[i * 3 + 2] = base.z + amp * Math.sin(tBrain * 2.9 + dustPhase[i * 3 + 2])
      }
      dustGeo.attributes.position.needsUpdate = true

      const linePos = lineGeo.attributes.position.array as Float32Array
      for (let k = 0; k < linePairs.length; k += 2) {
        const i = linePairs[k], j = linePairs[k + 1]
        const base = k * 3
        linePos[base] = corePos[i * 3]
        linePos[base + 1] = corePos[i * 3 + 1]
        linePos[base + 2] = corePos[i * 3 + 2]
        linePos[base + 3] = corePos[j * 3]
        linePos[base + 4] = corePos[j * 3 + 1]
        linePos[base + 5] = corePos[j * 3 + 2]
      }
      lineGeo.attributes.position.needsUpdate = true
    }

    if (composer) composer.render()
    else renderer.render(scene, camera)
  }
  animate()

  onBeforeUnmount(() => {
    cancelAnimationFrame(raf)
    window.removeEventListener("resize", onResize)
    coreGeo.dispose()
    lineGeo.dispose()
    dustGeo.dispose()
    coreMat.dispose()
    lineMat.dispose()
    dustMat.dispose()
    renderer.dispose()
    el.removeChild(renderer.domElement)
  })
})
</script>

<template>
  <div ref="container" class="particle-brain" />
</template>

<style scoped>
.particle-brain {
  width: 100%;
  height: 100%;
  cursor: grab;
}
.particle-brain:active {
  cursor: grabbing;
}
</style>
