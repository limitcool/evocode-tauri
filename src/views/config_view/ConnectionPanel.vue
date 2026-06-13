<template>
  <div class="connection-panel">
    <div class="glass panel">
      <div class="panel-head">
        <div style="display: inline-flex; align-items: center; gap: 10px;">
          <div class="panel-title">{{ t("config.providers") }}</div>
          <a-button size="small" type="dashed" @click="showAddModal = true">
            <template #icon><PlusOutlined /></template>
            {{ t("config.providers.add") }}
          </a-button>
        </div>
        <div class="active-with-sync">
          <a-tag v-if="activeId" class="active-tag">{{ t("config.providers.active") }} {{ activeId }}</a-tag>
          <a-select
            v-if="providerIds.length"
            v-model:value="activeId"
            style="width: 240px;"
            size="small"
            :loading="syncing"
            @change="handleSyncToCodex">
            <a-select-option v-for="id in providerIds" :key="id" :value="id">
              {{ id }}
            </a-select-option>
          </a-select>
          <a-tag v-else style="font-weight: 400;">{{ t("config.sync.no_providers") }}</a-tag>
        </div>
      </div>

      <a-tabs
        v-if="providerIds.length"
        type="editable-card"
        hideAdd
        :activeKey="editingId"
        @change="onTabChange"
        @edit="onTabEdit"
        class="prov-tabs"
        size="small"
        destroyInactiveTabPane
      >
        <a-tab-pane
          v-for="(item, key) in providers"
          :key="key"
          :tab="key"
        >
          <div class="tab-body">
            <!-- Provider config form -->
            <div class="tab-section">
              <div class="tab-section-head">
                <div class="tab-section-title">{{ t("config.form.title") }}</div>
                <a-button size="small" @click="resetForm(key)">
                  <template #icon><ReloadOutlined /></template> Reset
                </a-button>
              </div>
              <a-alert v-if="connResult" class="conn-alert"
                :type="connResult.ok ? 'success' : 'error'"
                :message="connResult.message"
                :description="t('config.form.test_latency') + ': ' + connResult.latency_ms + ' ms'"
                show-icon closable @close="connResult = null" />
              <a-form layout="vertical" class="form" :model="item">
                <a-row :gutter="16">
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.model')" required>
                      <a-input v-model:value="item.model" :placeholder="t('config.form.model_placeholder')" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.wire_api')">
                      <a-tooltip :title="t('config.wire_api.tooltip')">
                        <a-select v-model:value="item.wireApi" @change="(val: string) => onWireApiSelectChange(key, val)">
                          <a-select-option value="anthropic"><span class="opt-row"><span class="dot purple" /> {{ t("config.wire.anthropic") }}</span></a-select-option>
                          <a-select-option value="chat_completions"><span class="opt-row"><span class="dot blue" /> {{ t("config.wire.chat") }}</span></a-select-option>
                          <a-select-option value="openai"><span class="opt-row"><span class="dot cyan" /> {{ t("config.wire.openai") }}</span></a-select-option>
                        </a-select>
                      </a-tooltip>
                    </a-form-item>
                  </a-col>
                </a-row>
                <a-form-item :label="t('config.form.base_url')" required>
                  <a-input v-model:value="item.baseUrl" :placeholder="t('config.form.base_url_placeholder')" />
                </a-form-item>
                <a-row :gutter="16">
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.api_key')">
                      <a-input-password v-model:value="item.apiKey" :placeholder="t('config.form.api_key_placeholder')" />
                    </a-form-item>
                  </a-col>
                  <a-col :span="12">
                    <a-form-item :label="t('config.form.api_key_header')">
                      <a-tooltip :title="t('config.form.api_key_header_tooltip')">
                        <a-input v-model:value="item.apiKeyHeader" :placeholder="t('config.form.api_key_header_placeholder')" />
                      </a-tooltip>
                    </a-form-item>
                  </a-col>
                </a-row>
              </a-form>
            </div>

            <!-- Limits: context window + auto compact -->
            <div class="tab-section">
              <div class="tab-section-head">
                <div class="tab-section-title">{{ t("config.limits.title") }}</div>
                <div class="tab-section-sub muted-3">{{ t("config.limits.desc") }}</div>
                </div>

              <div class="slider-block">
                <div class="slider-head">
                  <div>
                    <div class="slider-label">{{ t("config.limits.context_window") }}</div>
                    <div class="slider-value">{{ contextLabel(item.modelContextWindow) }}</div>
                  </div>
                </div>
                <div :ref="(el: any) => ctxRailRef(key, el)" class="slider-rail"
                  @mousedown="(e: any) => startDrag(e, 'context', key)"
                  @touchstart.prevent="(e: any) => startDrag(e, 'context', key)">
                  <div class="slider-fill" :style="{ width: ctxPercent(key) + '%' }" />
                  <div class="slider-thumb" :style="{ left: ctxPercent(key) + '%' }"><span class="thumb-tip">{{ contextLabel(item.modelContextWindow) }}</span></div>
                  <div v-for="(_, i) in LIMIT_PRESETS" :key="i" class="slider-tick" :style="{ left: tickLeft(i) + '%' }" />
                </div>
                <div class="slider-stops">
                  <div v-for="p in LIMIT_PRESETS" :key="p.key" class="slider-stop"
                    :class="{ active: item.modelContextWindow === p.context }"
                    @click="applyLimitPreset(key, p.key)">
                    <div class="stop-dot" :style="{ color: p.key === 'compact' ? '#34d399' : '#4d7dff' }" />
                    <div class="stop-name">{{ t(p.labelKey) }}</div>
                    <div class="stop-label">{{ contextLabel(p.context) }}</div>
                  </div>
                </div>
              </div>

              <a-divider style="margin: 8px 0;" />

              <div class="slider-block">
                <div class="slider-head">
                  <div>
                    <div class="slider-label">
                      {{ t("config.limits.auto_compact") }}
                      <a-tooltip :title="t('config.limits.auto_compact_tip')"><InfoCircleOutlined class="compact-tag" /></a-tooltip>
                    </div>
                    <div class="slider-value">{{ contextLabel(autoCompactTokens(key)) }}</div>
                  </div>
                  <div class="slider-label" style="margin-bottom: 2px;">{{ autoCompactRatio(key) }}%</div>
                </div>
                <div :ref="(el: any) => ratioRailRef(key, el)" class="slider-rail"
                  @mousedown="(e: any) => startDrag(e, 'ratio', key)"
                  @touchstart.prevent="(e: any) => startDrag(e, 'ratio', key)">
                  <div class="slider-fill compact" :style="{ width: autoCompactRatio(key) + '%' }" />
                  <div class="slider-thumb" :style="{ left: autoCompactRatio(key) + '%' }">
                    <span class="thumb-tip">{{ autoCompactRatio(key) }}%</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </a-tab-pane>
      </a-tabs>
      <div v-else class="empty-tabs-hint">{{ t("config.sync.no_providers") }}</div>

    </div>
    <div class="actions-bar">
      <a-button :loading="testingConn" :disabled="testingConn" @click="testConnection(editingId)">
        <template #icon><ApiOutlined /></template>
        {{ t("config.form.test") }}
      </a-button>
      <a-button type="primary" :loading="saving" :disabled="saving" @click="handleSave">
        <template #icon><SaveOutlined /></template>
        {{ t("config.save") }}
      </a-button>
    </div>

    <a-modal
      v-model:open="showAddModal"
      :title="t('config.providers.add')"
      :ok-text="t('config.providers.ok')"
      :cancel-text="t('config.providers.cancel')"
      @ok="doAddProvider"
    >
      <a-input
        v-model:value="newProviderName"
        :placeholder="t('config.providers.placeholder')"
        @press-enter="doAddProvider"
      />
    </a-modal>

    <a-modal
      v-model:open="showRemoveModal"
      :title="t('config.providers.remove_title')"
      :ok-text="t('config.providers.ok')"
      :cancel-text="t('config.providers.cancel')"
      @ok="doRemoveProvider"
    >
      <p>{{ removeTarget ? t("config.providers.remove_confirm", { name: removeTarget }) : "" }}</p>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from "vue"
import { useLocale } from "../../composables/useLocale"
import { writeConfig, syncToCodex, readConfig, testProviderConnectivity } from "../../api/bridge"
import { message } from "ant-design-vue"
import { PlusOutlined, ReloadOutlined, ApiOutlined, InfoCircleOutlined, SaveOutlined } from "@ant-design/icons-vue"

const { t } = useLocale()

interface Provider {
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
  modelContextWindow: number
  modelAutoCompactLimit: number
}

const CONTEXT_MIN = 16_000
const CONTEXT_MAX = 10_000_000
const DEFAULT_CONTEXT_WINDOW = 128_000
const DEFAULT_COMPACT_LIMIT = 100_000

const LIMIT_PRESETS = [
  { key: "256k", context: 128_000, compact: 100_000, labelKey: "config.limits.preset_256k" as const },
  { key: "512k", context: 256_000, compact: 220_000, labelKey: "config.limits.preset_512k" as const },
  { key: "1m", context: 1_000_000, compact: 800_000, labelKey: "config.limits.preset_1m" as const },
  { key: "10m", context: 10_000_000, compact: 8_000_000, labelKey: "config.limits.preset_10m" as const },
]

const PRESETS = [
  { key: "anthropic", values: { wireApi: "anthropic" as const, apiKeyHeader: "X-Api-Key" as const } },
  { key: "chat_completions", values: { wireApi: "chat_completions" as const, apiKeyHeader: "Authorization" as const } },
  { key: "openai", values: { wireApi: "openai" as const, apiKeyHeader: "Authorization" as const } },
]


const providerIds = ref<string[]>([])
const providers = reactive<Record<string, Provider>>({})
const activeId = ref("")
const editingId = ref("")
const newProviderName = ref("")
const showAddModal = ref(false)
const showRemoveModal = ref(false)
const removeTarget = ref("")
const wirePresetKey = reactive<Record<string, string>>({})
const testingConn = ref(false)
const saving = ref(false)
const syncing = ref(false)
const connResult = ref<null | { ok: boolean; status: number; latency_ms: number; message: string }>(null)

// Per-provider slider rail refs
const ctxRails = reactive<Record<string, HTMLElement | null>>({})
const ratioRails = reactive<Record<string, HTMLElement | null>>({})

function ctxRailRef(id: string, el: any) { ctxRails[id] = el }
function ratioRailRef(id: string, el: any) { ratioRails[id] = el }

// Drag state
let dragging: { id: string; type: "context" | "ratio" } | null = null
let onMove: ((e: MouseEvent | TouchEvent) => void) | null = null
let onUp: (() => void) | null = null

function tickLeft(i: number) {
  const p = LIMIT_PRESETS[i]
  return Math.round(((p.context - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
}

function contextLabel(n: number) {
  if (!n) return "0"
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(n % 1_000_000 === 0 ? 0 : 1).replace(/\.0$/, "") + "M"
  if (n >= 1_000) return Math.round(n / 1_000) + "K"
  return String(n)
}

function railPercent(rail: HTMLElement, clientX: number): number {
  const rect = rail.getBoundingClientRect()
  return Math.min(100, Math.max(0, ((clientX - rect.left) / rect.width) * 100))
}

function ctxPercent(id: string) {
  const p = providers[id]
  if (!p) return 0
  const v = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, p.modelContextWindow))
  return Math.round(((v - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100)
}

function autoCompactRatio(id: string) {
  const p = providers[id]
  if (!p || !p.modelContextWindow) return 0
  return Math.round((p.modelAutoCompactLimit / p.modelContextWindow) * 1000) / 10
}

function autoCompactTokens(id: string) {
  const p = providers[id]
  if (!p) return 0
  return Math.round(p.modelContextWindow * autoCompactRatio(id) / 100)
}

function applyDrag(id: string, type: "context" | "ratio", clientX: number) {
  const p = providers[id]
  if (!p) return
  if (type === "context") {
    const rail = ctxRails[id]; if (!rail) return
    const pct = railPercent(rail, clientX)
    let snapped: number | null = null
    for (const preset of LIMIT_PRESETS) {
      if (Math.abs(((preset.context - CONTEXT_MIN) / (CONTEXT_MAX - CONTEXT_MIN)) * 100 - pct) < 6) { snapped = preset.context; break }
    }
    if (snapped == null) setContextWindow(id, CONTEXT_MIN + (CONTEXT_MAX - CONTEXT_MIN) * (pct / 100))
    else setContextWindow(id, snapped)
  } else {
    const rail = ratioRails[id]; if (!rail) return
    setCompactRatio(id, railPercent(rail, clientX))
  }
}

function setContextWindow(id: string, value: number) {
  const p = providers[id]
  if (!p) return
  p.modelContextWindow = Math.min(CONTEXT_MAX, Math.max(CONTEXT_MIN, Math.round(value)))
  p.modelAutoCompactLimit = Math.round(p.modelContextWindow * 0.8)
}

function setCompactRatio(id: string, percent: number) {
  const p = providers[id]
  if (!p) return
  p.modelAutoCompactLimit = Math.round(p.modelContextWindow * Math.min(100, Math.max(0, percent)) / 100)
}

function startDrag(e: MouseEvent | TouchEvent, type: "context" | "ratio", id: string) {
  const clientX = "touches" in e ? e.touches[0].clientX : (e as MouseEvent).clientX
  applyDrag(id, type, clientX)
  dragging = { id, type }
  onMove = (ev: MouseEvent | TouchEvent) => {
    if (!dragging) return
    applyDrag(dragging.id, dragging.type, "touches" in ev ? ev.touches[0].clientX : (ev as MouseEvent).clientX)
  }
  onUp = () => {
    dragging = null
    if (onMove) { window.removeEventListener("mousemove", onMove as any); window.removeEventListener("touchmove", onMove as any) }
    if (onUp) { window.removeEventListener("mouseup", onUp); window.removeEventListener("touchend", onUp) }
    onMove = null; onUp = null
  }
  window.addEventListener("mousemove", onMove as any)
  window.addEventListener("touchmove", onMove as any, { passive: true } as any)
  window.addEventListener("mouseup", onUp)
  window.addEventListener("touchend", onUp)
}

onUnmounted(() => {
  dragging = null
  if (onMove) { window.removeEventListener("mousemove", onMove as any); window.removeEventListener("touchmove", onMove as any) }
  if (onUp) { window.removeEventListener("mouseup", onUp); window.removeEventListener("touchend", onUp) }
})

function applyLimitPreset(id: string, key: string) {
  const p = providers[id]
  if (!p) return
  const preset = LIMIT_PRESETS.find((x) => x.key === key)
  if (!preset) return
  p.modelContextWindow = preset.context
  p.modelAutoCompactLimit = preset.compact
}

function onWireApiSelectChange(id: string, value: string) {
  const p = providers[id]
  if (!p) return
  p.wireApi = value
  const m = PRESETS.find((pr) => pr.values.wireApi === value)
  if (m) {
    wirePresetKey[id] = m.key
    p.apiKeyHeader = m.values.apiKeyHeader
  }
}

async function testConnection(id: string) {
  const p = providers[id]
  if (!p || !p.baseUrl) { message.warning(t("config.form.test_need_url")); return }
  testingConn.value = true; connResult.value = null
  try {
    const r = await testProviderConnectivity(p.baseUrl, p.apiKey || "", p.wireApi || "anthropic", p.apiKeyHeader || undefined, p.model || undefined)
    connResult.value = r
  } catch (e: any) {
    connResult.value = { ok: false, status: 0, latency_ms: 0, message: t("config.form.test_error") + ": " + (e?.message || String(e)) }
  } finally {
    testingConn.value = false
  }
}

function resetForm(id: string) {
  const p = providers[id]
  if (!p) return
  p.model = ""
  p.baseUrl = ""
  p.apiKey = ""
  p.wireApi = "anthropic"
  p.apiKeyHeader = "X-Api-Key"
  p.modelContextWindow = DEFAULT_CONTEXT_WINDOW
  p.modelAutoCompactLimit = DEFAULT_COMPACT_LIMIT
  const m = PRESETS.find((x) => x.values.wireApi === p.wireApi)
  if (m) wirePresetKey[id] = m.key
}

function onTabChange(key: string) {
  editingId.value = key
}

function onTabEdit(targetKey: string | MouseEvent, action: string) {
  if (action === "remove") {
    removeTarget.value = targetKey as string
    showRemoveModal.value = true
  }
}

function doAddProvider() {
  const name = newProviderName.value.trim()
  if (!name) return
  if (!providerIds.value.includes(name)) providerIds.value.push(name)
  if (!providers[name]) {
    providers[name] = {
      wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key",
      modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT,
    }
  }
  if (!wirePresetKey[name]) wirePresetKey[name] = "anthropic"
  newProviderName.value = ""
  showAddModal.value = false
  editingId.value = name
  const m = PRESETS.find((x) => x.values.wireApi === providers[name].wireApi)
  if (m) wirePresetKey[name] = m.key
}

async function doRemoveProvider() {
  const id = removeTarget.value
  if (!id) return
  const idx = providerIds.value.indexOf(id)
  if (idx > -1) providerIds.value.splice(idx, 1)
  delete providers[id]
  delete ctxRails[id]
  delete ratioRails[id]
  delete wirePresetKey[id]
  showRemoveModal.value = false
  removeTarget.value = ""
  if (editingId.value === id) {
    editingId.value = providerIds.value[0] || ""
  }
  try {
    await writeConfig(buildConfig())
  } catch {}
}

// parse / build / sync
function parseConfig(text: string) {
  providerIds.value = []
  for (const k of Object.keys(providers)) delete providers[k]
  for (const k of Object.keys(wirePresetKey)) delete wirePresetKey[k]
  if (!text) { activeId.value = ""; editingId.value = ""; return }

  const lines = text.split("\n")
  let cur = "", inProv = false, activeFromTop = ""

  for (const line of lines) {
    const t = line.trim()
    if (!t || t.startsWith("#")) continue
    if (t.startsWith("provider = ")) { activeFromTop = t.replace("provider = ", "").replace(/"/g, "") }
    else if (t.startsWith("[providers.")) {
      cur = t.replace("[providers.", "").replace("]", ""); inProv = true
      if (!providerIds.value.includes(cur)) providerIds.value.push(cur)
      if (!providers[cur]) providers[cur] = { wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT }
    } else if (t.startsWith("[")) { inProv = false }
    else if (inProv) {
      const p = providers[cur]; if (!p) continue
      if (t.startsWith("wire_api = ")) p.wireApi = t.replace("wire_api = ", "").replace(/"/g, "")
      else if (t.startsWith("base_url = ")) p.baseUrl = t.replace("base_url = ", "").replace(/"/g, "")
      else if (t.startsWith("model = ")) p.model = t.replace("model = ", "").replace(/"/g, "")
      else if (t.startsWith("api_key = ")) p.apiKey = t.replace("api_key = ", "").replace(/"/g, "")
      else if (t.startsWith("api_key_header = ")) p.apiKeyHeader = t.replace("api_key_header = ", "").replace(/"/g, "")
      else if (t.startsWith("model_context_window = ")) { const v = parseInt(t.replace("model_context_window = ", "")); if (!isNaN(v)) p.modelContextWindow = v }
      else if (t.startsWith("model_auto_compact_token_limit = ")) { const v = parseInt(t.replace("model_auto_compact_token_limit = ", "")); if (!isNaN(v)) p.modelAutoCompactLimit = v }
    }
  }

  for (const id of providerIds.value) {
    if (!wirePresetKey[id]) {
      const m = PRESETS.find((pr) => pr.values.wireApi === (providers[id]?.wireApi || "anthropic"))
      wirePresetKey[id] = m ? m.key : "anthropic"
    }
  }

  activeId.value = activeFromTop && providers[activeFromTop] ? activeFromTop : (providerIds.value[0] || "")
  editingId.value = activeId.value
}

function buildConfig(): string {
  const active = activeId.value
  const blocks: string[] = ["# evocode bridge config", "# Read by evocode-cli, not by upstream Codex directly.", "", "provider = \"" + active + "\"", ""]
  for (const id of providerIds.value) {
    const p = providers[id] || { wireApi: "anthropic", baseUrl: "", model: "", apiKey: "", apiKeyHeader: "X-Api-Key", modelContextWindow: DEFAULT_CONTEXT_WINDOW, modelAutoCompactLimit: DEFAULT_COMPACT_LIMIT }
    blocks.push("[providers." + id + "]", "wire_api = \"" + p.wireApi + "\"", "base_url = \"" + p.baseUrl + "\"", "model = \"" + p.model + "\"", "api_key = \"" + p.apiKey + "\"", "api_key_header = \"" + p.apiKeyHeader + "\"", "model_context_window = " + (p.modelContextWindow || DEFAULT_CONTEXT_WINDOW), "model_auto_compact_token_limit = " + (p.modelAutoCompactLimit || DEFAULT_COMPACT_LIMIT), "")
  }
  return blocks.join("\n").replace(/\n+$/, "\n")
}

async function handleSave() {
  saving.value = true
  try {
    await writeConfig(buildConfig())
    message.success("Config saved", 3)
  } catch (e: any) {
    message.error("Failed to save: " + (e?.message || String(e)), 4)
  } finally {
    saving.value = false
  }
}

async function handleSyncToCodex(providerId: string) {
  if (!providerId || syncing.value) return
  syncing.value = true
  try {
    const cfg = buildConfig()
    await writeConfig(cfg)
    await syncToCodex()
    message.success(t("config.sync.done") + " " + providerId, 3)
  } catch (e: any) {
    message.error(t("config.sync.failed") + ": " + (e?.message || String(e)), 4)
  } finally {
    syncing.value = false
  }
}

defineExpose({ activeId, providerIds, buildConfig, parseConfig, setActive: (id: string) => { activeId.value = id } })

onMounted(async () => {
  try {
    parseConfig(await readConfig())
  } catch (e) {
    console.error("Failed to load config in ConnectionPanel", e)
  }
})
</script>

<style scoped>
.connection-panel { display: flex; flex-direction: column; gap: 14px; }
.panel { padding: 20px 22px; display: flex; flex-direction: column; gap: 14px; }
.panel-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.panel-title { font-size: 15px; font-weight: 600; color: var(--text-1); }
.panel-sub { font-size: 12.5px; margin-top: 2px; }
.active-tag { border-radius: 999px; }
.active-with-sync { display: inline-flex; align-items: center; gap: 6px; }
.head-actions { display: inline-flex; gap: 8px; align-items: center; }
.empty-block { padding: 24px 0; }
.conn-alert { margin-bottom: 16px; }
.empty-tabs-hint { padding: 16px 0; color: var(--text-3); font-size: 13px; text-align: center; }
.add-btn-row { display: flex; gap: 8px; margin-top: 8px; }
.prov-tabs { margin-top: 4px; }
.prov-tabs :deep(.ant-tabs-nav) { margin-bottom: 0; }
.prov-tabs :deep(.ant-tabs-tab) { font-size: 13px; padding: 4px 12px; }
.form :deep(.ant-form-item-label > label) { color: var(--text-2); font-size: 12.5px; }
.opt-row { display: inline-flex; align-items: center; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.dot.blue { background: #60a5fa; box-shadow: 0 0 8px #60a5fa; }
.dot.purple { background: #a78bfa; box-shadow: 0 0 8px #a78bfa; }
.dot.cyan { background: #22d3ee; box-shadow: 0 0 8px #22d3ee; }
.slider-block { display: flex; flex-direction: column; gap: 12px; }
.slider-head { display: flex; align-items: flex-end; justify-content: space-between; gap: 12px; }
.slider-label { font-size: 12.5px; color: var(--text-3); margin-bottom: 2px; }
.slider-value { font-size: 22px; font-weight: 700; color: var(--text-1); }
.compact-tag { color: var(--text-2); }
.slider-rail { position: relative; height: 12px; border-radius: 999px; background: linear-gradient(90deg, var(--bg-elev-3), var(--bg-elev-2)); border: 1px solid var(--border); margin: 22px 6px 12px; cursor: pointer; user-select: none; touch-action: none; }
.slider-fill { position: absolute; left: 0; top: 0; bottom: 0; background: linear-gradient(90deg, #4d7dff, #8b5cf6); border-radius: 999px; box-shadow: 0 0 12px rgba(77,125,255,0.45); transition: width .08s linear; }
.slider-fill.compact { background: linear-gradient(90deg, #34d399, #22d3ee); box-shadow: 0 0 10px rgba(52,211,153,0.35); }
.slider-thumb { position: absolute; top: 50%; width: 22px; height: 22px; border-radius: 50%; background: white; border: 3px solid #4d7dff; transform: translate(-50%, -50%); box-shadow: 0 0 14px rgba(77,125,255,0.55); cursor: grab; transition: left .08s linear; z-index: 2; }
.slider-thumb:hover { box-shadow: 0 6px 18px rgba(77,125,255,0.65); }
.slider-thumb:active { cursor: grabbing; transform: translate(-50%, -50%) scale(1.08); }
.thumb-tip { position: absolute; bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); padding: 2px 8px; border-radius: 6px; background: var(--bg-elev-3); border: 1px solid var(--border-strong); color: var(--text-1); font-size: 11px; white-space: nowrap; font-weight: 600; pointer-events: none; opacity: 0; transition: opacity .15s ease; }
.slider-thumb:hover .thumb-tip, .slider-thumb:active .thumb-tip { opacity: 1; }
.slider-tick { position: absolute; top: -4px; width: 2px; height: 16px; background: var(--border-strong); transform: translateX(-50%); border-radius: 2px; }
.slider-stops { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin-top: 4px; }
.slider-stop { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 10px 12px; border-radius: var(--r-md); background: var(--bg-elev-2); border: 1px solid var(--border); color: var(--text-2); cursor: pointer; transition: border-color .15s ease, color .15s ease, transform .15s ease; }
.slider-stop:hover { border-color: var(--border-strong); color: var(--text-1); transform: translateY(-1px); }
.slider-stop.active { color: var(--text-1); border-color: rgba(77,125,255,0.5); background: linear-gradient(135deg, rgba(77,125,255,0.12), rgba(139,92,246,0.06)); box-shadow: 0 0 0 1px rgba(77,125,255,0.18); }
.stop-dot { width: 8px; height: 8px; border-radius: 50%; box-shadow: 0 0 6px currentColor; }
.stop-name { font-size: 12px; }
.stop-label { font-size: 13px; font-weight: 700; color: var(--text-1); }
.tab-body { padding: 18px 0 4px; display: flex; flex-direction: column; gap: 18px; }
.tab-section { display: flex; flex-direction: column; gap: 10px; }
.tab-section-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.tab-section-title { font-size: 14px; font-weight: 600; color: var(--text-1); }
.tab-section-sub { font-size: 12px; margin-top: 1px; }
.actions-bar {
  margin-top: 14px; position: sticky; bottom: 16px; z-index: 5;
  display: flex; justify-content: flex-end; gap: 10px;
  padding: 12px 16px; border-radius: var(--r-lg);
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  box-shadow: var(--shadow-md);
}
</style>



