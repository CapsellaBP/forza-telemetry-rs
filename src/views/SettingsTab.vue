<script setup lang="ts">
import { ref, watch } from "vue"
import InfoTip from "../components/InfoTip.vue"

const props = defineProps<{ telemetry: any }>()
const emit = defineEmits<{ set: [key: string, value: any], action: [action: string] }>()

function s(key: string, fallback: any = null) {
  return props.telemetry?.settings?.[key] ?? fallback
}

const portVal = ref(s('udp_port', 5300))
watch(() => s('udp_port', 5300), (v) => { portVal.value = v })

function onRenderModeChange(e: Event) {
  emit('set', 'render_mode', (e.target as HTMLSelectElement).value)
  emit('action', 'restart_app')
}

</script>

<template>
  <div class="tab-page">
    <h2>设置</h2>
    <p class="subtitle">应用级设置 · 功能参数在对应页面</p>

    <div class="card">
      <h3>渲染模式 <InfoTip>切换后应用自动重启生效。自动 = 检测到 Motorsport 用软件渲染、Horizon 用硬件渲染</InfoTip></h3>
      <div class="row">
        <label>渲染模式</label>
        <select :value="String(s('render_mode', 'software'))" @change="onRenderModeChange"
          style="background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:4px;padding:3px 8px">
          <option value="auto">自动</option>
          <option value="software">软件渲染</option>
          <option value="hardware">硬件渲染</option>
        </select>
        <div style="font-size:10px;color:var(--dim);margin-top:2px">切换后应用自动重启生效</div>
      </div>
    </div>

    <div class="card">
      <h3>UDP 端口 <InfoTip>修改后重启应用生效。游戏内 Data Out 端口需与此一致</InfoTip></h3>
      <div class="row">
        <label>监听端口</label>
        <input type="number" min="1024" max="65535" :value="portVal"
          @input="portVal = Number(($event.target as HTMLInputElement).value)"
          @change="emit('set', 'udp_port', portVal)"
          style="width:80px;background:var(--bg);color:var(--text);border:1px solid var(--border);border-radius:4px;padding:4px 8px">
        <button class="btn" @click="emit('action', 'restart_app')" style="margin-left:8px">保存并重启</button>
        <div style="font-size:10px;color:var(--dim);margin-top:2px">修改端口后需手动重启应用生效</div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.tab-page { padding: 24px; max-width: 600px; }
h2 { font-size: 18px; margin-bottom: 2px; }
.subtitle { font-size: 12px; color: var(--dim); margin-bottom: 20px; }

.card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: 8px; padding: 16px; margin-bottom: 12px;
}
.card h3 {
  font-size: 12px; color: var(--accent); text-transform: uppercase;
  letter-spacing: 1px; margin-bottom: 12px;
}

.row {
  display: flex; align-items: center; gap: 10px; margin-bottom: 10px;
}
.row label { font-size: 12px; color: var(--dim); min-width: 110px; flex-shrink: 0; }
.row input[type=range] { flex: 1; accent-color: var(--accent); height: 4px; }
.val { font-size: 11px; color: var(--text); min-width: 60px; text-align: right; }
.btn {
  padding: 5px 12px; border: 1px solid var(--border); border-radius: 4px;
  background: var(--card); color: var(--text); font-size: 12px; cursor: pointer;
}
.btn:hover { background: #333; }
</style>
