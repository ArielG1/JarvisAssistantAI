<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import { useHudStore } from "@/stores/hud"
import { STATUS_COLORS } from "@/types/status"

const canvas = ref<HTMLCanvasElement | null>(null)
const store = useHudStore()
let animationId = 0

interface Particle {
  x: number
  y: number
  vx: number
  vy: number
  size: number
  opacity: number
}

function hexToRgb(hex: string) {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return { r, g, b }
}

onMounted(() => {
  const el = canvas.value
  if (!el) return

  const ctx = el.getContext("2d")
  if (!ctx) return

  function resize() {
    el!.width = window.innerWidth
    el!.height = window.innerHeight
  }
  resize()
  window.addEventListener("resize", resize)

  const PARTICLE_COUNT = 65
  const CONNECTION_DIST = 120
  const particles: Particle[] = []

  for (let i = 0; i < PARTICLE_COUNT; i++) {
    particles.push({
      x: Math.random() * el!.width,
      y: Math.random() * el!.height,
      vx: (Math.random() - 0.5) * 0.4,
      vy: (Math.random() - 0.5) * 0.4,
      size: Math.random() * 2 + 1,
      opacity: Math.random() * 0.5 + 0.2,
    })
  }

  function animate() {
    if (!ctx || !el) return
    ctx.clearRect(0, 0, el.width, el.height)

    const color = STATUS_COLORS[store.currentState]
    const rgb = hexToRgb(color)

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i]
      p.x += p.vx
      p.y += p.vy

      if (p.x < 0) p.x = el.width
      if (p.x > el.width) p.x = 0
      if (p.y < 0) p.y = el.height
      if (p.y > el.height) p.y = 0

      ctx.beginPath()
      ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2)
      ctx.fillStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${p.opacity})`
      ctx.fill()

      for (let j = i + 1; j < particles.length; j++) {
        const q = particles[j]
        const dx = p.x - q.x
        const dy = p.y - q.y
        const dist = Math.sqrt(dx * dx + dy * dy)

        if (dist < CONNECTION_DIST) {
          const alpha = (1 - dist / CONNECTION_DIST) * 0.3
          ctx.beginPath()
          ctx.moveTo(p.x, p.y)
          ctx.lineTo(q.x, q.y)
          ctx.strokeStyle = `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`
          ctx.lineWidth = 0.5
          ctx.stroke()
        }
      }
    }

    animationId = requestAnimationFrame(animate)
  }

  animate()

  onUnmounted(() => {
    cancelAnimationFrame(animationId)
    window.removeEventListener("resize", resize)
  })
})
</script>

<template>
  <canvas
    ref="canvas"
    class="fixed inset-0 w-full h-full pointer-events-none"
    style="z-index: 0"
  />
</template>
