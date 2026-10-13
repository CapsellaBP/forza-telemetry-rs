<script setup lang="ts">
import { ref, onMounted, watch, nextTick, computed } from "vue"
import InfoTip from "../components/InfoTip.vue"
import { usePressGestures } from "../composables/usePressGestures"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any], action: [action: string] }>()

function s(key: string, fb: any = null) { return props.telemetry?.settings?.[key] ?? fb }

const isLocked = computed(() => !!props.telemetry?.curve?.locked)
const boostStable = computed(() => !!props.telemetry?.curve?.boost_stable)
const boostStableOn = computed(() => !!s('boost_stable_sample'))

const statusText = computed(() => {
  if (isLocked.value) return "已锁定"
  if (props.telemetry?.shift_advice?.ready) return "就绪"
  return "采样中"
})
const statusColor = computed(() => {
  if (isLocked.value) return "var(--orange)"
  if (props.telemetry?.shift_advice?.ready) return "var(--green)"
  return "#3399ff"
})

function toggleLock() {
  emit('action', isLocked.value ? 'unlock_curve' : 'lock_curve')
}
const { pressing, onPointerdown, onPointerup, onPointercancel, onDblclick } = usePressGestures({
  onLongPress: toggleLock,
  onDoubleClick: () => emit('action', 'reset_curve'),
})

const pwCanvas = ref<HTMLCanvasElement | null>(null)
function drawPower() {
  const c = pwCanvas.value; if (!c) return
  const dpr = window.devicePixelRatio || 1
  const w = c.clientWidth, h = c.clientHeight
  const bw = Math.round(w * dpr), bh = Math.round(h * dpr)
  if (c.width !== bw || c.height !== bh) { c.width = bw; c.height = bh }
  const ctx = c.getContext("2d")!
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.fillStyle = "#121314"; ctx.fillRect(0, 0, w, h)

  const pad = { t: 24, r: 55, b: 28, l: 48 }
  const pw = w - pad.l - pad.r, ph = h - pad.t - pad.b
  const curve = props.telemetry?.curve
  const tq: [number,number][] = curve?.torque || []
  const pk: [number,number][] = curve?.power_kw || []

  let rLo = 0, rHi = props.telemetry?.rpm_max || 9000, tM = 500, kM = 300
  for (const [r,v] of tq) { rLo = Math.min(rLo, r); rHi = Math.max(rHi, r); tM = Math.max(tM, v) }
  for (const [,v] of pk) kM = Math.max(kM, v)
  if (rLo === 0 && pk.length === 0) { rLo = 1500; rHi = 9000; tM = 500; kM = 300 }
  const pR = (rHi - rLo) * 0.05 || 200; rLo = Math.max(0, rLo - pR); rHi += pR
  tM *= 1.1; kM *= 1.1
  if (tM < 200) tM = 500; if (kM < 100) kM = 300
  const hM = kM * 1.341

  const x = (r: number) => pad.l + (r - rLo) / (rHi - rLo) * pw
  const yt = (v: number) => pad.t + ph - (v / tM) * ph
  const yp = (v: number) => pad.t + ph - (v / kM) * ph

  // Grid
  ctx.strokeStyle = "rgba(255,255,255,0.04)"; ctx.lineWidth = 0.5
  for (let i = 0; i <= 4; i++) { const gy = pad.t + ph * i / 4; ctx.beginPath(); ctx.moveTo(pad.l, gy); ctx.lineTo(w - pad.r, gy); ctx.stroke() }
  ctx.strokeStyle = "#2d2e30"; ctx.lineWidth = 1
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t); ctx.lineTo(pad.l, pad.t + ph); ctx.stroke()
  ctx.beginPath(); ctx.moveTo(pad.l, pad.t + ph); ctx.lineTo(w - pad.r, pad.t + ph); ctx.stroke()

  // X axis (RPM)
  ctx.fillStyle = "#8b8f93"; ctx.font = "9px sans-serif"; ctx.textAlign = "center"
  for (let r = Math.ceil(rLo / 1000) * 1000; r <= rHi; r += 1000) ctx.fillText((r / 1000).toFixed(0) + "k", x(r), pad.t + ph + 16)

  // Torque Y axis (Nm)
  ctx.textAlign = "right"
  for (let i = 0; i <= 4; i++) {
    const v = tM * i / 4
    ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"
    ctx.fillText(v.toFixed(0), pad.l - 6, yt(v) + 3)
  }
  ctx.fillStyle = "#58a6ff"; ctx.font = "bold 9px sans-serif"
  ctx.fillText("Nm", pad.l - 6, pad.t - 8)

  // Power Y axis (hp)
  ctx.textAlign = "left"
  for (let i = 0; i <= 4; i++) {
    const v = hM * i / 4
    ctx.fillStyle = "#8b8f93"; ctx.font = "8px sans-serif"
    ctx.fillText(v.toFixed(0), w - pad.r + 6, yp(v / 1.341) + 3)
  }
  ctx.fillStyle = "#f85149"; ctx.font = "bold 9px sans-serif"
  ctx.fillText("hp", w - pad.r + 6, pad.t - 8)

  // Torque curve
  if (tq.length > 1) { ctx.strokeStyle = "#58a6ff"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < tq.length; i++) i === 0 ? ctx.moveTo(x(tq[i][0]), yt(tq[i][1])) : ctx.lineTo(x(tq[i][0]), yt(tq[i][1])); ctx.stroke() }
  // Power curve
  if (pk.length > 1) { ctx.setLineDash([3, 3]); ctx.strokeStyle = "#f85149"; ctx.lineWidth = 1.5; ctx.beginPath(); for (let i = 0; i < pk.length; i++) i === 0 ? ctx.moveTo(x(pk[i][0]), yp(pk[i][1])) : ctx.lineTo(x(pk[i][0]), yp(pk[i][1])); ctx.stroke(); ctx.setLineDash([]) }
  // Shift line
  const sr = props.telemetry?.shift_advice?.shift_rpm
  if (sr > 0) { ctx.strokeStyle = "#d4a843"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(sr), pad.t); ctx.lineTo(x(sr), pad.t + ph); ctx.stroke() }
  // Current RPM
  const cr = props.telemetry?.rpm
  if (cr > 0) { ctx.setLineDash([2, 4]); ctx.strokeStyle = "#ccc"; ctx.lineWidth = 1; ctx.beginPath(); ctx.moveTo(x(cr), pad.t); ctx.lineTo(x(cr), pad.t + ph); ctx.stroke(); ctx.setLineDash([]) }
}
watch(() => props.telemetry?.pkt_count, () => { if (!pwCanvas.value?.clientWidth) return; nextTick(drawPower) })
onMounted(() => nextTick(drawPower))
</script>

<template>
  <div class="tab-page">
    <h2>动力系统</h2>
    <p class="subtitle">功率曲线 · 采样 · 增压 · 换挡 · EV 检测</p>

    <div class="card curve-card" :class="{ 'card-locked': isLocked, pressing }"
      @pointerdown="onPointerdown" @pointerup="onPointerup" @pointercancel="onPointercancel"
      @dblclick="onDblclick" title="双击重置此车曲线 · 长按切换锁定">
      <h3>功率 / 扭矩曲线 <span v-if="isLocked" class="lock-badge">已锁定</span></h3>
      <canvas ref="pwCanvas" style="width:100%;height:280px"></canvas>
    </div>

    <div class="two-col">
      <div class="card">
        <h3>功率带</h3>
        <div class="row">
          <label>功率带阈值 <InfoTip>峰值功率百分比。高于此值的 RPM 区间计入功率带（HUD 白色带）。全局设置</InfoTip></label>
          <input type="range" min="50" max="100" :value="(s('power_band_pct', 0.93)) * 100"
            @input="emit('set', 'power_band_pct', Number(($event.target as HTMLInputElement).value) / 100)">
          <span class="val">{{ ((s('power_band_pct', 0.93)) * 100).toFixed(0) }}%</span>
        </div>
      </div>

      <div class="card">
        <h3>当前数据</h3>
      <div class="data-grid">
        <div class="d-item"><span class="lbl">峰值功率</span><span class="v">{{ telemetry?.shift_advice?.peak_power_rpm?.toFixed(0) ?? "-" }} RPM</span></div>
        <div class="d-item"><span class="lbl">换挡点</span><span class="v">{{ telemetry?.shift_advice?.shift_rpm?.toFixed(0) ?? "-" }} RPM</span></div>
        <div class="d-item"><span class="lbl">最大转速</span><span class="v" :style="{ color: telemetry?.shift_advice?.fuel_cut_rpm ? 'var(--red)' : '' }">{{ telemetry?.shift_advice?.fuel_cut_rpm?.toFixed(0) || telemetry?.shift_advice?.limiter_rpm?.toFixed(0) || "-" }} RPM</span></div>
        <div class="d-item"><span class="lbl">采样数</span><span class="v">{{ telemetry?.shift_advice?.samples ?? 0 }}</span></div>
        <div class="d-item boost-item"><span class="lbl">增压</span><span class="v">{{ telemetry?.boost_psi?.toFixed(1) ?? "-" }} psi<span v-if="boostStableOn" class="boost-state">{{ boostStable ? '当前压力稳定' : '等待压力稳定' }}</span></span></div>
        <div class="d-item boost-item"><span class="lbl">当前转速基准</span><span class="v">{{ telemetry?.curve?.boost_ref != null ? (telemetry.curve.boost_ref as number).toFixed(1) + ' psi' : '-' }}<span v-if="boostStableOn" class="boost-state">增压稳定采样状态：{{ boostStable ? '已获取到压力稳定数据' : '等待稳定增压数据' }}</span></span></div>
        <div class="d-item"><span class="lbl">状态</span><span class="v" :style="{ color: statusColor }">{{ statusText }}</span></div>
        </div>
      </div>
    </div>

    <div class="two-col">
      <div class="card">
        <h3>曲线管理</h3>
        <div class="action-row">
          <button class="btn" :class="{ 'btn-active': isLocked }" @click="toggleLock">{{ isLocked ? '已锁定' : '未锁定' }}</button>
          <button class="btn" @click="emit('action', 'reset_curve')">重置此车</button>
          <button class="btn" @click="emit('action', 'reset_all')">全部重置</button>
        </div>
      </div>
      <div class="card">
        <h3>采样参数</h3>
        <div class="row">
          <label>最低油门 <InfoTip>低于此油门开度不采样，过滤松油/滑行数据</InfoTip></label>
          <input type="range" min="50" max="100" :value="(s('sample_throttle_min', 0.95)) * 100"
            @input="emit('set', 'sample_throttle_min', Number(($event.target as HTMLInputElement).value) / 100)">
          <span class="val">{{ ((s('sample_throttle_min', 0.95)) * 100).toFixed(0) }}%</span>
        </div>
        <div class="row">
          <label>采样跳过时长 <InfoTip>换挡后跳过 N 毫秒再采样，过滤换挡瞬间的脏数据。按时长计算，与发包率无关</InfoTip></label>
          <input type="range" min="0" max="500" step="10" :value="s('sample_skip_ms', 130)"
            @input="emit('set', 'sample_skip_ms', Number(($event.target as HTMLInputElement).value))">
          <span class="val">{{ s('sample_skip_ms', 130) }}ms</span>
        </div>
        <div class="row">
          <label>EMA 权重 <InfoTip>新数据的权重。1/2=快速覆盖，1/16=极度平滑。值越大越敏感</InfoTip></label>
          <select :value="String(s('curve_alpha', 0.25))"
            @change="emit('set', 'curve_alpha', Number(($event.target as HTMLSelectElement).value))"
            style="background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:4px;padding:3px 8px">
            <option value="0.5">1/2 (快)</option>
            <option value="0.25">1/4 (默认)</option>
            <option value="0.125">1/8</option>
            <option value="0.0625">1/16 (慢)</option>
          </select>
        </div>
      </div>
    </div>

    <div class="card">
      <h3>增压稳定采样</h3>
      <p class="card-note">按转速分段学习全油门增压基准，压力接近当前转速的基准才采样。涡轮与离心式机械增压（压力随转速爬升）均适用。</p>
      <div class="row">
        <label class="check-label">
          <input type="checkbox" :checked="!!s('boost_stable_sample', true)"
            @change="emit('set', 'boost_stable_sample', ($event.target as HTMLInputElement).checked)">
          启用
        </label>
      </div>
      <div class="two-col-inner">
        <div class="row">
          <label>增压容差 <InfoTip>当前增压 >= 当前转速基准 × 峰值容忍 − 此容差即放行采样。补偿低增压区的压力脉动，默认 0</InfoTip></label>
          <input type="range" min="0" max="100" step="5" :value="(s('boost_margin', 0)) * 100"
            @input="emit('set', 'boost_margin', Number(($event.target as HTMLInputElement).value) / 100)">
          <span class="val">{{ (s('boost_margin', 0)).toFixed(2) }} psi</span>
        </div>
        <div class="row">
          <label>峰值容忍 <InfoTip>当前增压 >= 当前转速基准 × 此比例 − 增压容差 即放行采样。0.90 = 允许 10% 波动。更小 = 更宽容</InfoTip></label>
          <input type="range" min="50" max="100" :value="(s('boost_stable_tol', 0.98)) * 100"
            @input="emit('set', 'boost_stable_tol', Number(($event.target as HTMLInputElement).value) / 100)">
          <span class="val">{{ (s('boost_stable_tol', 0.98) * 100).toFixed(0) }}%</span>
        </div>
      </div>
    </div>

    <div class="card">
      <h3>EV 检测 <span class="tag">Motorsport 禁用</span></h3>
      <div class="row">
        <label>检测帧数 <InfoTip>全油门时档位≤2 持续 N 帧判为电车（自动隐藏换挡线/最大转速线）</InfoTip></label>
        <input type="range" min="100" max="2000" step="50" :value="s('ev_detect_frames', 300)"
          @input="emit('set', 'ev_detect_frames', Number(($event.target as HTMLInputElement).value))">
        <span class="val">{{ s('ev_detect_frames', 300) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tab-page { padding: 24px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }
.card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 12px; transition: border-color 0.2s; }
.card h3 { font-size: 12px; color: var(--accent); text-transform: uppercase; letter-spacing: 1px; margin-bottom: 12px; }
.card-note { font-size: 11px; color: var(--dim); margin-bottom: 10px; line-height: 1.45; }
.card-locked { border-color: var(--accent); }
.curve-card { position: relative; overflow: hidden; user-select: none; cursor: pointer; }
/* 长按进度条：与 usePressGestures 默认 duration (500ms) 同步 */
.curve-card::after {
  content: ""; position: absolute; left: 0; bottom: 0; height: 3px; width: 0;
  background: var(--accent);
}
.curve-card.pressing::after { width: 100%; transition: width 0.5s linear; }
.lock-badge { font-size: 10px; color: var(--orange); font-weight: 400; margin-left: 6px; text-transform: none; }
.tag { font-size: 10px; color: var(--orange); font-weight: 400; margin-left: 4px; }
.row { display: flex; align-items: center; gap: 10px; margin-bottom: 10px; }
.row label { font-size: 12px; color: var(--dim); min-width: 110px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 60px; text-align: right; }
.data-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 6px 16px; }
.d-item { display: flex; gap: 8px; font-size: 13px; }
.lbl { color: var(--dim); width: 60px; flex-shrink: 0; }
.v { color: var(--text); }
.two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.two-col-inner { display: grid; grid-template-columns: 1fr 1fr; gap: 0 10px; }
.action-row { display: flex; gap: 6px; flex-wrap: wrap; }
.check-label { min-width: auto !important; cursor: pointer; display: flex; align-items: center; gap: 6px; }
.check-label input[type=checkbox] { accent-color: var(--accent); }
.btn {
  padding: 5px 12px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--card); color: var(--text); font-size: 12px; cursor: pointer;
}
.btn:hover { background: #333; }
.btn-active {
  border-color: var(--orange); color: var(--orange);
}
.btn-active:hover { background: rgba(210,153,29,0.12); }
.boost-item { grid-column: 1 / -1; }
.boost-state { font-size: 11px; color: var(--dim); margin-left: 8px; }
</style>
