import { ref } from "vue"

// 双击 + 长按共存于同一元素：双击 = 重置，按住不放 = 锁定/解锁。
// 慢速双击的第二击若按住超过 duration 会先触发长按，随后的 dblclick
// 必须吞掉，否则两种手势同时生效（锁定又重置）。
export function usePressGestures(opts: {
  onDoubleClick: () => void
  onLongPress: () => void
  duration?: number
}) {
  const duration = opts.duration ?? 500
  const pressing = ref(false)
  let timer: ReturnType<typeof setTimeout> | null = null
  let longFiredAt = 0

  function cancel() {
    pressing.value = false
    if (timer !== null) {
      clearTimeout(timer)
      timer = null
    }
  }

  function onPointerdown(e: PointerEvent) {
    if (e.button !== 0) return
    ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
    pressing.value = true
    timer = setTimeout(() => {
      timer = null
      pressing.value = false
      longFiredAt = Date.now()
      opts.onLongPress()
    }, duration)
  }

  function onDblclick() {
    if (Date.now() - longFiredAt < 800) return
    opts.onDoubleClick()
  }

  return { pressing, onPointerdown, onPointerup: cancel, onPointercancel: cancel, onDblclick }
}
