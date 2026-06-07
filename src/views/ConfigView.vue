<template>
  <div class="config-view">
    <header class="page-head fade-up">
      <div>
        <div class="eyebrow">
          <span class="dot" />
          <span>Settings</span>
        </div>
        <h1>Configuration</h1>
        <p class="muted-3">Pick a provider preset, fill in credentials, save. You can store many providers and switch the active one anytime.</p>
      </div>
      <div class="head-actions">
        <a-button @click="$router.push('/')" class="ghost">
          <template #icon><LeftOutlined /></template>
          Back to Dashboard
        </a-button>
      </div>
    </header>

    <a-alert
      v-if="msg.text"
      :type="msg.type"
      :message="msg.text"
      show-icon
      closable
      class="fade-up alert"
      @close="msg.text = ''"
    />

    <a-tabs v-model:activeKey="activeKey" class="config-tabs fade-up">
      <!-- ============== Connection ============== -->
      <a-tab-pane key="provider" tab="Connection">
        <!-- Top: provider chips for switch / add / remove -->
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Providers</div>
              <div class="panel-sub muted-3">Add as many as you like. The active one is what the bridge uses.</div>
            </div>
            <a-tag v-if="activeId" class="active-tag">Active: {{ activeId }}</a-tag>
          </div>

          <div class="prov-chips">
            <button
              v-for="id in providerIds"
              :key="id"
              class="chip"
              :class="{ active: id === activeId }"
              @click="setActive(id)"
            >
              <span class="chip-dot" :class="{ on: id === activeId }" />
              <span class="chip-name">{{ id }}</span>
              <a-popconfirm
                v-if="providerIds.length > 1"
                title="Remove this provider?"
                ok-text="Yes"
                cancel-text="No"
                @confirm.stop="removeProvider(id)"
              >
                <CloseCircleFilled class="chip-x" @click.stop />
              </a-popconfirm>
            </button>

            <div class="chip add-chip">
              <a-input
                v-model:value="newProviderName"
                placeholder="new provider id"
                size="small"
                @press-enter="addProvider"
              />
              <a-button size="small" type="primary" :disabled="!newProviderName" @click="addProvider">
                <template #icon><PlusOutlined /></template>
                Add
              </a-button>
            </div>
          </div>
        </div>

        <!-- Wire API Switch - replaces the Start from a template panel -->
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Wire API</div>
              <div class="panel-sub muted-3">Choose the protocol. Model and Base URL are filled separately below.</div>
            </div>
          </div>
          <a-segmented
            v-model:value="activePresetKey"
            :options="wireOptions"
            block
            @change="onWireApiChange"
          />
        </div>

        <!-- Bottom: form for the active provider -->
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Settings for Provider</div>
              <div class="panel-sub muted-3">These values are saved under <code class="mono">[providers.{{ activeId || '...' }}]</code>.</div>
            </div>
            <a-button size="small" :disabled="!activeId" @click="resetForm">
              <template #icon><ReloadOutlined /></template>
              Reset
            </a-button>
          </div>

          <a-empty
            v-if="!activeId"
            description="Add a provider above to get started."
            class="empty-block"
          />

          <a-form v-else layout="vertical" class="form" :model="formState">
            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item label="Model" required>
                  <a-input v-model:value="formState.model" placeholder="e.g. MiniMax-M3" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item label="Wire API">
                  <a-tooltip title="Which API protocol this upstream provider speaks">
                    <a-select v-model:value="formState.wireApi" @change="onWireApiSelectChange">
                      <a-select-option value="anthropic">
                        <span class="opt-row"><span class="dot purple" /> Anthropic (/v1/messages)</span>
                      </a-select-option>
                      <a-select-option value="chat_completions">
                        <span class="opt-row"><span class="dot blue" /> Chat Completions (/v1/chat)</span>
                      </a-select-option>
                      <a-select-option value="openai">
                        <span class="opt-row"><span class="dot cyan" /> OpenAI Responses (/responses)</span>
                      </a-select-option>
                    </a-select>
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>

            <a-form-item label="Base URL" required>
              <a-input v-model:value="formState.baseUrl" placeholder="https://api.example.com/v1" />
            </a-form-item>

            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item label="API Key">
                  <a-input-password v-model:value="formState.apiKey" placeholder="Your API key" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item label="API Key Header">
                  <a-tooltip title="The HTTP header name for the API key, e.g. X-Api-Key or Authorization">
                    <a-input v-model:value="formState.apiKeyHeader" placeholder="X-Api-Key" />
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>
          </a-form>
        </div>
      </a-tab-pane>

      <!-- ============== Limits ============== -->
      <a-tab-pane key="limits" tab="Model limits">
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">Context & auto-compact</div>
              <div class="panel-sub muted-3">Drag the sliders or click the preset values.</div>
            </div>
          </div>

          <!-- Context Window Slider - Draggable -->
          <div class="slider-block">
            <div class="slider-head">
              <div>
                <div class="slider-label">Context window</div>
                <div class="slider-value mono">{{ contextLabel(limits.contextWindow) }}<span class="muted-3"> tokens</span></div>
              </div>
              <a-tag class="active-tag">{{ contextLabel(limits.contextWindow) }}</a-tag>
            </div>
            <div
              class="slider-rail"
              ref="ctxRail"
              @mousedown="(e) => startDrag(e, 'context')"
              @touchstart.passive="(e) => startDrag(e, 'context')"
            >
              <span
                v-for="(p, i) in LIMIT_PRESETS"
                :key="`ctx-tick-${p.key}`"
                class="slider-tick"
                :style="{ left: tickLeft(i) + '%' }"
                :title="p.name + ' - ' + p.label"
              />
              <div class="slider-fill" :style="{ width: fillPercent + '%' }" />
              <div class="slider-thumb" :style="{ left: fillPercent + '%' }">
                <span class="thumb-tip">{{ contextLabel(limits.contextWindow) }}</span>
              </div>
            </div>
            <div class="slider-stops">
              <button
                v-for="p in LIMIT_PRESETS"
                :key="`ctx-${p.key}`"
                type="button"
                class="slider-stop"
                :class="{ active: limits.contextWindow === p.context }"
                @click="applyLimitPreset(p.key)"
              >
                <span class="stop-dot" :style="{ background: p.color }" />
                <span class="stop-name">{{ p.name }}</span>
                <span class="stop-label mono">{{ p.label }}</span>
              </button>
            </div>
          </div>

          <!-- Auto Compact Limit Slider - Percentage based, follows Context window -->
          <div class="slider-block">
            <div class="slider-head">
              <div>
                <div class="slider-label">Auto compact limit</div>
                <div class="slider-value mono">{{ compactRatio }}%<span class="muted-3"> of {{ contextLabel(limits.contextWindow) }}</span></div>
              </div>
              <a-tag class="active-tag compact-tag">≈ {{ contextLabel(compactTokens) }} tokens</a-tag>
            </div>
            <div
              class="slider-rail"
              ref="ratioRail"
              @mousedown="(e) => startDrag(e, 'ratio')"
              @touchstart.passive="(e) => startDrag(e, 'ratio')"
            >
              <span
                v-for="t in RATIO_TICKS"
                :key="`ratio-tick-${t}`"
                class="slider-tick"
                :style="{ left: t + '%' }"
                :title="t + '%'"
              />
              <div class="slider-fill compact" :style="{ width: compactRatio + '%' }" />
              <div class="slider-thumb" :style="{ left: compactRatio + '%' }">
                <span class="thumb-tip">{{ compactRatio }}%</span>
              </div>
            </div>
            <!-- No quick selection buttons - removed as requested -->
          </div>
        </div>
      </a-tab-pane>
    </a-tabs>

    <footer class="actions fade-up">
      <a-button
        type="primary"
        size="large"
        :loading="saving"
        :disabled="!activeId"
        @click="handleSave"
      >
        <template #icon><SaveOutlined /></template>
        Save Config
      </a-button>
      <a-button
        size="large"
        :disabled="!activeId"
        @click="handleSyncToCodex"
      >
        <template #icon><SyncOutlined /></template>
        Sync to Codex
      </a-button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, onMounted, onUnmounted } from "vue"
import {
  LeftOutlined,
  PlusOutlined,
  SaveOutlined,
  SyncOutlined,
  ReloadOutlined,
  CloseCircleFilled,
  ApiOutlined,
  MessageOutlined,
  ThunderboltOutlined,
  RocketOutlined,
  FireOutlined,
} from "@ant-design/icons-vue"
import { readConfig, writeConfig, syncToCodex } from "../api/bridge"

interface Provider {
  providerId: string
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
}

interface Limits {
  contextWindow: number
  compactLimit: number
}

const PRESETS: Array<{
  key: string
  name: string
  tag: string
  desc: string
  baseUrl: string
  color: string
  icon: any
  values: Partial<Provider>
}> = [
  {
    key: "anthropic",
    name: "Anthropic",
    tag: "/v1/messages",
    desc: "Anthropic Messages protocol.",
    baseUrl: "https://api.anthropic.com",
    color: "linear-gradient(135deg, #d97757, #b6553a)",
    icon: MessageOutlined,
    values: { wireApi: "anthropic", apiKeyHeader: "X-Api-Key" },
  },
  {
    key: "chat_completions",
    name: "OpenAI Chat",
    tag: "/v1/chat/completions",
    desc: "Standard Chat Completions.",
    baseUrl: "https://api.openai.com/v1",
    color: "linear-gradient(135deg, #10a37f, #0d8a6b)",
    icon: ApiOutlined,
    values: { wireApi: "chat_completions", apiKeyHeader: "Authorization" },
  },
  {
    key: "openai",
    name: "OpenAI Responses",
    tag: "/responses",
    desc: "OpenAI Responses protocol.",
    baseUrl: "https://api.openai.com/v1",
    color: "linear-gradient(135deg, #6366f1, #4338ca)",
    icon: RocketOutlined,
    values: { wireApi: "openai", apiKeyHeader: "Authorization" },
  },
]

const LIMIT_PRESETS = [
  { key: "128k",  name: "Small",    label: "128K",  context: 128_000,  compact: 112_000,  color: "#34d399" },
  { key: "256k",  name: "Standard", label: "256K",  context: 256_000,  compact: 220_000,  color: "#60a5fa" },
  { key: "512k",  name: "Large",    label: "512K",  context: 512_000,  compact: 460_000,  color: "#a78bfa" },
  { key: "1m",    name: "Huge",     label: "1M",    context: 1_000_000, compact: 900_000, color: "#f472b6" },
]

const CONTEXT_MIN = 64_000
const CONTEXT_MAX = 1_048_576
const DEFAULT_CONTEXT_WINDOW = 256_000
const DEFAULT_COMPACT_LIMIT = 220_000

const RATIO_TICKS = [10, 25, 50, 75, 90]

const activeKey = ref("provider")
const providerIds = ref<string[]>([])
const newProviderName = ref("")
const saving = ref(false)
const msg = reactive({ text: "", type: "success" as "success" | "error" | "warning" })

// Each provider keeps its own fields
const providers = reactive<Record<string, Provider>>({})
const limits = reactive<Limits>({
  contextWindow: DEFAULT_CONTEXT_WINDOW,
  compactLimit: DEFAULT_COMPACT_LIMIT,
})
const activeId = ref("")

// Form state
const formState = reactive<Provider>({
  providerId: "",
  wireApi: "anthropic",
  baseUrl: "",
  model: "",
  apiKey: "",
  apiKeyHeader: "X-Api-Key",
})

// Wire API switch state - bidirectional sync
const activePresetKey = ref<string>("anthropic")
const wireOptions = computed(() =>
  PRESETS.map((p) => ({ value: p.key, label: p.name })),
)

// Bidirectional sync functions
function onWireApiChange(key: string | number) {
  const preset = PRESETS.find((p) => p.key === key)
  if (!preset) return
  if (preset.values.wireApi) formState.wireApi = preset.values.wireApi
  if (preset.values.apiKeyHeader) formState.apiKeyHeader = preset.values.apiKeyHeader
  if (activeId.value) providers[activeId.value] = { ...formState, providerId: activeId.value }
}

function onWireApiSelectChange() {
  // Sync the segmented switch when select changes
  const matchingPreset = PRESETS.find((p) => p.values.wireApi === formState.wireApi)
  if (matchingPreset && matchingPreset.key !== activePresetKey.value) {
    activePresetKey.value = matchingPreset.key
  }
}

// Slider drag handling
const ctxRail = ref<HTMLElement | null>(null)
const ratioRail = ref<HTMLElement | null>(null)
let dragging: 'context' | 'ratio' | null = null
let onMove: ((e: MouseEvent | TouchEvent) => void) | null = null
let onUp: (() => void) | null = null

function railPercent(rail: HTMLElement, clientX: number): number {
  const rect = rail.getBoundingClientRect()
  const x = clientX - rect.left
  return Math.min(100, Math.max(0, (x / rect.width) * 100))
}

function applyDrag(target: 'context' | 'ratio', clientX: number) {
  if (target === 'context') {
    const rail = ctxRail.value
    if (!rail) return
    const pct = railPercent(rail, clientX)
    let snapped: number | null = null
    for (const p of LIMIT_PRESETS) {
      const pp = ((p.context - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100
      if (Math.abs(pp - pct) < 6) { snapped = p.context; break }
    }
    if (snapped == null) {
      const newValue = CONTEXT_MIN + (CONTEXT_MAX - CONTEXT_MIN) * (pct / 100)
      setContextWindow(newValue)
    } else {
      setContextWindow(snapped)
    }
  } else {
    const rail = ratioRail.value
    if (!rail) return
    const pct = railPercent(rail, clientX)
    setCompactRatio(pct)
  }
}

function setContextWindow(value: number) {
  const prev = limits.contextWindow
  const next = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, Math.round(value)))
  limits.contextWindow = next
  // Auto-set compact to 80% of new context window
  limits.compactLimit = Math.round(next * 0.8)
}

function setCompactRatio(percent: number) {
  const p = Math.min(100, Math.max(0, percent))
  limits.compactLimit = Math.round(limits.contextWindow * p / 100)
}

function startDrag(e: MouseEvent | TouchEvent, target: 'context' | 'ratio') {
  const clientX = 'touches' in e ? e.touches[0].clientX : (e as MouseEvent).clientX
  applyDrag(target, clientX)
  dragging = target
  onMove = (ev: MouseEvent | TouchEvent) => {
    if (!dragging) return
    const x = 'touches' in ev ? ev.touches[0].clientX : (ev as MouseEvent).clientX
    applyDrag(dragging, x)
  }
  onUp = () => stopDrag()
  window.addEventListener('mousemove', onMove as any)
  window.addEventListener('touchmove', onMove as any, { passive: true } as any)
  window.addEventListener('mouseup', onUp)
  window.addEventListener('touchend', onUp)
}

function stopDrag() {
  dragging = null
  if (onMove) {
    window.removeEventListener('mousemove', onMove as any)
    window.removeEventListener('touchmove', onMove as any)
  }
  if (onUp) {
    window.removeEventListener('mouseup', onUp)
    window.removeEventListener('touchend', onUp)
  }
  onMove = null
  onUp = null
}

onUnmounted(stopDrag)

// Computed values for limits display
const compactRatio = computed(() => {
  if (!limits.contextWindow) return 0
  return Math.round((limits.compactLimit / limits.contextWindow) * 1000) / 10
})
const compactTokens = computed(() => Math.round(limits.contextWindow * compactRatio.value / 100))
const fillPercent = computed(() => {
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, limits.contextWindow || CONTEXT_MIN))
  return Math.round(((v - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
})

function contextLabel(n: number) {
  if (!n) return "0"
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(n % 1_000_000 === 0 ? 0 : 1).replace(/\.0$/, "") + "M"
  if (n >= 1_000) return Math.round(n / 1_000) + "K"
  return String(n)
}

function tickLeft(i: number) {
  const p = LIMIT_PRESETS[i]
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, p.context))
  return Math.round(((v - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
}

function applyLimitPreset(key: string) {
  const p = LIMIT_PRESETS.find((x) => x.key === key)
  if (!p) return
  limits.contextWindow = p.context
  limits.compactLimit = p.compact
}

// Provider management functions
function snapshotActive() {
  if (!activeId.value) return
  providers[activeId.value] = { ...formState, providerId: activeId.value }
}

function loadActiveIntoForm() {
  const id = activeId.value
  if (!id || !providers[id]) {
    Object.assign(formState, { providerId: "", wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key" })
    return
  }
  Object.assign(formState, providers[id])
  // Sync the segmented switch
  const matchingPreset = PRESETS.find((p) => p.values.wireApi === formState.wireApi)
  if (matchingPreset) activePresetKey.value = matchingPreset.key
}

function setActive(id: string) {
  snapshotActive()
  activeId.value = id
  loadActiveIntoForm()
}

function addProvider() {
  const name = newProviderName.value.trim()
  if (!name) return
  if (!providerIds.value.includes(name)) providerIds.value.push(name)
  if (!providers[name]) {
    providers[name] = {
      providerId: name,
      wireApi: "anthropic",
      baseUrl: "",
      model: "",
      apiKey: "",
      apiKeyHeader: "X-Api-Key",
    }
  }
  newProviderName.value = ""
  setActive(name)
}

function removeProvider(id: string) {
  const idx = providerIds.value.indexOf(id)
  if (idx > -1) providerIds.value.splice(idx, 1)
  delete providers[id]
  if (activeId.value === id) {
    activeId.value = providerIds.value[0] || ""
    loadActiveIntoForm()
  }
}

function resetForm() {
  formState.model = ""
  formState.baseUrl = ""
  formState.apiKey = ""
  formState.wireApi = "anthropic"
  formState.apiKeyHeader = "X-Api-Key"
  if (activeId.value) providers[activeId.value] = { ...formState, providerId: activeId.value }
  const matchingPreset = PRESETS.find((p) => p.values.wireApi === formState.wireApi)
  if (matchingPreset) activePresetKey.value = matchingPreset.key
}

function parseConfig(text: string) {
  providerIds.value = []
  for (const k of Object.keys(providers)) delete providers[k]

  if (!text) {
    activeId.value = ""
    return
  }

  const lines = text.split("\n")
  let currentProvider = ""
  let inProviderSection = false
  let activeFromTop = ""

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith("#")) continue

    if (trimmed.startsWith("provider = ")) {
      activeFromTop = trimmed.replace("provider = ", "").replace(/"/g, "")
    } else if (trimmed.startsWith("model_context_window = ")) {
      const v = parseInt(trimmed.replace("model_context_window = ", ""))
      if (!isNaN(v)) limits.contextWindow = v
    } else if (trimmed.startsWith("model_auto_compact_token_limit = ")) {
      const v = parseInt(trimmed.replace("model_auto_compact_token_limit = ", ""))
      if (!isNaN(v)) limits.compactLimit = v
    } else if (trimmed.startsWith("[providers.")) {
      currentProvider = trimmed.replace("[providers.", "").replace("]", "")
      inProviderSection = true
      if (!providerIds.value.includes(currentProvider)) providerIds.value.push(currentProvider)
      if (!providers[currentProvider]) {
        providers[currentProvider] = {
          providerId: currentProvider,
          wireApi: "anthropic",
          baseUrl: "",
          model: "",
          apiKey: "",
          apiKeyHeader: "X-Api-Key",
        }
      }
    } else if (trimmed.startsWith("[") && trimmed !== "[providers." + currentProvider + "]") {
      inProviderSection = false
    } else if (inProviderSection) {
      const p = providers[currentProvider]
      if (!p) continue
      if (trimmed.startsWith("wire_api = ")) p.wireApi = trimmed.replace("wire_api = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("base_url = ")) p.baseUrl = trimmed.replace("base_url = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("model = ")) p.model = trimmed.replace("model = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("api_key = ")) p.apiKey = trimmed.replace("api_key = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("api_key_header = ")) p.apiKeyHeader = trimmed.replace("api_key_header = \"", "").replace(/"/g, "")
    }
  }

  activeId.value = activeFromTop && providers[activeFromTop] ? activeFromTop : (providerIds.value[0] || "")
  loadActiveIntoForm()
}

function buildConfig(): string {
  snapshotActive()
  const ctx = limits.contextWindow || DEFAULT_CONTEXT_WINDOW
  const compact = limits.compactLimit || DEFAULT_COMPACT_LIMIT
  const active = activeId.value

  const blocks: string[] = [
    "# evocode bridge config",
    "# Read by evocode-cli, not by upstream Codex directly.",
    "",
    `provider = "${active}"`,
    "",
    `model_context_window = ${ctx}`,
    `model_auto_compact_token_limit = ${compact}`,
    "",
  ]

  for (const id of providerIds.value) {
    const p = providers[id] || {
      providerId: id, wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key",
    }
    blocks.push(
      `[providers.${id}]`,
      `wire_api = "${p.wireApi}"`,
      `base_url = "${p.baseUrl}"`,
      `model = "${p.model}"`,
      `api_key = "${p.apiKey}"`,
      `api_key_header = "${p.apiKeyHeader}"`,
      "",
    )
  }
  return blocks.join("\n").replace(/\n+$/, "\n")
}

async function handleSave() {
  if (!activeId.value) {
    msg.text = "Add a provider first."
    msg.type = "error"
    return
  }
  snapshotActive()
  const cur = providers[activeId.value]
  if (!cur?.model || !cur?.baseUrl) {
    msg.text = `Fill in Model and Base URL for "${activeId.value}".`
    msg.type = "error"
    return
  }
  saving.value = true
  try {
    await writeConfig(buildConfig())
    msg.text = "Config saved! Restart the bridge to apply changes."
    msg.type = "success"
    setTimeout(() => { msg.text = "" }, 4000)
  } catch (e: any) {
    msg.text = "Failed to save: " + (e?.message || String(e))
    msg.type = "error"
  } finally {
    saving.value = false
  }
}

async function handleSyncToCodex() {
  if (!activeId.value) {
    msg.text = "Add a provider first."
    msg.type = "error"
    return
  }
  snapshotActive()
  const cur = providers[activeId.value]
  saving.value = true
  try {
    await syncToCodex(
      activeId.value,
      cur.model,
      cur.baseUrl,
      cur.apiKey,
      cur.apiKeyHeader,
      cur.wireApi,
    )
    msg.text = "Synced to .codex/config.toml!"
    msg.type = "success"
    setTimeout(() => { msg.text = "" }, 4000)
  } catch (e: any) {
    msg.text = "Failed to sync: " + (e?.message || String(e))
    msg.type = "error"
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    const text = await readConfig()
    parseConfig(text)
  } catch {}
})
</script>

<style scoped>
.config-view { display: flex; flex-direction: column; gap: 18px; }

.page-head {
  display: flex; align-items: flex-end; justify-content: space-between;
  gap: 16px; flex-wrap: wrap;
}
.eyebrow {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 4px 10px; border-radius: 999px;
  background: var(--bg-elev-3); color: var(--text-3); font-size: 12px; width: max-content;
  border: 1px solid var(--border);
}
.eyebrow .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--info); box-shadow: 0 0 8px var(--info); }

.page-head h1 { font-size: 24px; font-weight: 700; color: var(--text-1); margin: 6px 0 2px; }
.page-head p { color: var(--text-3); max-width: 60ch; }
.head-actions { display: flex; gap: 8px; }
.ghost { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.ghost:hover { border-color: var(--border-strong); }

.alert { border-radius: var(--r-md); }

.config-tabs :deep(.ant-tabs-nav) { margin-bottom: 14px; }
.config-tabs :deep(.ant-tabs-tab) { padding: 8px 4px; color: var(--text-3); }
.config-tabs :deep(.ant-tabs-tab:hover) { color: var(--text-1); }
.config-tabs :deep(.ant-tabs-tab-active .ant-tabs-tab-btn) { color: var(--text-1) !important; }
.config-tabs :deep(.ant-tabs-ink-bar) { background: linear-gradient(90deg, #4d7dff, #8b5cf6); height: 3px; border-radius: 2px; }

.panel { padding: 20px 22px; display: flex; flex-direction: column; gap: 14px; }
.panel-head {
  display: flex; align-items: flex-start; justify-content: space-between;
  gap: 12px;
}
.panel-title { font-size: 15px; font-weight: 600; color: var(--text-1); }
.panel-sub { font-size: 12.5px; margin-top: 2px; }
.active-tag { border-radius: 999px; }

.form :deep(.ant-form-item-label > label) { color: var(--text-2); font-size: 12.5px; }

.opt-row { display: inline-flex; align-items: center; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.blue { background: #60a5fa; box-shadow: 0 0 8px #60a5fa; }
.dot.purple { background: #a78bfa; box-shadow: 0 0 8px #a78bfa; }
.dot.cyan { background: #22d3ee; box-shadow: 0 0 8px #22d3ee; }

.empty-block { padding: 24px 0; }

/* Provider chips */
.prov-chips { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.chip {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 6px 10px 6px 8px;
  border-radius: 999px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  color: var(--text-2);
  cursor: pointer;
  transition: border-color .15s ease, color .15s ease, background .15s ease;
  font-size: 13px;
}
.chip:hover { border-color: var(--border-strong); color: var(--text-1); }
.chip.active {
  color: var(--text-1);
  background: linear-gradient(135deg, rgba(77,125,255,0.18), rgba(139,92,246,0.10));
  border-color: rgba(77,125,255,0.5);
  box-shadow: 0 0 0 1px rgba(77,125,255,0.18);
}
.chip-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--text-4); }
.chip-dot.on { background: var(--ok); box-shadow: 0 0 8px var(--ok); }
.chip-x { color: var(--text-4); padding: 2px; }
.chip-x:hover { color: var(--err); }
.add-chip { background: transparent; border-style: dashed; padding: 4px 6px 4px 8px; gap: 6px; }
.add-chip :deep(.ant-input) { background: transparent; }

/* Slider styles */
.slider-block { display: flex; flex-direction: column; gap: 12px; }
.slider-head {
  display: flex; align-items: flex-end; justify-content: space-between;
  gap: 12px;
}
.slider-label { font-size: 12.5px; color: var(--text-3); margin-bottom: 2px; }
.slider-value { font-size: 22px; font-weight: 700; color: var(--text-1); }
.compact-tag { color: var(--text-2); }

.slider-rail {
  position: relative;
  height: 12px;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--bg-elev-3), var(--bg-elev-2));
  border: 1px solid var(--border);
  margin: 22px 6px 12px;
  cursor: pointer;
  user-select: none;
  touch-action: none;
}
.slider-fill {
  position: absolute; left: 0; top: 0; bottom: 0;
  background: linear-gradient(90deg, #4d7dff, #8b5cf6);
  border-radius: 999px;
  box-shadow: 0 0 12px rgba(77,125,255,0.45);
  transition: width .08s linear;
}
.slider-fill.compact { background: linear-gradient(90deg, #34d399, #22d3ee); box-shadow: 0 0 10px rgba(52,211,153,0.35); }
.slider-thumb {
  position: absolute; top: 50%;
  width: 22px; height: 22px;
  border-radius: 50%;
  background: white;
  border: 3px solid #4d7dff;
  transform: translate(-50%, -50%);
  box-shadow: 0 4px 14px rgba(77,125,255,0.55);
  cursor: grab;
  transition: left .08s linear, box-shadow .15s ease, transform .15s ease;
  z-index: 2;
}
.slider-thumb:hover { box-shadow: 0 6px 18px rgba(77,125,255,0.65); }
.slider-thumb:active { cursor: grabbing; transform: translate(-50%, -50%) scale(1.08); }
.thumb-tip {
  position: absolute;
  bottom: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  padding: 2px 8px;
  border-radius: 6px;
  background: var(--bg-elev-3);
  border: 1px solid var(--border-strong);
  color: var(--text-1);
  font-size: 11px;
  white-space: nowrap;
  font-weight: 600;
  pointer-events: none;
  opacity: 0;
  transition: opacity .15s ease;
}
.slider-thumb:hover .thumb-tip,
.slider-thumb:active .thumb-tip { opacity: 1; }
.slider-tick {
  position: absolute; top: -4px;
  width: 2px; height: 16px;
  background: var(--border-strong);
  transform: translateX(-50%);
  border-radius: 2px;
}
.slider-stops {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px;
  margin-top: 4px;
}
.slider-stop {
  display: flex; flex-direction: column; align-items: flex-start; gap: 2px;
  padding: 10px 12px;
  border-radius: var(--r-md);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  color: var(--text-2);
  cursor: pointer;
  transition: border-color .15s ease, color .15s ease, transform .15s ease;
}
.slider-stop:hover { border-color: var(--border-strong); color: var(--text-1); transform: translateY(-1px); }
.slider-stop.active {
  color: var(--text-1);
  border-color: rgba(77,125,255,0.5);
  background: linear-gradient(135deg, rgba(77,125,255,0.12), rgba(139,92,246,0.06));
  box-shadow: 0 0 0 1px rgba(77,125,255,0.18);
}
.stop-dot { width: 8px; height: 8px; border-radius: 50%; box-shadow: 0 0 6px currentColor; }
.stop-name { font-size: 12px; }
.stop-label { font-size: 13px; font-weight: 700; color: var(--text-1); }

.actions {
  position: sticky; bottom: 16px; z-index: 5;
  display: flex; gap: 10px; justify-content: flex-end; flex-wrap: wrap;
  padding: 12px 16px; border-radius: var(--r-lg);
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  -webkit-backdrop-filter: blur(14px) saturate(140%);
  box-shadow: var(--shadow-md);
}

@media (max-width: 720px) {
  .actions { justify-content: stretch; }
  .actions .ant-btn { flex: 1; }
  .slider-stops { grid-template-columns: repeat(2, 1fr); }
}
</style>