<script setup lang="ts">
import { ref, watch, nextTick } from "vue"

const props = defineProps<{ telemetry: any }>()

const throttle = () => props.telemetry?.throttle ?? 0
const brake = () => props.telemetry?.brake ?? 0

const maxFrames = 300
const cv = ref<HTMLCanvasElement | null>(null)

// Rolling buffer, drawn on data arrival — no scroll state machine. The only
// state is the buffer itself, so there is nothing left to drift.
const tw = window as any
if (!tw._tbHistory) tw._tbHistory = []
const hist: [number, number][] = tw._tbHistory // [throttle, brake]
watch(() => props.telemetry, () => {
  if (hist.length < maxFrames) {
    // Backfill with the first sample: full-width trace immediately
    const v: [number, number] = [throttle(), brake()]
    while (hist.length < maxFrames) hist.push(v)
  } else {
    hist.push([throttle(), brake()])
    hist.shift()
  }
  if (cv.value?.clientWidth) nextTick(draw)
})

function draw() {
  const c = cv.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  const w = c.clientWidth, h = c.clientHeight
  if (w === 0 || h === 0) return
  // Resize only when needed — assigning width/height reallocates the backing
  // store, wasted work at 60fps
  const bw = Math.round(w * dpr), bh = Math.round(h * dpr)
  if (c.width !== bw || c.height !== bh) { c.width = bw; c.height = bh }
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.fillStyle = "#121314"; ctx.fillRect(0, 0, w, h)

  // Top/bottom padding keeps lines out of the card's 8px rounded corners
  const pad = { t: 8, r: 2, b: 8, l: 2 }
  const pw = w - pad.l - pad.r, ph = h - pad.t - pad.b

  ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.lineWidth = 0.5
  for (let i = 0; i <= 5; i++) {
    const gy = pad.t + ph * i / 5
    ctx.beginPath(); ctx.moveTo(pad.l, gy); ctx.lineTo(w - pad.r, gy); ctx.stroke()
  }

  ctx.save()
  // Horizontal clip at the plot edges: the trace runs past the edges and is
  // cut with its natural slope — no flat hold-stubs at the line ends.
  ctx.beginPath(); ctx.rect(pad.l, pad.t, pw, ph); ctx.clip()
  // Additive blending fuses overlapping red/blue regions instead of
  // hard-covering; round joins/caps avoid corner spikes and clipped strokes.
  ctx.lineJoin = "round"; ctx.lineCap = "round"
  ctx.globalCompositeOperation = "lighter"
  const lw = 1.5
  const dx = pw / maxFrames
  const yOf = (v: number) => pad.t + ph - Math.max(0, Math.min(1, v)) * ph
  const xOf = (i: number) => pad.l + pw - (hist.length - 1 - i) * dx
  const line = (idx: number, color: string) => {
    const n = hist.length
    if (!n) return
    ctx.beginPath()
    ctx.moveTo(xOf(0), yOf(hist[0][idx]))
    for (let i = 1; i < n; i++) {
      ctx.lineTo(xOf(i), yOf(hist[i][idx]))
    }
    ctx.strokeStyle = color; ctx.lineWidth = lw; ctx.stroke()
  }
  line(1, "rgba(241,76,76,0.85)")   // brake
  line(0, "rgba(51,153,255,0.9)")  // throttle
  ctx.restore()
}
</script>

<template>
  <canvas ref="cv" class="tb-scope"></canvas>
</template>

<style scoped>
.tb-scope { width: 100%; height: 100%; display: block; }
</style>
