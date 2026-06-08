import { ref, readonly, watch } from "vue"
import { en } from "../locales/en"
import { zh } from "../locales/zh"

export type Locale = "en" | "zh"

const STORAGE_KEY = "evocode.locale"
const DEFAULT_LOCALE: Locale = "en"

function readInitial(): Locale {
  if (typeof window === "undefined") return DEFAULT_LOCALE
  const stored = window.localStorage.getItem(STORAGE_KEY) as Locale | null
  if (stored === "en" || stored === "zh") return stored
  return DEFAULT_LOCALE
}

const locale = ref<Locale>(readInitial())

const messages: Record<Locale, Record<string, string>> = { en, zh }

function t(key: string, params?: Record<string, string>): string {
  const msg = messages[locale.value]?.[key]
  if (msg == null) return key
  if (!params) return msg
  return msg.replace(/\{(\w+)\}/g, (_, p) => params[p] ?? `{${p}}`)
}

// Persist on change
if (typeof window !== "undefined") {
  watch(locale, (val) => {
    window.localStorage.setItem(STORAGE_KEY, val)
  })
}

export function useLocale() {
  function toggle() {
    locale.value = locale.value === "en" ? "zh" : "en"
  }

  function set(l: Locale) {
    locale.value = l
  }

  return {
    locale: readonly(locale),
    t,
    toggle,
    set,
  }
}
