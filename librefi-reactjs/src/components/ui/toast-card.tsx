import React, {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react"
import { XIcon } from "lucide-react"
import { Button } from "./button"
import { cn } from "#lib/utils"
import {
  type ToastData,
  type ToastPosition,
  type ToasterProps,
  defaultOptions,
  removeToast,
  toastVariants,
  defaultIcons,
  setInheritPosition
} from "./toast"

export function ToastCard({
  item,
  position,
  icons,
  y,
  scale,
  zIndex,
  hidden,
  onHeightChange,
  onUnmount,
}: {
  item: ToastData
  position: ToastPosition
  icons?: ToasterProps["icons"]
  y: number
  scale: number
  zIndex: number
  hidden: boolean
  onHeightChange: (id: string, height: number) => void
  onUnmount: (id: string) => void
}) {
  const duration = item.duration ?? defaultOptions.duration ?? 5000
  const autoDismiss = duration > 0 && duration !== Infinity
  const closeButton = item.closeButton ?? defaultOptions.closeButton ?? false
  const dismissible = item.dismissible ?? defaultOptions.dismissible ?? true

  const pausedRef = useRef(false)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const cardRef = useRef<HTMLDivElement | null>(null)

  // Mount/Animation state
  const [mounted, setMounted] = useState(false)
  const [isDragging, setIsDragging] = useState(false)
  const [dragX, setDragX] = useState(0)
  const startXRef = useRef(0)

  useEffect(() => {
    // Delay setting mount state slightly so initial offscreen positions render first, 
    // enabling the CSS transition to trigger as it slides in.
    const frame = requestAnimationFrame(() => {
      setMounted(true)
    })
    return () => cancelAnimationFrame(frame)
  }, [])

  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current)
      timerRef.current = null
    }
  }, [])

  const armTimer = useCallback(() => {
    clearTimer()
    if (!autoDismiss || pausedRef.current) return
    timerRef.current = setTimeout(() => removeToast(item.id), duration)
  }, [autoDismiss, clearTimer, duration, item.id])

  useEffect(() => {
    armTimer()
    return clearTimer
  }, [armTimer, clearTimer])

  const pause = useCallback(() => {
    pausedRef.current = true
    clearTimer()
  }, [clearTimer])

  const resume = useCallback(() => {
    pausedRef.current = false
    armTimer()
  }, [armTimer])

  useLayoutEffect(() => {
    const node = cardRef.current
    if (!node) return
    const report = () => onHeightChange(item.id, node.offsetHeight)
    report()
    if (typeof ResizeObserver === "undefined") return
    const observer = new ResizeObserver(report)
    observer.observe(node)
    return () => observer.disconnect()
  }, [item.id, onHeightChange])

  useEffect(() => {
    return () => onUnmount(item.id)
  }, [item.id, onUnmount])

  const isTop = position.startsWith("top")
  const isLeft = position.endsWith("left")
  const isCenter = position.endsWith("center")

  // Calculate raw positions for CSS Transforms
  const restY = isTop ? y : -y
  const offScreenY = isTop ? "-100%" : "100%"
  const enterY = isCenter ? `calc(${offScreenY} + ${restY}px)` : `${restY}px`
  const restYValue = isCenter ? `calc(0% + ${restY}px)` : `${restY}px`
  const enterX = isCenter ? "0px" : isLeft ? "-100%" : "100%"

  const currentX = mounted && isDragging ? `${dragX}px` : (mounted ? "0px" : enterX)
  const currentY = mounted ? restYValue : enterY
  const currentScale = (mounted && !hidden) ? scale : scale * 0.95
  const currentOpacity = (mounted && !hidden) ? 1 : 0

  const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!dismissible) return
    e.currentTarget.setPointerCapture(e.pointerId)
    startXRef.current = e.clientX
    setIsDragging(true)
  }

  const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!isDragging) return
    const delta = e.clientX - startXRef.current
    // Add drag elasticity 
    setDragX(delta * 0.5)
  }

  const onPointerUp = () => {
    if (!isDragging) return
    setIsDragging(false)
    if (Math.abs(dragX) > 100) {
      removeToast(item.id)
    } else {
      setDragX(0)
    }
  }

  const icon =
    item.icon ??
    (item.type !== "default"
      ? (icons?.[item.type] ?? defaultIcons[item.type])
      : null)

  return (
    <div
      ref={cardRef}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onMouseEnter={pause}
      onMouseLeave={resume}
      onFocus={pause}
      onBlur={resume}
      style={{
        position: "absolute",
        left: 0,
        width: "100%",
        [isTop ? "top" : "bottom"]: 0,
        pointerEvents: hidden ? "none" : "auto",
        zIndex,
        opacity: currentOpacity,
        transform: `translate3d(${currentX}, ${currentY}, 0) scale(${currentScale})`,
        transition: isDragging
          ? "none"
          : "transform 350ms cubic-bezier(0.22, 1, 0.36, 1), opacity 350ms cubic-bezier(0.22, 1, 0.36, 1)",
      }}
      className={cn(
        "cursor-default select-none outline-none touch-none",
        item.custom
          ? "overflow-visible"
          : cn(
            "flex items-start gap-3 overflow-hidden rounded-2xl border p-4 shadow-lg",
            toastVariants[item.type]
          ),
        item.className
      )}
    >
      {item.custom ? (
        item.custom
      ) : (
        <>
          {icon ? (
            <span className="shrink-0 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4">
              {icon}
            </span>
          ) : null}
          <div className="flex min-w-0 flex-1 flex-col gap-1">
            {item.title ? (
              <div className="text-sm font-medium">{item.title}</div>
            ) : null}
            {item.description ? (
              <div className="text-sm opacity-80">
                {item.description}
              </div>
            ) : null}
          </div>
          {item.action || item.cancel ? (
            <div className="flex shrink-0 items-center gap-2">
              {item.cancel ? (
                <Button
                  variant="ghost"
                  size="sm"
                  className="shrink-0"
                  onClick={(event) => {
                    setInheritPosition(item.position)
                    item.cancel?.onClick(event)
                    setInheritPosition(undefined)
                    removeToast(item.id)
                  }}
                >
                  {item.cancel.label}
                </Button>
              ) : null}
              {item.action ? (
                <Button
                  variant="outline"
                  size="sm"
                  className="shrink-0"
                  onClick={(event) => {
                    setInheritPosition(item.position)
                    item.action?.onClick(event)
                    setInheritPosition(undefined)
                    if (item.action?.dismiss) removeToast(item.id)
                  }}
                >
                  {item.action.label}
                </Button>
              ) : null}
            </div>
          ) : null}
          {closeButton ? (
            <button
              type="button"
              aria-label="Close toast"
              onClick={() => removeToast(item.id)}
              className="shrink-0 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:opacity-100 focus:outline-none"
            >
              <XIcon aria-hidden="true" className="size-4" />
            </button>
          ) : null}
        </>
      )}
    </div>
  )
}
