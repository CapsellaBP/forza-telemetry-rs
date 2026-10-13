<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch, nextTick } from "vue"
import HudPreview from "../components/HudPreview.vue"
import ThrottleBrakeScope from "../components/ThrottleBrakeScope.vue"
import { usePressGestures } from "../composables/usePressGestures"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ action: [action: string] }>()

const s = (key: string, fb: any = null) => props.telemetry?.settings?.[key] ?? fb

const isLocked = computed(() => !!props.telemetry?.curve?.locked)
const statusText = computed(() => {
  if (isLocked.value) return "已锁定"
  if (props.telemetry?.shift_advice?.ready) return "就绪"
  return "采样中"
})
const statusColor = computed(() => {
  if (isLocked.value) return "var(--orange)"
  if (props.telemetry?.shift_advice?.ready) return "#3399ff"
  return "#3399ff"
})
function toggleLock() {
  emit('action', isLocked.value ? 'unlock_curve' : 'lock_curve')
}
const { pressing, onPointerdown, onPointerup, onPointercancel, onDblclick } = usePressGestures({
  onLongPress: toggleLock,
  onDoubleClick: () => emit('action', 'reset_curve'),
})

// 总览页设计稿 1162×762: 窗口小于设计稿时 zoom 等比缩小 (比例不动);
// 大于设计稿时 zoom 锁 ZOOM_MAX, 余量由 dashWidth/dashHeight 浮动吸收, 居中显示
const ZOOM_MIN = 0.7, ZOOM_MAX = 1.0, BASE_W = 1162, BASE_H = 762, MAX_W = 1280
const MAX_H = 810        // 高度浮动上限: 两个行间垂直间距 10 → 最多各 +24
const ROW_GAP_FL = 0.12  // 行内左右间距浮动系数: 每像素宽度余量中间距分 0.12
const viewport = ref<HTMLElement | null>(null)
const dashZoom = ref(1)
const dashWidth = ref(BASE_W)
const dashHeight = ref(BASE_H)
const gapX = ref(0)  // --hfl: 行内间距浮动量 (行1/行2)
const gapY = ref(0)  // --vfl: 行间垂直间距浮动量
let ro: ResizeObserver | null = null
onMounted(() => {
  const el = viewport.value; if (!el) return
  const update = () => {
    if (!el.clientWidth) return   // v-show 隐藏时保持上次缩放值
    // 2px safety: sizing content exactly equal to the viewport is metastable —
    // zoom sub-pixel rounding can flip overflow on/off, toggling the scrollbar
    // (~17px layout shift) every frame = whole-panel double-image flicker.
    const cw = el.clientWidth - 2, ch = el.clientHeight - 2
    const z = Math.max(ZOOM_MIN, Math.min(cw / BASE_W, ch / BASE_H, ZOOM_MAX))
    dashZoom.value = z
    if (z >= ZOOM_MAX) {
      dashWidth.value = Math.min(MAX_W, cw / z)
      dashHeight.value = Math.min(MAX_H, ch / z)
      gapX.value = (dashWidth.value - BASE_W) * ROW_GAP_FL
      gapY.value = (dashHeight.value - BASE_H) / 2   // 两个行间距平分
    } else {
      dashWidth.value = BASE_W; dashHeight.value = BASE_H; gapX.value = 0; gapY.value = 0
    }
  }
  ro = new ResizeObserver(update)
  ro.observe(el)
  update()
})
onUnmounted(() => ro?.disconnect())

const throttle = computed(() => (props.telemetry?.throttle ?? 0) * 100)
const brake = computed(() => (props.telemetry?.brake ?? 0) * 100)
const gLon = computed(() => props.telemetry?.accel_lon ?? 0)
const gLat = computed(() => props.telemetry?.accel_lat ?? 0)
const tires = computed(() => props.telemetry?.tire_temp ?? [0,0,0,0])
const slipVals = computed(() => props.telemetry?.tire_slip ?? [0,0,0,0])
const susp = computed(() => props.telemetry?.susp_travel ?? [0,0,0,0])
const boost = computed(() => props.telemetry?.boost_psi ?? 0)
const speed = computed(() => props.telemetry?.speed_kmh?.toFixed(0) ?? "0")

function tempColor(t: number): string {
  if (t <= 0) return "#555"; if (t < 60) return "#4a90d9"; if (t < 80) return "#58a6ff"
  if (t < 95) return "#4ec46b"; if (t < 110) return "#d4a843"; return "#f14c4c"
}
function slipColor(v: number): string {
  const s = props.telemetry?.settings
  const a = Math.abs(v)
  if (a >= (s?.slip_danger ?? 2.0)) return "#f14c4c"
  if (a >= (s?.slip_warn ?? 0.5)) return "#d4a843"
  return "#4ec46b"
}

const suspOrder = [2, 0, 1, 3]
const suspNames = ["RL","FL","FR","RR"]
function travelColor(v: number): string {
  const s = props.telemetry?.settings
  if (v >= (s?.susp_thr3 ?? 0.80)) return "#f14c4c"
  if (v >= (s?.susp_thr2 ?? 0.55)) return "#d4a843"
  if (v >= (s?.susp_thr1 ?? 0.30)) return "#3399ff"
  return "#4ec46b"
}

// G-G
const ggCanvas = ref<HTMLCanvasElement | null>(null)
const SZ = 250
function drawGG() {
  const c = ggCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  c.style.width = SZ + "px"; c.style.height = SZ + "px"
  // 位图尺寸走 getBoundingClientRect: 含 CSS zoom (总览页等比缩放器), SZ 固定值不含
  const rect = c.getBoundingClientRect()
  if (rect.width === 0) return
  const bw = Math.round(rect.width * dpr)
  if (c.width !== bw || c.height !== bw) { c.width = bw; c.height = bw }
  const ctx = c.getContext("2d")!
  const eff = rect.width / SZ * dpr
  ctx.setTransform(eff, 0, 0, eff, 0, 0)
  const cx = SZ / 2, cy = SZ / 2, r = SZ / 2 - 32

  ctx.clearRect(0, 0, SZ, SZ)
  ctx.beginPath(); ctx.arc(cx, cy, r, 0, Math.PI * 2)
  ctx.fillStyle = "rgba(255,255,255,0.02)"; ctx.fill()
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1; ctx.stroke()
  ctx.strokeStyle = "rgba(255,255,255,0.06)"; ctx.lineWidth = 0.5
  ctx.beginPath(); ctx.moveTo(cx - r, cy); ctx.lineTo(cx + r, cy); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(cx, cy - r); ctx.lineTo(cx, cy + r); ctx.stroke()
  for (const p of [0.5, 1.0]) { ctx.beginPath(); ctx.arc(cx, cy, r * p, 0, Math.PI * 2); ctx.strokeStyle = "rgba(255,255,255,0.05)"; ctx.stroke() }

  const scale = r / 3.0
  const dx = Math.max(-3.0, Math.min(3.0, gLat.value)) * scale
  const dy = Math.max(-3.0, Math.min(3.0, -gLon.value)) * scale
  // Trail
  const tw = window as any; if (!tw._ggTrail) tw._ggTrail = []
  const t = tw._ggTrail; t.push({ x: cx+dx, y: cy+dy }); if (t.length > 12) t.shift()
  for (let i = 0; i < t.length; i++) {
    const a = (i/t.length) * 0.2
    ctx.beginPath(); ctx.arc(t[i].x, t[i].y, 2+(i/t.length)*2, 0, Math.PI*2)
    ctx.fillStyle = `rgba(212,168,67,${a})`; ctx.fill()
  }
  // Tether + point
  if (t.length > 0) {
    const l = t[t.length-1]
    ctx.strokeStyle = "rgba(212,168,67,0.25)"; ctx.lineWidth = 1
    ctx.beginPath(); ctx.moveTo(cx, cy); ctx.lineTo(l.x, l.y); ctx.stroke()
  }
  ctx.beginPath(); ctx.arc(cx + dx, cy + dy, 5, 0, Math.PI * 2)
  ctx.fillStyle = "#d4a843"; ctx.fill()

  const fwd = gLon.value > 0 ? gLon.value.toFixed(2) : "0.00"
  const brk = gLon.value < 0 ? Math.abs(gLon.value).toFixed(2) : "0.00"
  const lg = gLat.value < 0 ? Math.abs(gLat.value).toFixed(2) : "0.00"
  const rg = gLat.value > 0 ? gLat.value.toFixed(2) : "0.00"

  ctx.fillStyle = "#8b8f93"; ctx.font = "bold 12px sans-serif"
  ctx.textAlign = "center"
  ctx.fillText(fwd, cx, 16)
  ctx.fillText(brk, cx, SZ - 6)
  ctx.textAlign = "right"
  ctx.fillText(lg, 28, cy + 5)
  ctx.textAlign = "left"
  ctx.fillText(rg, SZ - 28, cy + 5)

  const rawG = Math.sqrt(gLon.value*gLon.value + gLat.value*gLat.value)
  // Window-persistent EMA for total G
  const w = window as any; if (!w._gTotalEma) w._gTotalEma = rawG
  const alpha = props.telemetry?.settings?.g_total_ema_alpha ?? 0.05
  w._gTotalEma += (rawG - w._gTotalEma) * alpha
  const totalG = w._gTotalEma
  ctx.fillStyle = "#fff"; ctx.font = "bold 16px sans-serif"; ctx.textAlign = "right"
  ctx.fillText(totalG.toFixed(2), cx + r, cy - r + 16)
}
watch([gLon, gLat], () => { if (!ggCanvas.value?.clientWidth) return; nextTick(drawGG) })
onMounted(() => nextTick(drawGG))

// Power
const pwCanvas = ref<HTMLCanvasElement | null>(null)
function drawPower() {
  const c = pwCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  const w = c.clientWidth, h = c.clientHeight
  if (w === 0 || h === 0) return
  // 位图尺寸走 getBoundingClientRect: 含 CSS zoom (总览页等比缩放器), clientWidth 不含
  const rect = c.getBoundingClientRect()
  const eff = rect.width / w * dpr
  const bw = Math.round(rect.width * dpr), bh = Math.round(rect.height * dpr)
  if (c.width !== bw || c.height !== bh) { c.width = bw; c.height = bh }
  const ctx = c.getContext("2d")!
  ctx.setTransform(eff, 0, 0, eff, 0, 0)
  ctx.fillStyle = "#121314"; ctx.fillRect(0, 0, w, h)
  const pad = { t: 8, r: 55, b: 18, l: 32 }, pw = w - pad.l - pad.r, ph = h - pad.t - pad.b
  const curve = props.telemetry?.curve
  const tq: [number,number][] = curve?.torque || [], pk: [number,number][] = curve?.power_kw || []
  let rLo = 0, rHi = props.telemetry?.rpm_max || 9000, tM = 500, kM = 300
  for (const [r,v] of tq) { rLo = Math.min(rLo, r); rHi = Math.max(rHi, r); tM = Math.max(tM, v) }
  for (const [,v] of pk) kM = Math.max(kM, v)
  if (rLo === 0 && pk.length === 0) { rLo = 1500; rHi = 9000; tM = 500; kM = 300 }
  const pR = (rHi - rLo) * 0.05 || 200; rLo = Math.max(0, rLo - pR); rHi += pR; tM *= 1.1; kM *= 1.1
  if (tM < 200) tM = 500; if (kM < 100) kM = 300
  const hM = kM * 1.341
  const x = (r: number) => pad.l + (r - rLo) / (rHi - rLo) * pw
  const yt = (t: number) => pad.t + ph - (t / tM) * ph
  const yp = (p: number) => pad.t + ph - (p / kM) * ph
  ctx.strokeStyle = "rgba(255,255,255,0.04)"; ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) { const gy = pad.t + ph * i / 4; ctx.beginPath(); ctx.moveTo(pad.l, gy); ctx.lineTo(w - pad.r, gy); ctx.stroke() }
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t); ctx.lineTo(pad.l, pad.t + ph); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t + ph); ctx.lineTo(w - pad.r, pad.t + ph); ctx.stroke()
  ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"; ctx.textAlign = "center"
  for (let r = Math.ceil(rLo / 1000) * 1000; r <= rHi; r += 1000) ctx.fillText((r / 1000).toFixed(0) + "k", x(r), pad.t + ph + 13)
  // Y axis labels (Nm left, hp right)
  ctx.textAlign = "right"
  ctx.fillText(tM.toFixed(0), pad.l - 4, pad.t + 8)
  ctx.fillText((tM/2).toFixed(0), pad.l - 4, pad.t + ph/2 + 4)
  ctx.fillText("0", pad.l - 4, pad.t + ph + 4)
  ctx.fillStyle = "#58a6ff"; ctx.font = "bold 8px sans-serif"; ctx.fillText("Nm", pad.l - 4, pad.t - 2)
  ctx.textAlign = "left"; ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"
  ctx.fillText(hM.toFixed(0), w - pad.r + 3, pad.t + 8)
  ctx.fillText((hM/2).toFixed(0), w - pad.r + 3, pad.t + ph/2 + 4)
  ctx.fillText("0", w - pad.r + 3, pad.t + ph + 4)
  ctx.fillStyle = "#f85149"; ctx.font = "bold 8px sans-serif"
  ctx.fillText("hp", w - pad.r + 3, pad.t - 2)

  if (tq.length > 1) { ctx.strokeStyle = "#58a6ff"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < tq.length; i++) i === 0 ? ctx.moveTo(x(tq[i][0]), yt(tq[i][1])) : ctx.lineTo(x(tq[i][0]), yt(tq[i][1])); ctx.stroke() }
  if (pk.length > 1) { ctx.setLineDash([3, 3]); ctx.strokeStyle = "#f85149"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < pk.length; i++) i === 0 ? ctx.moveTo(x(pk[i][0]), yp(pk[i][1])) : ctx.lineTo(x(pk[i][0]), yp(pk[i][1])); ctx.stroke(); ctx.setLineDash([]) }
  const sr = props.telemetry?.shift_advice?.shift_rpm
  if (sr > 0) { ctx.strokeStyle = "#d4a843"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(sr), pad.t); ctx.lineTo(x(sr), pad.t + ph); ctx.stroke() }
  const cr = props.telemetry?.rpm
  if (cr > 0) { ctx.setLineDash([2, 4]); ctx.strokeStyle = "#ccc"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(cr), pad.t); ctx.lineTo(x(cr), pad.t + ph); ctx.stroke(); ctx.setLineDash([]) }
}
// pkt_count changes every broadcast — a shallow signal avoids the deep
// traversal of the whole telemetry object (settings/curves) at 60Hz
watch(() => props.telemetry?.pkt_count, () => { if (!pwCanvas.value?.clientWidth) return; nextTick(drawPower) })
onMounted(() => nextTick(drawPower))
// zoom 变化后 canvas 位图尺寸需立即跟进, 否则等下一帧数据期间发虚
watch(dashZoom, () => nextTick(() => { drawPower(); drawGG() }))
</script>

<template>
  <div class="dash-viewport" ref="viewport">
  <div class="dashboard" :style="{ zoom: dashZoom, width: dashWidth + 'px', height: dashHeight + 'px', '--hfl': gapX + 'px', '--vfl': gapY + 'px' }">
    <!-- Row 1: scope | label/brake/throttle/label | buttons | HUD -->
    <div class="top-row">
      <div class="scope-col"><ThrottleBrakeScope :telemetry="telemetry" /></div>

      <!-- TB: 字符/刹车/油门/字符 -->
      <div class="tb-col">
        <div class="tb-side"><span class="tb-lbl">刹车</span><span class="tb-pct">{{ brake.toFixed(0) }}%</span></div>
        <div class="tb-wrap brk"><div class="tb-fill" :style="{ height: brake + '%' }"></div></div>
        <div class="tb-wrap thr"><div class="tb-fill" :style="{ height: throttle + '%' }"></div></div>
        <div class="tb-side"><span class="tb-lbl">油门</span><span class="tb-pct">{{ throttle.toFixed(0) }}%</span></div>
      </div>

      <div class="hud-col"><HudPreview :telemetry="telemetry" /></div>

      <div class="hud-ctrl">
        <button class="btn gold" @click="emit('action','start_hud')">启动 HUD</button>
        <button class="btn red" @click="emit('action','stop_hud')">停止 HUD</button>
        <button class="btn" :class="{ 'edit-on': s('hud_edit_mode',false) }" @click="emit('action','hud_edit_mode')" @dblclick="$emit('action','reset_hud')" title="单击激活 HUD | 双击重置 HUD">{{ s('hud_edit_mode',false) ? '激活 HUD' : '激活 HUD' }}</button>
      </div>
    </div>

    <!-- Row 2: Power + Car info -->
    <div class="mid-row">
      <div class="pw-card card curve-card" :class="{ 'card-locked': isLocked, pressing }"
        @pointerdown="onPointerdown" @pointerup="onPointerup" @pointercancel="onPointercancel"
        @dblclick="onDblclick" title="双击重置此车曲线 · 长按切换锁定"><div class="card-label">功率 / 扭矩</div><canvas ref="pwCanvas" class="pw-canvas"></canvas></div>
      <div class="info-card card">
        <div class="card-label">车况</div>
        <div class="info-top">
          <div class="ir"><span>速度</span><span class="in">{{ speed }}</span><span>km/h</span></div>
          <div class="ir"><span>增压</span><span class="in">{{ boost.toFixed(1) }}</span><span>psi</span></div>
          <div class="ir"><span>驱动</span><span>{{ ({0:"FWD",1:"RWD",2:"AWD"}as any)[props.telemetry?.drivetrain]??"-" }}</span></div>
          <div class="ir"><span>缸数</span><span>{{ props.telemetry?.cylinders ? props.telemetry.cylinders + '缸' : '-' }}</span></div>
          <div class="ir"><span>PI</span><span>{{ props.telemetry?.car_perf_index??"-" }}</span></div>
          <div class="ir"><span>ID</span><span>{{ props.telemetry?.car_id??"-" }}</span></div>
        </div>
        <div class="info-bot">
          <div class="ir status-row"><span>曲线</span><span :style="{ color: statusColor, fontWeight: 600 }">{{ statusText }}</span></div>
          <div class="stab-row"><span class="stab-lbl">当前转速增压基准</span><span class="stab-val">{{ telemetry?.curve?.boost_ref != null ? (telemetry.curve.boost_ref as number).toFixed(1) + ' psi' : '-' }}</span></div>
          <div class="lock-row"><button class="btn-lock-dash" :class="{ on: isLocked }" @click="toggleLock">{{ isLocked ? '已锁定' : '未锁定' }}</button></div>
        </div>
      </div>
    </div>

    <!-- Row 3: Temp | G-G | Slip | Susp -->
    <div class="quad-row">
      <div class="tire-box">
        <div class="card-label">胎温 °C</div>
        <div class="tire-inner">
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[0]??0)/120*100,100)+'%', background: tempColor(tires[0]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">FL</span><span class="t-num" :style="{ color: tempColor(tires[0]??0) }">{{ (tires[0]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">FR</span><span class="t-num" :style="{ color: tempColor(tires[1]??0) }">{{ (tires[1]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[1]??0)/120*100,100)+'%', background: tempColor(tires[1]??0) }"></div></div>
          </div>
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[2]??0)/120*100,100)+'%', background: tempColor(tires[2]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">RL</span><span class="t-num" :style="{ color: tempColor(tires[2]??0) }">{{ (tires[2]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">RR</span><span class="t-num" :style="{ color: tempColor(tires[3]??0) }">{{ (tires[3]??0).toFixed(1).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min((tires[3]??0)/120*100,100)+'%', background: tempColor(tires[3]??0) }"></div></div>
          </div>
        </div>
      </div>

      <div class="gg-box card">
        <div class="card-label">G 力</div>
        <div class="gg-wrap"><canvas ref="ggCanvas"></canvas></div>
      </div>

      <div class="susp-box card">
        <div class="card-label">悬挂行程</div>
        <div class="susp-inner">
          <div class="susp-row">
            <div v-for="(idx, i) in suspOrder" :key="suspNames[i]"
              class="susp-cell" :class="{ rear: i === 0 || i === 3, front: i === 1 || i === 2 }">
              <div class="susp-bar-wrap">
                <div class="susp-bar" :style="{ height: ((susp[idx]??0)*100).toFixed(0)+'%', background: travelColor(susp[idx]??0) }"></div>
              </div>
              <div class="susp-pos">{{ suspNames[i] }}</div>
              <div class="susp-val">{{ (susp[idx]??0).toFixed(3) }}</div>
            </div>
          </div>
        </div>
      </div>

      <div class="tire-box">
        <div class="card-label">滑移率</div>
        <div class="tire-inner">
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[0]??0)/2*100,100)+'%', background: slipColor(slipVals[0]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">FL</span><span class="t-num" :style="{ color: slipColor(slipVals[0]??0) }">{{ Math.abs(slipVals[0]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">FR</span><span class="t-num" :style="{ color: slipColor(slipVals[1]??0) }">{{ Math.abs(slipVals[1]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[1]??0)/2*100,100)+'%', background: slipColor(slipVals[1]??0) }"></div></div>
          </div>
          <div class="t-row">
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[2]??0)/2*100,100)+'%', background: slipColor(slipVals[2]??0) }"></div></div>
            <div class="t-meta"><span class="t-pos">RL</span><span class="t-num" :style="{ color: slipColor(slipVals[2]??0) }">{{ Math.abs(slipVals[2]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-meta"><span class="t-pos">RR</span><span class="t-num" :style="{ color: slipColor(slipVals[3]??0) }">{{ Math.abs(slipVals[3]??0).toFixed(3).padStart(5,'0') }}</span></div>
            <div class="t-gauge"><div class="t-fill" :style="{ height: Math.min(Math.abs(slipVals[3]??0)/2*100,100)+'%', background: slipColor(slipVals[3]??0) }"></div></div>
          </div>
        </div>
      </div>
    </div>
  </div>
  </div>
</template>

<style scoped>
/* 缩放视口: 内容按设计稿尺寸钉死, 由 dashZoom 等比缩放并居中 (margin:auto 在溢出时回退为起点对齐) */
.dash-viewport { height: 100%; overflow: auto; display: grid; }
.dashboard { width: 1162px; height: 762px; margin: auto; padding: 16px; display: flex; flex-direction: column; gap: calc(10px + var(--vfl, 0px)); }

/* ── 布局备忘 / Layout notes (hand-tuned, don't change blindly) ──
   行2/3 最大总宽 = 1130 (功率1010+车况110+间距10; 胎温260+G力290+悬挂290+滑移260+间距30)
   行1: 示波器+油门柱(152)+预览 = 功率最大宽1010 → 示波器/预览同步 = (1010-152-20)/2 = 419
   按钮列 110 宽 = 车况卡宽(上下竖线对齐), 高 32*3+4*2 = 104
   手k旋钮: scope/hud min/max-width(160/419), 按钮 margin-top(下沉), 字号13/R角7
   预览高110固定(HudPreview.vue), 示波器绘制见 ThrottleBrakeScope.vue
   固定尺寸是刻意的: 框定胎温/滑移表针与悬挂胶囊的比例美观 + 悬挂前后高低差姿态 — 
   要适配窗口请用 dash-viewport 的 zoom 等比缩放, 不要弹性化单个组件
   浮动间距: zoom 只缩不放 (0.7~1.0, 旋钮 ZOOM_MIN/MAX; 低于 0.7 出滚动条——更小没有参考价值); 窗宽 >1162 时 dashWidth 浮动至
   MAX_W 1280, 余量分摊: 行1/2 行内间距 (calc +var(--hfl), 系数 ROW_GAP_FL) + 示波器/预览/
   功率曲线 (flex 吸收剩余) + 行3盒间距 (space-between); 窗高 >762 时 dashHeight 浮动至
   MAX_H 810, 余量全部给两个行间垂直间距 (calc +var(--vfl), 各行高度不动);
   四盒/车况卡/按钮列固定像素不动 */

/* ── Row 1 ── */
.top-row { display: flex; gap: calc(10px + var(--hfl, 0px)); align-items: center; }
.scope-col {
  flex: 1; min-width: 160px; height: 110px;
  border: 1px solid var(--border); border-radius: 8px;
  background: var(--card); overflow: hidden;
}
.hud-col { flex: 1; min-width: 160px; }
.hud-col > * { width: 100%; }

.tb-col { display: flex; gap: 6px; flex-shrink: 0; height: 110px; align-items: stretch; }
.tb-side { display: flex; flex-direction: column; align-items: center; justify-content: center; width: 22px; gap: 2px; }
.tb-lbl { font-size: 11px; color: var(--dim); }
.tb-pct { font-size: 13px; color: var(--text); font-weight: 600; }
.tb-wrap { width: 45px; background: rgba(255,255,255,0.05); border-radius: 11px; overflow: hidden; display: flex; align-items: flex-end; }
.tb-fill { width: 100%; background: #fff; border-radius: 0; transition: height 0.15s; }

/* 32 + 4 + 32 + 4 + 32 = 104; margin-top sinks the block below row center */
.hud-ctrl { display: flex; flex-direction: column; gap: 4px; flex-shrink: 0; width: 110px; height: 104px; align-items: stretch; margin-top: 2px; }
.btn {
  height: 32px; display: flex; align-items: center; justify-content: center;
  padding: 0 8px; border: 1px solid var(--border); border-radius: 7px;
  background: var(--card); color: var(--text); font-size: 13px; cursor: pointer; text-align: center;
}
.btn:hover { background: #222; }
.btn.gold { background: var(--accent); color: #121314; border-color: var(--accent); font-weight: 600; }
.btn.gold:hover { background: #e0b850; }
.btn.red { color: var(--red); border-color: var(--red); }
.btn.red:hover { background: rgba(241,76,76,0.12); }
.btn.edit-on { border: 2px dashed rgba(255,204,0,0.55); color: #ffcc00; }

/* ── Row 2 ── */
.mid-row { display: flex; gap: calc(10px + var(--hfl, 0px)); height: 300px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 10px; display: flex; flex-direction: column; }
.card-label { font-size: 10px; color: var(--dim); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 4px; flex-shrink: 0; }
.pw-card { flex: 1; min-width: 200px; }
.pw-canvas { flex: 1; width: 100%; min-height: 0; }
.info-card { width: 110px; flex-shrink: 0; align-items: center; text-align: center; }
.info-top { align-self: stretch; text-align: left; }
.info-bot { display: flex; flex-direction: column; align-items: center; }
.ir { display: flex; gap: 4px; color: var(--dim); font-size: 14px; margin-bottom: 3px; }
.ir span:first-child { width: 28px; flex-shrink: 0; }
.in { color: var(--text); font-weight: 600; }

/* ── Row 3 ── */
.quad-row { display: flex; gap: 10px; align-items: stretch; height: 300px; justify-content: space-between; }
.tire-box { width:260px; flex-shrink: 0; background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 10px; display: flex; flex-direction: column; }
.tire-inner { flex: 1; display: flex; flex-direction: column; gap: 8px; }
.t-row { display: flex; gap: 4px; flex: 1; }
/* char / gauge / gauge / char */
.t-meta { display: flex; flex-direction: column; align-items: center; justify-content: center; width: 36px; gap: 3px; flex-shrink: 0; }
.t-pos { font-size: 16px; font-weight: 700; color: var(--text); line-height: 1; }
.t-num { font-size: 13px; font-weight: 600; font-variant-numeric: tabular-nums; }
.t-gauge { flex: 1; background: rgba(255,255,255,0.04); border-radius: 12px; overflow: hidden; display: flex; align-items: flex-end; }
.t-fill { width: 100%; border-radius: 0; transition: height 0.3s; }

.gg-box { width: 290px; flex-shrink: 0; align-items: center; display: flex; flex-direction: column; }
.gg-wrap { flex: 1; display: flex; align-items: center; justify-content: center; width: 100%; }
.gg-wrap canvas { width: 250px; height: 250px; }

.susp-box { width: 290px; flex-shrink: 0; }
.susp-inner { flex: 1; display: flex; align-items: center; justify-content: center; }
.susp-row { display: flex; gap: 28px; justify-content: center; align-items: flex-end; }
.susp-cell { display: flex; flex-direction: column; align-items: center; gap: 2px; position: relative; }
.susp-cell.front { top: -28px; }
.susp-cell.rear { top: 20px; }
.susp-pos { font-size: 12px; font-weight: 700; color: var(--accent); }
.susp-val { font-size: 9px; color: var(--dim); }
.susp-bar-wrap { width: 32px; height: 140px; background: rgba(255,255,255,0.05); border-radius: 16px; overflow: hidden; display: flex; align-items: flex-end; }
.susp-bar { width: 100%; border-radius: 0; transition: height 0.2s; min-height: 0; }

.card-locked { border-color: var(--accent); }
.curve-card { position: relative; overflow: hidden; user-select: none; cursor: pointer; }
/* 长按进度条：与 usePressGestures 默认 duration (500ms) 同步 */
.curve-card::after {
  content: ""; position: absolute; left: 0; bottom: 0; height: 3px; width: 0;
  background: var(--accent);
}
.curve-card.pressing::after { width: 100%; transition: width 0.5s linear; }
/* Divider spans the full card width (into the 10px card padding), text stays aligned */
.status-row {
  align-self: stretch;
  margin: 4px -10px 0;
  padding: 10px 10px 0;
  border-top: 1px solid var(--border);
}
.stab-row { display: flex; flex-direction: column; align-items: center; margin-bottom: 6px; }
.stab-lbl { font-size: 14px; color: var(--dim); text-transform: uppercase; letter-spacing: 0.5px; }
.stab-val { font-size: 14px; color: var(--text); font-weight: 600; }
.lock-row { margin-top: auto; padding-top: 4px; }
.btn-lock-dash {
  padding: 5px 12px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--card); color: var(--dim); font-size: 12px; cursor: pointer; width: 100%;
}
.btn-lock-dash:hover { background: #333; color: var(--text); }
.btn-lock-dash.on { border-color: var(--orange); color: var(--orange); }
.btn-lock-dash.on:hover { background: rgba(210,153,29,0.12); }
</style>
