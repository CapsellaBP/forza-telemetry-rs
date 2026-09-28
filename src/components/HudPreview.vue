<script setup lang="ts">
import { computed } from "vue"

const props = defineProps<{ telemetry: any }>()

// ── settings-aware values (use defaults from server.py) ──
const settings = computed(() => props.telemetry?.settings || {})

const fsGear = computed(() => settings.value.gear_font_size ?? 28)
const fsSpeed = computed(() => settings.value.speed_font_size ?? 42)
const fsLabel = computed(() => settings.value.label_font_size ?? 12)
const fwGear = computed(() => settings.value.gear_font_weight ?? 500)
const fwSpeed = computed(() => settings.value.speed_font_weight ?? 500)
const italicGear = computed(() => settings.value.gear_italic ?? true)
const italicSpeed = computed(() => settings.value.speed_italic ?? true)
const italicLabel = computed(() => settings.value.label_italic ?? true)
const glow = computed(() => settings.value.gear_glow ?? true)
const topGap = computed(() => settings.value.top_row_gap ?? 4)
const speedLabelGap = computed(() => settings.value.speed_label_gap ?? 3)
const rpmBright = computed(() => settings.value.rpm_bar_brightness ?? 1)
const rpmHue = computed(() => settings.value.rpm_bar_hue ?? 0)
const glowStrength = computed(() => settings.value.hud_glow ?? 1)
const powerBandOpacity = computed(() => settings.value.power_band_opacity ?? 0.5)

// ── telemetry data ──
const speed = computed(() => String(Math.round(props.telemetry?.speed_kmh ?? 0)).padStart(3, "0"))
const gear = computed(() => props.telemetry?.gear ?? 1)
function gearLabel(g: number) { return g === 0 ? "R" : g === -1 ? "N" : String(g ?? "-") }

const redZone = computed(() => {
  if (props.telemetry?.is_ev) return false
  const rf = (settings.value.red_flash_pct ?? 0.01) * 100
  return rpmPct.value >= limiterPct.value - rf
})

const gearColor = computed(() => {
  if (redZone.value) return "var(--red)"
  const a = props.telemetry?.shift_advice
  if (!a) return "var(--text)"
  return a.urgency === "over" || a.urgency === "shift" ? "#3399ff" :
         a.urgency === "near" ? "var(--orange)" : "var(--text)"
})

const rpm = computed(() => props.telemetry?.rpm ?? 0)
const rpmMax = computed(() => props.telemetry?.rpm_max || 8000)
const rpmIdle = computed(() => props.telemetry?.rpm_idle || 1500)
const rpmRange = computed(() => Math.max(rpmMax.value - rpmIdle.value, 1))
const rpmPct = computed(() =>
  Math.max(0, Math.min(100, (rpm.value - rpmIdle.value) / rpmRange.value * 100)))

const powerKw = computed(() => props.telemetry?.power_kw ?? 0)
const boostPsi = computed(() => props.telemetry?.boost_psi ?? 0)

const yellowStart = computed(() => settings.value.gear_yellow_start ?? 0.5)

function rpmColor(pct: number, bluePct: number): string {
  // Zones anchor to the shift (blue) line so the full progression displays
  // left of it; past the line stays red under the strobes. Fallback: full range.
  const y0 = yellowStart.value * 100
  const top = bluePct > y0 + 1 ? bluePct : 100
  if (pct < y0) return "rgba(255,255,255,0.45)"
  const zone = (top - y0) / 3
  if (pct < y0 + zone) return "#ffdd00"
  if (pct < y0 + 2 * zone) return "#ff8800"
  return "#ff3300"
}

// Markers — show demo if no data
const shiftLinePct = computed(() => {
  const a = props.telemetry?.shift_advice
  if (a?.ready) return Math.max(0, Math.min(100, (a.shift_rpm - rpmIdle.value) / rpmRange.value * 100))
  return 0
})
const fuelCutRpm = computed(() => {
  const a = props.telemetry?.shift_advice
  return a?.fuel_cut_rpm || a?.limiter_rpm || 0
})
const limiterPct = computed(() => {
  if (fuelCutRpm.value > 0) return Math.max(0, Math.min(100, (fuelCutRpm.value - rpmIdle.value) / rpmRange.value * 100))
  return 100
})
const pbLoPct = computed(() => {
  const a = props.telemetry?.shift_advice
  if (a?.ready && a?.shift_rpm > rpmIdle.value) {
    const pb = props.telemetry?.power_band
    const lo = pb?.lo ?? (rpmIdle.value + rpmRange.value * 0.33)
    const hi = pb?.hi ?? (rpmIdle.value + rpmRange.value * 0.63)
    const left = Math.max(0, (lo - rpmIdle.value) / rpmRange.value * 100)
    const width = Math.min(100 - left, Math.max(0, (hi - lo) / rpmRange.value * 100))
    return { left: left.toFixed(1), width: width.toFixed(1) }
  }
  // Default fallback when curve not ready
  return { left: '33', width: '30' }
})

const editMode = computed(() => !!settings.value.hud_edit_mode)

const maxSlip = computed(() => props.telemetry?.max_slip ?? 0)

// Slip bar: max |slip| over all four wheels, EMA-smoothed like the real HUD;
// speed-scaled leniency below 25 km/h
const slipVal = computed(() => {
  const t = props.telemetry
  if (!t || !(settings.value.slip_bar ?? true)) return null
  const ts: number[] = t.tire_slip ?? [0, 0, 0, 0]
  let m = 0
  for (const v of ts) { const a = Math.abs(v ?? 0); if (a > m) m = a }
  const w = window as any
  w._pvSlip = (w._pvSlip ?? 0) + ((m - (w._pvSlip ?? 0)) * 0.12)
  return w._pvSlip
})
const slipBarColor = computed(() => {
  const kmh = props.telemetry?.speed_kmh ?? 0
  const leni = Math.max(0, Math.min(1, (25 - kmh) / 20))
  const yW = settings.value.slip_warn ?? 0.5, rD = settings.value.slip_danger ?? 2.0
  const yT = yW * (1 + leni), rT = rD + 1.5 * leni
  const L = slipVal.value ?? 0
  return L < yT ? "rgba(255,255,255,0.6)" : L < rT ? "rgba(212,168,67,0.8)" : "rgba(255,51,51,0.9)"
})
const slipWarn = computed(() => settings.value.slip_warn ?? 0.5)
const slipDanger = computed(() => settings.value.slip_danger ?? 2.0)
const slipColor = computed(() => {
  if (maxSlip.value > slipDanger.value) return "#ff3333"
  if (maxSlip.value > slipWarn.value) return "#ffcc00"
  return ""
})
</script>

<template>
  <div class="hud-preview" :class="{ 'edit-mode': editMode }"
    :style="{
      '--fs-gear': fsGear + 'px',
      '--fs-speed': fsSpeed + 'px',
      '--fs-label': fsLabel + 'px',
      '--fw-gear': fwGear,
      '--fw-speed': fwSpeed,
      '--gap': topGap + 'px',
      '--sl-gap': speedLabelGap + 'px',
      '--rpm-bright': rpmBright,
      '--rpm-hue': rpmHue + 'deg',
      '--glow': glowStrength,
    }">
    <div class="hud-top-row" :style="{ gap: topGap + 'px' }">
      <div class="hud-gear"
        :class="{ italic: italicGear, glow }"
        :style="{ color: gearColor }">{{ gearLabel(gear) }}</div>
      <div class="hud-speed-group" :style="{ gap: speedLabelGap + 'px' }">
        <span class="hud-speed" :class="{ italic: italicSpeed }">{{ speed }}</span>
        <div class="hud-labels" :class="{ italic: italicLabel }">
          <span>KM/H</span>
          <span :style="{ color: slipColor }">SLIP</span>
        </div>
      </div>
    </div>

    <div class="hud-rpm-wrap">
      <div class="hud-rpm-bar" :style="{ width: rpmPct + '%', background: rpmColor(rpmPct, shiftLinePct > 0 ? shiftLinePct : 100), filter: 'brightness(' + rpmBright + ') hue-rotate(' + rpmHue + 'deg)' }"></div>
      <div class="hud-shift-line" :style="{ left: shiftLinePct + '%' }"></div>
      <div class="hud-limiter-line" :style="{ left: limiterPct + '%' }"></div>
      <div class="hud-power-band"
        :style="{ left: pbLoPct.left + '%', width: pbLoPct.width + '%', opacity: powerBandOpacity }">
      </div>
    </div>
    <div v-if="slipVal !== null"
      style="position:relative;height:3px;background:rgba(255,255,255,0.06);margin:-2px 0 4px">
      <div :style="{ width: Math.min(Math.log1p(slipVal) / Math.log1p(100) * 100, 100) + '%', height: '100%', background: slipBarColor }"></div>
    </div>

    <div class="hud-info-row">
      <span>{{ rpm.toFixed(0) }} rpm</span>
      <span>{{ (powerKw * 1.341).toFixed(0) }} hp</span>
      <span v-if="boostPsi > 0.1">{{ boostPsi.toFixed(1) }} psi</span>
    </div>
    <div class="hud-debug-row">
      PB:{{ pbLoPct.left }}+{{ pbLoPct.width }} rdy={{ props.telemetry?.shift_advice?.ready ? 1 : 0 }} sr={{ props.telemetry?.shift_advice?.shift_rpm }} sl={{ shiftLinePct }} lm={{ limiterPct }}
    </div>
  </div>
</template>

<style scoped>
.hud-preview {
  background: rgba(0,0,0,0.45);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 14px 8px;
  height: 110px;
  overflow: hidden;
}
.hud-preview.edit-mode {
  border: 2px dashed rgba(255,204,0,0.5);
}
.hud-top-row {
  display: flex; align-items: baseline; justify-content: center;
  margin-bottom: 4px;
}
.hud-gear {
  font-size: var(--fs-gear, 28px);
  font-weight: var(--fw-gear, 500);
  line-height: 1;
  text-shadow: 0 1px 3px rgba(0,0,0,0.8), 0 0 24px rgba(0,0,0,0.9);
}
.hud-gear.italic { font-style: italic; }
.hud-gear.glow {
  background: linear-gradient(to top, rgba(255,255,255,0.16), transparent);
  padding: 1px 5px;
}
.hud-speed-group { display: flex; align-items: center; gap: 4px; }
.hud-speed {
  font-size: var(--fs-speed, 42px);
  font-weight: var(--fw-speed, 500);
  font-family: 'Consolas', 'Cascadia Code', 'Courier New', monospace;
  color: #fff; line-height: 1;
}
.hud-speed.italic { font-style: italic; }
.hud-labels {
  display: flex; flex-direction: column;
  font-size: var(--fs-label, 11px);
  color: #fff; opacity: 0.85; line-height: 1.15;
}
.hud-labels.italic { font-style: italic; }

.hud-rpm-wrap {
  position: relative; height: 5px;
  background: rgba(255,255,255,0.10);
  margin-bottom: 4px; overflow: visible;
}
.hud-rpm-bar { height: 100%; position: relative; }
.hud-rpm-bar::after {
  content:""; position:absolute; right:0; top:0; width:1px; height:100%; background:#fff;
  box-shadow: 0 0 calc(4px*var(--glow,1)) calc(1px*var(--glow,1)) rgba(255,255,255,calc(0.6*var(--glow,1))),
              0 0 calc(10px*var(--glow,1)) calc(2px*var(--glow,1)) rgba(255,255,255,calc(0.35*var(--glow,1)));
}
.hud-shift-line, .hud-limiter-line {
  position: absolute; top: -2px; width: 2px; height: 9px;
}
.hud-shift-line { background: #3399ff; }
.hud-limiter-line { background: #ff3333; }
.hud-shift-line::before, .hud-limiter-line::before {
  content:""; position:absolute; inset:0; pointer-events:none;
  box-shadow: 0 0 calc(9px*var(--glow,1)) calc(3px*var(--glow,1)) rgba(0,0,0,calc(0.5*var(--glow,1)));
}
.hud-shift-line::after, .hud-limiter-line::after {
  content:""; position:absolute; inset:0; pointer-events:none;
}
.hud-shift-line::after {
  box-shadow: 0 0 calc(4px*var(--glow,1)) calc(1px*var(--glow,1)) rgba(200,230,255,calc(0.5*var(--glow,1))),
              0 0 calc(12px*var(--glow,1)) calc(3px*var(--glow,1)) rgba(51,153,255,calc(0.65*var(--glow,1)));
}
.hud-limiter-line::after {
  box-shadow: 0 0 calc(4px*var(--glow,1)) calc(1px*var(--glow,1)) rgba(255,210,200,calc(0.5*var(--glow,1))),
              0 0 calc(12px*var(--glow,1)) calc(3px*var(--glow,1)) rgba(255,51,51,calc(0.65*var(--glow,1)));
}
.hud-power-band {
  position: absolute; top: 2px; height: 1px;
  background: #fff;
}
.hud-info-row {
  display: flex; gap: 12px; font-size: 12px; opacity: 0.85; justify-content: center;
}
.hud-debug-row {
  font-size: 9px; opacity: 0.35; text-align: center; margin-top: 2px;
  white-space: nowrap; overflow: hidden;
}
</style>
