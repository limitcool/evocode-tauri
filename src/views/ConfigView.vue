<template>
  <div class="config-view">
    <header class="page-head fade-up">
      <div>
        <div class="eyebrow">
          <span class="dot" />
          <span>{{ t("config.settings") }}</span>
        </div>
        <h1>{{ t("config.title") }}</h1>
        <p class="muted-3">{{ t("config.desc") }}</p>
      </div>
      <div class="head-actions">
        <a-button @click="$router.push('/')" class="ghost">
          <template #icon><LeftOutlined /></template>
          {{ t("config.back") }}
        </a-button>
      </div>
    </header>

    <a-tabs v-model:activeKey="activeKey" class="config-tabs fade-up">
      <!-- ============== General ============== -->
      <a-tab-pane key="general" :tab="t('config.tab.general')">
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">{{ t("config.general.title") }}</div>
              <div class="panel-sub muted-3">{{ t("config.general.desc") }}</div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-meta">
              <div class="setting-name">{{ t("config.autostart.title") }}</div>
              <div class="setting-desc muted-3">{{ t("config.autostart.desc") }}</div>
            </div>
            <a-switch
              :checked="autostartEnabled"
              :loading="autostartLoading"
              @change="onAutostartChange"
            />
          </div>

          <a-divider class="row-divider" />

          <div class="setting-row">
            <div class="setting-meta">
              <div class="setting-name">{{ t("config.configdir.title") }}</div>
              <div class="setting-desc muted-3">{{ configDirHint }}</div>
            </div>
            <a-button @click="openConfigDir" :loading="openingDir">
              <template #icon><FolderOpenOutlined /></template>
              {{ t("config.configdir.open") }}
            </a-button>
          </div>
        </div>
      </a-tab-pane>

      <!-- ============== Connection ============== -->
      <a-tab-pane key="provider" :tab="t('config.tab.connection')">
        <!-- Top: provider chips for switch / add / remove -->
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">{{ t("config.providers") }}</div>
              <div class="panel-sub muted-3">{{ t("config.providers.desc") }}</div>
            </div>
            <a-tag v-if="activeId" class="active-tag">{{ t("config.providers.active") }} {{ activeId }}</a-tag>
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
                v-if="providerIds.length > 0"
                :title="t('config.providers.remove_title')"
                :ok-text="t('config.providers.ok')"
                :cancel-text="t('config.providers.cancel')"
                @confirm.stop="removeProvider(id)"
              >
                <CloseCircleFilled class="chip-x" @click.stop />
              </a-popconfirm>
            </button>

            <div class="chip add-chip">
              <a-input
                v-model:value="newProviderName"
                :placeholder="t('config.providers.placeholder')"
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

        <!-- Wire API Switch -->
        <div class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">{{ t("config.wire_api") }}</div>
              <div class="panel-sub muted-3">{{ t("config.wire_api.desc") }}</div>
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
              <div class="panel-title">{{ t("config.form.title") }}</div>
              <div class="panel-sub muted-3">{{ t("config.form.desc") }} <code class="mono">[providers.{{ activeId || '...' }}]</code>.</div>
            </div>
            <div class="head-actions">
              <a-button
                size="small"
                :disabled="!activeId"
                @click="resetForm"
              >
                <template #icon><ReloadOutlined /></template>
                Reset
              </a-button>
            </div>
          </div>

          <a-alert
            v-if="connResult"
            class="conn-alert"
            :type="connResult.ok ? 'success' : 'error'"
            :message="connResult.message"
            :description="t('config.form.test_latency') + ': ' + connResult.latency_ms + ' ms'"
            show-icon
            closable
            @close="connResult = null"
          />

          <a-empty
            v-if="!activeId"
            :description="t('config.form.empty')"
            class="empty-block"
          />

          <a-form v-else layout="vertical" class="form" :model="formState">
            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item :label="t('config.form.model')" required>
                  <a-input v-model:value="formState.model" :placeholder="t('config.form.model_placeholder')" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item :label="t('config.form.wire_api')">
                  <a-tooltip :title="t('config.wire_api.tooltip')">
                    <a-select v-model:value="formState.wireApi" @change="onWireApiSelectChange">
                      <a-select-option value="anthropic">
                        <span class="opt-row"><span class="dot purple" /> {{ t("config.wire.anthropic") }}</span>
                      </a-select-option>
                      <a-select-option value="chat_completions">
                        <span class="opt-row"><span class="dot blue" /> {{ t("config.wire.chat") }}</span>
                      </a-select-option>
                      <a-select-option value="openai">
                        <span class="opt-row"><span class="dot cyan" /> {{ t("config.wire.openai") }}</span>
                      </a-select-option>
                    </a-select>
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>

            <a-form-item :label="t('config.form.base_url')" required>
              <a-input v-model:value="formState.baseUrl" :placeholder="t('config.form.base_url_placeholder')" />
            </a-form-item>

            <a-row :gutter="16">
              <a-col :span="12">
                <a-form-item :label="t('config.form.api_key')">
                  <a-input-password v-model:value="formState.apiKey" :placeholder="t('config.form.api_key_placeholder')" />
                </a-form-item>
              </a-col>
              <a-col :span="12">
                <a-form-item :label="t('config.form.api_key_header')">
                  <a-tooltip :title="t('config.form.api_key_header_tooltip')">
                    <a-input v-model:value="formState.apiKeyHeader" :placeholder="t('config.form.api_key_header_placeholder')" />
                  </a-tooltip>
                </a-form-item>
              </a-col>
            </a-row>
          </a-form>
        </div>

        <!-- ============== Model Limits (per-provider) ============== -->
        <div v-if="activeId" class="glass panel">
          <div class="panel-head">
            <div>
              <div class="panel-title">{{ t("config.limits.title") }}</div>
              <div class="panel-sub muted-3">{{ t("config.limits.desc") }}</div>
            </div>
          </div>

          <!-- Context Window Slider - Draggable -->
          <div class="slider-block">
            <div class="slider-head">
              <div>
                <div class="slider-label">{{ t("config.limits.context") }}</div>
                <div class="slider-value mono">{{ contextLabel(ctxValue) }}<span class="muted-3"> {{ t("config.limits.tokens") }}</span></div>
              </div>
              <a-tag class="active-tag">{{ contextLabel(ctxValue) }}</a-tag>
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
                :title="t('config.limits.preset_' + p.key) + ' - ' + p.label"
              />
              <div class="slider-fill" :style="{ width: ctxPercent + '%' }" />
              <div class="slider-thumb" :style="{ left: ctxPercent + '%' }">
                <span class="thumb-tip">{{ contextLabel(ctxValue) }}</span>
              </div>
            </div>
            <div class="slider-stops">
              <button
                v-for="p in LIMIT_PRESETS"
                :key="`ctx-${p.key}`"
                type="button"
                class="slider-stop"
                :class="{ active: ctxValue === p.context }"
                @click="applyLimitPreset(p.key)"
              >
                <span class="stop-dot" :style="{ background: p.color }" />
                <span class="stop-name">{{ t("config.limits.preset_" + p.key) }}</span>
                <span class="stop-label mono">{{ p.label }}</span>
              </button>
            </div>
          </div>

          <!-- Auto Compact Limit Slider - Percentage based, follows Context window -->
          <div class="slider-block">
            <div class="slider-head">
              <div>
                <div class="slider-label">{{ t("config.limits.compact") }}</div>
                <div class="slider-value mono">{{ compactRatio }}%<span class="muted-3"> {{ t("config.limits.of") }} {{ contextLabel(ctxValue) }}</span></div>
              </div>
              <a-tag class="active-tag compact-tag">鈮?{{ contextLabel(compactTokens) }} {{ t("config.limits.tokens") }}</a-tag>
            </div>
            <div
              class="slider-rail"
              ref="ratioRail"
              @mousedown="(e) => startDrag(e, 'ratio')"
              @touchstart.passive="(e) => startDrag(e, 'ratio')"
            >
              <span
                v-for="(p, i) in LIMIT_PRESETS"
                :key="`ratio-tick-${p.key}`"
                class="slider-tick"
                :style="{ left: tickLeft(i) + '%' }"
              />
              <div class="slider-fill compact" :style="{ width: compactRatio + '%' }" />
              <div class="slider-thumb" :style="{ left: compactRatio + '%' }">
                <span class="thumb-tip">{{ compactRatio }}%</span>
              </div>
            </div>
          </div>
        </div>
      </a-tab-pane>
    </a-tabs>

    <!-- Bottom actions bar -->
    <div class="actions fade-up">
      <a-button type="primary" size="large" shape="round"
        :loading="saving"
        :disabled="!activeId"
        @click="handleSave"
      >
        <template #icon><SaveOutlined /></template>
        {{ t("config.save") }}
      </a-button>
      <a-button size="large" shape="round"
        @click="handleSyncToCodex"
      >
        <template #icon><SyncOutlined /></template>
        {{ t("config.sync") }}
      </a-button>
      <a-button size="large" shape="round"
        :disabled="!activeId || !formState.baseUrl"
        :loading="testingConn"
        @click="testConnection"
        class="ghost"
      >
        <template #icon><ApiOutlined /></template>
        {{ t("config.form.test") }}
      </a-button>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted } from "vue"
import { useLocale } from "../composables/useLocale"
import { useRouter } from "vue-router"

import {
  readConfig, writeConfig,
  testProviderConnectivity,
  openConfigDir as openConfigDirApi,
  syncToCodex,
} from "../api/bridge"
import {
  LeftOutlined,
  PlusOutlined,
  SaveOutlined,
  SyncOutlined,
  ReloadOutlined,
  CloseCircleFilled,
  FolderOpenOutlined,
  ApiOutlined,
} from "@ant-design/icons-vue"
import { message } from "ant-design-vue"
import { enable, isEnabled, disable } from "@tauri-apps/plugin-autostart"

const { t } = useLocale()
const router = useRouter()

interface Provider {
  providerId: string
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
  modelContextWindow: number
  modelAutoCompactLimit: number
}

const PRESETS: Array<{
  key: string
  name: string
  values: { wireApi: string; baseUrl: string; apiKeyHeader: string }
}> = [
  { key: "anthropic",  name: "Anthropic",  values: { wireApi: "anthropic", baseUrl: "", apiKeyHeader: "X-Api-Key" } },
  { key: "openai",     name: "OpenAI",     values: { wireApi: "openai", baseUrl: "", apiKeyHeader: "Authorization" } },
  { key: "chat",       name: "Chat",       values: { wireApi: "chat_completions", baseUrl: "", apiKeyHeader: "Authorization" } },
]

const wireOptions = PRESETS.map((p) => ({
  value: p.key,
  label: p.name,
}))

const activeKey = ref("general")

// Provider management state
const providerIds = ref<string[]>([])
const providers = reactive<Record<string, Provider>>({})
const activeId = ref("")
const newProviderName = ref("")
const testingConn = ref(false)
const connResult = ref<null | { ok: boolean; status: number; latency_ms: number; message: string }>(null)

// Each provider stores its own limits. The form state mirrors the active one.
const formState = reactive<Provider>({
  providerId: "",
  wireApi: "anthropic",
  baseUrl: "",
  model: "",
  apiKey: "",
  apiKeyHeader: "X-Api-Key",
  modelContextWindow: DEFAULT_CONTEXT_WINDOW,
  modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
})

const activePresetKey = ref("anthropic")
const saving = ref(false)

// Autostart
const autostartEnabled = ref(false)
const autostartLoading = ref(false)
const openingDir = ref(false)
const configDirHint = ref("")

const LIMIT_PRESETS = [
  { key: "128k",  name: "Small",    label: "128K",  context: 128_000,  compact: 112_000,  color: "#34d399" },
  { key: "256k",  name: "Standard", label: "256K",  context: 256_000,  compact: 220_000,  color: "#60a5fa" },
  { key: "512k",  name: "Large",    label: "512K",  context: 512_000,  compact: 460_000,  color: "#a78bfa" },
  { key: "1m",    name: "Huge",     label: "1M",    context: 1_000_000, compact: 900_000, color: "#f472b6" },
]

const CONTEXT_MIN = 16_000
const CONTEXT_MAX = 2_000_000
const DEFAULT_CONTEXT_WINDOW = 256_000
const DEFAULT_COMPACT_LIMIT = 220_000

// Computed for the active provider's limits
const ctxValue = computed(() => formState.modelContextWindow || DEFAULT_CONTEXT_WINDOW)
const compactRatio = computed(() => {
  if (!formState.modelContextWindow) return 0
  return Math.round((formState.modelAutoCompactLimit / formState.modelContextWindow) * 1000) / 10
})
const compactTokens = computed(() => Math.round(ctxValue.value * compactRatio.value / 100))
const ctxPercent = computed(() => {
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, ctxValue.value))
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
  formState.modelContextWindow = p.context
  formState.modelAutoCompactLimit = p.compact
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
  const next = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, Math.round(value)))
  formState.modelContextWindow = next
  formState.modelAutoCompactLimit = Math.round(next * 0.8)
}

function setCompactRatio(percent: number) {
  const p = Math.min(100, Math.max(0, percent))
  formState.modelAutoCompactLimit = Math.round(formState.modelContextWindow * p / 100)
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

// Provider management functions
function snapshotActive() {
  if (!activeId.value) return
  providers[activeId.value] = { ...formState, providerId: activeId.value }
}

function loadActiveIntoForm() {
  const id = activeId.value
  if (!id || !providers[id]) {
    Object.assign(formState, {
      providerId: "",
      wireApi: "anthropic",
      baseUrl: "",
      model: "",
      apiKey: "",
      apiKeyHeader: "X-Api-Key",
      modelContextWindow: DEFAULT_CONTEXT_WINDOW,
      modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
    })
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
      modelContextWindow: DEFAULT_CONTEXT_WINDOW,
      modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
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
  formState.modelContextWindow = DEFAULT_CONTEXT_WINDOW
  formState.modelAutoCompactLimit = DEFAULT_COMPACT_LIMIT
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
          modelContextWindow: DEFAULT_CONTEXT_WINDOW,
          modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
        }
      }
    } else if (trimmed.startsWith("[")) {
      inProviderSection = false
    } else if (inProviderSection) {
      const p = providers[currentProvider]
      if (!p) continue
      if (trimmed.startsWith("wire_api = ")) p.wireApi = trimmed.replace("wire_api = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("base_url = ")) p.baseUrl = trimmed.replace("base_url = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("model = ")) p.model = trimmed.replace("model = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("api_key = ")) p.apiKey = trimmed.replace("api_key = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("api_key_header = ")) p.apiKeyHeader = trimmed.replace("api_key_header = ", "").replace(/"/g, "")
      else if (trimmed.startsWith("model_context_window = ")) {
        const v = parseInt(trimmed.replace("model_context_window = ", ""))
        if (!isNaN(v)) p.modelContextWindow = v
      }
      else if (trimmed.startsWith("model_auto_compact_token_limit = ")) {
        const v = parseInt(trimmed.replace("model_auto_compact_token_limit = ", ""))
        if (!isNaN(v)) p.modelAutoCompactLimit = v
      }
    }
  }

  activeId.value = activeFromTop && providers[activeFromTop] ? activeFromTop : (providerIds.value[0] || "")
  loadActiveIntoForm()
}

function buildConfig(): string {
  snapshotActive()
  const active = activeId.value

  const blocks: string[] = [
    "# evocode bridge config",
    "# Read by evocode-cli, not by upstream Codex directly.",
    "",
    "provider = \"" + active + "\"",
    "",
  ]

  for (const id of providerIds.value) {
    const p = providers[id] || {
      providerId: id,
      wireApi: "anthropic",
      baseUrl: "",
      model: "",
      apiKey: "",
      apiKeyHeader: "X-Api-Key",
      modelContextWindow: DEFAULT_CONTEXT_WINDOW,
      modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
    }
    blocks.push(
      "[providers." + id + "]",
      "wire_api = \"" + p.wireApi + "\"",
      "base_url = \"" + p.baseUrl + "\"",
      "model = \"" + p.model + "\"",
      "api_key = \"" + p.apiKey + "\"",
      "api_key_header = \"" + p.apiKeyHeader + "\"",
      "model_context_window = " + (p.modelContextWindow || DEFAULT_CONTEXT_WINDOW),
      "model_auto_compact_token_limit = " + (p.modelAutoCompactLimit || DEFAULT_COMPACT_LIMIT),
      "",
    )
  }
  return blocks.join("\n").replace(/\n+$/, "\n")
}

async function handleSave() {
  if (!activeId.value) {
    message.warning("Add a provider first.")
    return
  }
  snapshotActive()
  const cur = providers[activeId.value]
  if (!cur?.model || !cur?.baseUrl) {
    message.warning("Fill in Model and Base URL for \"" + activeId.value + "\".")
    return
  }
  saving.value = true
  try {
    await writeConfig(buildConfig())
    message.success("Config saved! Restart the bridge to apply changes.", 4)
  } catch (e: any) {
    message.error("Failed to save: " + (e?.message || String(e)), 4)
  } finally {
    saving.value = false
  }
}

// Sync reads from ~/.evocode/config.toml directly, so make sure to
// save before calling this.
async function handleSyncToCodex() {
  if (!activeId.value) {
    message.warning("Add a provider first.")
    return
  }
  try {
    await syncToCodex()
    message.success("Synced to Codex!", 4)
  } catch (e: any) {
    message.error("Sync failed: " + (e?.message || String(e)), 4)
  }
}

function buildConfigDirHint() {
  // Show the conventional path; the renderer cannot resolve USERPROFILE.
  const sep = navigator?.platform?.toLowerCase().includes("win") ? "\\" : "/"
  configDirHint.value = "~" + sep + ".evocode" + sep + "config.toml"
}

async function loadAutostartStatus() {
  try {
    autostartEnabled.value = await isEnabled()
  } catch {
    autostartEnabled.value = false
  }
}

async function onAutostartChange(checked: boolean) {
  autostartLoading.value = true
  try {
    if (checked) {
      await enable()
      message.success(t("config.autostart.enabled"), 3)
    } else {
      await disable()
      message.success(t("config.autostart.disabled"), 3)
    }
    autostartEnabled.value = checked
  } catch (e: any) {
    message.error(t("config.autostart.error") + ": " + (e?.message || String(e)), 4)
  } finally {
    autostartLoading.value = false
  }
}

function onWireApiChange(key: string | number) {
  const preset = PRESETS.find((p) => p.key === key)
  if (!preset) return
  formState.wireApi = preset.values.wireApi
  formState.apiKeyHeader = preset.values.apiKeyHeader
  // Keep current baseUrl if user already typed one
  snapshotActive()
}

function onWireApiSelectChange(value: string) {
  formState.wireApi = value
  const matchingPreset = PRESETS.find((p) => p.values.wireApi === value)
  if (matchingPreset) {
    activePresetKey.value = matchingPreset.key
    formState.apiKeyHeader = matchingPreset.values.apiKeyHeader
  }
}

async function openConfigDir() {
  openingDir.value = true
  try {
    const path = await openConfigDirApi()
    message.success(t("config.configdir.opened") + ": " + path, 3)
  } catch (e: any) {
    message.error(t("config.configdir.error") + ": " + (e?.message || String(e)), 4)
  } finally {
    openingDir.value = false
  }
}

async function testConnection() {
  if (!formState.baseUrl) {
    message.warning(t("config.form.test_need_url"))
    return
  }
  testingConn.value = true
  connResult.value = null
  try {
    const r = await testProviderConnectivity(
      formState.baseUrl,
      formState.apiKey || "",
      formState.wireApi || "anthropic",
      formState.apiKeyHeader || undefined,
    )
    connResult.value = r
  } catch (e: any) {
    connResult.value = {
      ok: false,
      status: 0,
      latency_ms: 0,
      message: t("config.form.test_error") + ": " + (e?.message || String(e)),
    }
  } finally {
    testingConn.value = false
  }
}

onMounted(async () => {
  try {
    const text = await readConfig()
    parseConfig(text)
  } catch {}
  buildConfigDirHint()
  await loadAutostartStatus()
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

.head-actions { display: inline-flex; gap: 8px; align-items: center; }

.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  gap: 16px;
  padding: 14px 4px;
}
.setting-meta { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
.setting-name { font-size: 14px; font-weight: 600; color: var(--text-1); }
.setting-desc { font-size: 12.5px; color: var(--text-3); }
.row-divider { margin: 4px 0; border-color: var(--border); }

.conn-alert { margin-bottom: 16px; }

@media (max-width: 720px) {
  .actions { justify-content: stretch; }
  .actions .ant-btn { flex: 1; }
  .slider-stops { grid-template-columns: repeat(2, 1fr); }
}
</style>
