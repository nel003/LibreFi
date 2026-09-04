import { useState, useCallback } from "react"
import {
  type ToastData,
  type ToastPosition,
  type ToasterProps,
  MAX_VISIBLE_TOASTS,
  COLLAPSED_PEEK,
  SCALE_STEP,
  TOAST_WIDTH,
  getOffsetStyle
} from "./toast"
import { ToastCard } from "./toast-card"

export function ToastGroup({
  position,
  items,
  icons,
  offset,
  gap,
}: {
  position: ToastPosition
  items: ToastData[]
  icons?: ToasterProps["icons"]
  offset: NonNullable<ToasterProps["offset"]>
  gap: number
}) {
  const [expanded, setExpanded] = useState(false)
  const [heights, setHeights] = useState<Record<string, number>>({})

  const setHeight = useCallback((id: string, height: number) => {
    setHeights((prev) => (prev[id] === height ? prev : { ...prev, [id]: height }))
  }, [])

  const removeHeight = useCallback((id: string) => {
    setHeights((prev) => {
      if (!(id in prev)) return prev
      const next = { ...prev }
      delete next[id]
      return next
    })
  }, [])

  const ordered = [...items].reverse()

  let cumulative = 0
  const offsets = ordered.map((item, index) => {
    const y = expanded
      ? cumulative
      : Math.min(index, MAX_VISIBLE_TOASTS - 1) * COLLAPSED_PEEK
    cumulative += (heights[item.id] ?? 0) + gap
    return y
  })
  const frontHeight = heights[ordered[0]?.id ?? ""] ?? 0
  const expandedHeight = cumulative > 0 ? cumulative - gap : 0

  return (
    <div
      onMouseEnter={() => setExpanded(true)}
      onMouseLeave={() => setExpanded(false)}
      onFocus={() => setExpanded(true)}
      onBlur={() => setExpanded(false)}
      aria-live="polite"
      className="pointer-events-none fixed"
      style={{
        ...getOffsetStyle(position, offset),
        width: `min(${TOAST_WIDTH}px, calc(100vw - 2rem))`,
        height: expanded ? expandedHeight : frontHeight,
        transition: "height 350ms cubic-bezier(0.22, 1, 0.36, 1)",
      }}
    >
      {ordered.map((item, index) => (
        <ToastCard
          key={item.id}
          item={item}
          position={position}
          icons={icons}
          y={offsets[index]}
          scale={
            expanded ? 1 : 1 - Math.min(index, MAX_VISIBLE_TOASTS) * SCALE_STEP
          }
          zIndex={ordered.length - index}
          hidden={!expanded && index >= MAX_VISIBLE_TOASTS}
          onHeightChange={setHeight}
          onUnmount={removeHeight}
        />
      ))}
    </div>
  )
}
