import React, {
  useState,
  useSyncExternalStore,
  useEffect,
  isValidElement,
} from "react"
import { createPortal } from "react-dom"
import { createRoot } from "react-dom/client"

import {
  CircleCheckIcon,
  InfoIcon,
  TriangleAlertIcon,
  OctagonXIcon,
  Loader2Icon,
} from "lucide-react"
import { ToastGroup } from "./toast-group"
import { cn } from "#lib/utils"

type ToastType =
  | "default"
  | "success"
  | "error"
  | "info"
  | "warning"
  | "loading"

type ToastPosition =
  | "top-left"
  | "top-center"
  | "top-right"
  | "bottom-left"
  | "bottom-center"
  | "bottom-right"

type ToastMessage = React.ReactNode

type ToastCustomRender = (id: string) => ToastMessage

interface ToastAction {
  label: React.ReactNode
  onClick: (event: React.MouseEvent<HTMLButtonElement>) => void
  dismiss?: boolean
}

interface ToastOptions {
  id?: string
  type?: ToastType
  title?: ToastMessage
  description?: ToastMessage
  custom?: ToastMessage | ToastCustomRender
  duration?: number
  position?: ToastPosition
  action?: ToastAction
  cancel?: ToastAction
  icon?: React.ReactNode
  closeButton?: boolean
  dismissible?: boolean
  onDismiss?: (id: string) => void
  className?: string
}

interface ToastData extends Omit<ToastOptions, "id" | "type" | "custom"> {
  id: string
  type: ToastType
  custom?: ToastMessage
  createdAt: number
}

interface ToasterProps {
  position?: ToastPosition
  duration?: number
  gap?: number
  offset?: number | Partial<Record<"top" | "right" | "bottom" | "left", number>>
  closeButton?: boolean
  dismissible?: boolean
  icons?: Partial<Record<Exclude<ToastType, "default">, React.ReactNode>>
  toastOptions?: Omit<ToastOptions, "id" | "title" | "type">
  className?: string
  autoMount?: boolean
}

interface PromiseToastData<Value> {
  loading?: ToastMessage | Omit<ToastOptions, "id" | "type">
  success?:
  | ToastMessage
  | Omit<ToastOptions, "id" | "type">
  | ((result: Value) => ToastMessage | Omit<ToastOptions, "id" | "type">)
  error?:
  | ToastMessage
  | Omit<ToastOptions, "id" | "type">
  | ((error: unknown) => ToastMessage | Omit<ToastOptions, "id" | "type">)
}

type ToastCustom = ToastMessage | ((id: string) => ToastMessage)

export const toastVariants: Record<ToastType, string> = {
  default: "bg-popover text-popover-foreground border-border",
  success: "bg-green-50 text-green-900 border-green-200 dark:bg-green-900 dark:text-green-50 dark:border-green-800",
  error: "bg-red-50 text-red-900 border-red-200 dark:bg-red-900 dark:text-red-50 dark:border-red-800",
  info: "bg-blue-50 text-blue-900 border-blue-200 dark:bg-blue-900 dark:text-blue-50 dark:border-blue-800",
  warning: "bg-yellow-50 text-yellow-900 border-yellow-200 dark:bg-yellow-900 dark:text-yellow-50 dark:border-yellow-800",
  loading: "bg-popover text-popover-foreground border-border",
}

export const defaultIcons: Record<
  Exclude<ToastType, "default">,
  React.ReactNode
> = {
  success: <CircleCheckIcon aria-hidden="true" />,
  info: <InfoIcon aria-hidden="true" />,
  warning: <TriangleAlertIcon aria-hidden="true" />,
  error: <OctagonXIcon aria-hidden="true" />,
  loading: <Loader2Icon className="animate-spin" aria-hidden="true" />,
}

export const MAX_VISIBLE_TOASTS = 3
export const COLLAPSED_PEEK = 14
export const SCALE_STEP = 0.06
export const TOAST_WIDTH = 380

let toastCounter = 0
export let defaultOptions: Partial<ToastOptions> = {}
export let inheritPosition: ToastOptions["position"] | undefined
export function setInheritPosition(val: typeof inheritPosition) {
  inheritPosition = val;
}
let toastState: ToastData[] = []
let registeredToasters = 0
let autoRoot: ReturnType<typeof createRoot> | null = null
let autoContainer: HTMLDivElement | null = null
let autoMounted = false
let autoConfig: ToasterProps = {}
const listeners = new Set<() => void>()

function emit() {
  for (const listener of listeners) listener()
}

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => {
    listeners.delete(listener)
  }
}

function getSnapshot() {
  return toastState
}

function ensureMount() {
  if (typeof document === "undefined") return
  if (registeredToasters > 0 || autoMounted) return

  autoMounted = true
  const container = document.createElement("div")
  autoContainer = container
  document.body.appendChild(container)
  autoRoot = createRoot(container)
  autoRoot.render(<Toaster autoMount {...autoConfig} />)
}

function configure(options: ToasterProps) {
  autoConfig = options
  defaultOptions = {
    position: options.position,
    duration: options.duration,
    closeButton: options.closeButton,
    dismissible: options.dismissible,
    ...options.toastOptions,
  }
  if (autoRoot && autoMounted) {
    autoRoot.render(<Toaster autoMount {...autoConfig} />)
  }
}

function generateId() {
  toastCounter += 1
  return `toast-${toastCounter}`
}

function isToastMessage(
  value: ToastMessage | ToastOptions
): boolean {
  return (
    typeof value === "string" ||
    typeof value === "number" ||
    isValidElement(value as React.ReactNode)
  )
}

export function removeToast(id: string) {
  const item = toastState.find((toastItem) => toastItem.id === id)
  item?.onDismiss?.(id)
  toastState = toastState.filter((toastItem) => toastItem.id !== id)
  emit()
}

function resolveData(
  value: ToastMessage | Omit<ToastOptions, "id" | "type">
): ToastOptions {
  return isToastMessage(value as ToastMessage)
    ? { title: value as ToastMessage }
    : (value as ToastOptions)
}

function add(
  input: ToastMessage | ToastOptions,
  options?: ToastOptions
): string {
  ensureMount()
  const isMessage = isToastMessage(input)
  const resolved = isMessage
    ? { title: input as ToastMessage }
    : (input as ToastOptions)
  const merged: ToastOptions = {
    ...defaultOptions,
    ...resolved,
    ...(isMessage ? options : undefined),
  }
  const explicitPosition = isMessage ? options?.position : resolved.position
  const position = explicitPosition ?? inheritPosition
  if (position !== undefined) merged.position = position
  inheritPosition = undefined
  const id = merged.id ?? generateId()
  const existing = toastState.find((toastItem) => toastItem.id === id)

  const next: ToastData = {
    id,
    type: merged.type ?? "default",
    title: merged.title,
    description: merged.description,
    custom:
      typeof merged.custom === "function"
        ? (merged.custom as (id: string) => ToastMessage)(id)
        : merged.custom,
    duration: merged.duration,
    position: merged.position,
    action: merged.action,
    cancel: merged.cancel,
    icon: merged.icon,
    closeButton: merged.closeButton,
    dismissible: merged.dismissible,
    onDismiss: merged.onDismiss,
    className: merged.className,
    createdAt: existing ? existing.createdAt : Date.now(),
  }

  toastState = existing
    ? toastState.map((toastItem) =>
      toastItem.id === id
        ? { ...toastItem, ...next, createdAt: toastItem.createdAt }
        : toastItem
    )
    : [...toastState, next]
  emit()
  return id
}

function update(id: string, options: Omit<ToastOptions, "id">) {
  const existing = toastState.find((toastItem) => toastItem.id === id)
  if (!existing) return
  toastState = toastState.map((toastItem) =>
    toastItem.id === id
      ? {
        ...toastItem,
        ...options,
        id,
        type: options.type ?? toastItem.type,
        custom:
          typeof options.custom === "function"
            ? (options.custom as ToastCustomRender)(id)
            : (options.custom ?? toastItem.custom),
      }
      : toastItem
  )
  emit()
}

function dismiss(id?: string) {
  if (!id) {
    toastState = []
    emit()
    return
  }
  removeToast(id)
}

function promise<Value>(
  promise: Promise<Value>,
  data: PromiseToastData<Value>,
  options?: Omit<ToastOptions, "id" | "title" | "type">
): Promise<Value> {
  const loading = resolveData(data.loading ?? "Loading...")
  const id = add({
    ...options,
    ...loading,
    type: "loading",
    duration: loading.duration ?? options?.duration ?? Infinity,
  })

  promise
    .then((result) => {
      const resolved = resolveData(
        typeof data.success === "function"
          ? data.success(result)
          : (data.success ?? "Done")
      )
      update(id, {
        ...options,
        ...resolved,
        type: "success",
        duration:
          resolved.duration ?? options?.duration ?? defaultOptions.duration ?? 5000,
      })
    })
    .catch((error) => {
      const resolved = resolveData(
        typeof data.error === "function"
          ? data.error(error)
          : (data.error ?? "Something went wrong")
      )
      update(id, {
        ...options,
        ...resolved,
        type: "error",
        duration:
          resolved.duration ?? options?.duration ?? defaultOptions.duration ?? 5000,
      })
    })

  return promise
}

function success(
  message: ToastMessage,
  options?: Omit<ToastOptions, "type">
) {
  return add(message, { ...options, type: "success" })
}

function error(message: ToastMessage, options?: Omit<ToastOptions, "type">) {
  return add(message, { ...options, type: "error" })
}

function info(message: ToastMessage, options?: Omit<ToastOptions, "type">) {
  return add(message, { ...options, type: "info" })
}

function warning(message: ToastMessage, options?: Omit<ToastOptions, "type">) {
  return add(message, { ...options, type: "warning" })
}

function loading(message: ToastMessage, options?: Omit<ToastOptions, "type">) {
  return add(message, {
    ...options,
    type: "loading",
    duration: options?.duration ?? Infinity,
  })
}

function custom(
  content: ToastCustom,
  options?: Omit<ToastOptions, "title" | "type">
) {
  return add({ custom: content, ...options })
}

const toast = {
  add,
  update,
  dismiss,
  close: dismiss,
  promise,
  success,
  error,
  info,
  warning,
  loading,
  custom,
  configure,
  Toaster: null as any,
}

export function getOffsetStyle(
  position: ToastPosition,
  offset: NonNullable<ToasterProps["offset"]>
): React.CSSProperties {
  const resolved =
    typeof offset === "number"
      ? { top: offset, right: offset, bottom: offset, left: offset }
      : { top: 16, right: 16, bottom: 16, left: 16, ...offset }

  const style: React.CSSProperties = {}
  if (position.startsWith("top")) style.top = resolved.top
  else style.bottom = resolved.bottom

  if (position.endsWith("left")) style.left = resolved.left
  else if (position.endsWith("right")) style.right = resolved.right
  else {
    style.left = "50%"
    style.transform = "translateX(-50%)"
  }
  return style
}

function groupToasts(
  toasts: ToastData[],
  defaultPosition: ToastPosition
): Map<ToastPosition, ToastData[]> {
  const groups = new Map<ToastPosition, ToastData[]>()
  for (const item of toasts) {
    const pos = item.position ?? defaultPosition
    const group = groups.get(pos)
    if (group) group.push(item)
    else groups.set(pos, [item])
  }
  return groups
}



function Toaster({
  position = "bottom-right",
  duration = 5000,
  gap = 12,
  offset = 16,
  closeButton = false,
  dismissible = true,
  icons,
  toastOptions,
  className,
  autoMount = false,
}: ToasterProps) {
  const [mounted, setMounted] = useState(false)
  const toasts = useSyncExternalStore(
    subscribe,
    getSnapshot,
    () => []
  )

  useEffect(() => {
    setMounted(true)
    registeredToasters += 1
    defaultOptions = { position, duration, closeButton, dismissible, ...toastOptions }

    if (!autoMount && autoRoot && autoMounted) {
      autoRoot.unmount()
      autoRoot = null
      autoMounted = false
      autoContainer?.remove()
      autoContainer = null
    }

    return () => {
      registeredToasters -= 1
    }
  }, [autoMount, position, duration, closeButton, dismissible, toastOptions])

  if (!mounted) return null

  const groups = groupToasts(toasts, position)

  return createPortal(
    <div className={cn("pointer-events-none fixed inset-0 z-100", className)}>
      {[...groups.entries()].map(([pos, items]) => (
        <ToastGroup
          key={pos}
          position={pos}
          items={items}
          icons={icons}
          offset={offset}
          gap={gap}
        />
      ))}
    </div>,
    document.body
  )
}

toast.Toaster = Toaster;

export {

  toast,

  configure,
  type ToasterProps,
  type ToastAction,
  type ToastData,
  type ToastMessage,
  type ToastOptions,
  type ToastPosition,
  type ToastType,
}