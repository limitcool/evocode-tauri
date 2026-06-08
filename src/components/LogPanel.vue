<template>
  <div class="log-card glass fade-up">
    <div class="log-head">
      <div class="title">
        <span class="bar" />
        <span>{{ t("logs.title") }}</span>
        <a-tag :color="statusColor" class="status-tag">{{ statusLabel }}</a-tag>
      </div>
      <div class="actions">
        <a-tooltip :title="t('logs.refresh')">
          <a-button type="text" size="small" class="icon-btn" @click="pollStatus">
            <ReloadOutlined />
          </a-button>
        </a-tooltip>
        <a-tooltip :title="t('logs.clear')">
          <a-button type="text" size="small" class="icon-btn" @click="logLines = []">
            <ClearOutlined />
          </a-button>
        </a-tooltip>
        <a-tooltip :title="t('logs.auto_scroll')">
          <a-switch v-model:checked="autoScroll" size="small" />
        </a-tooltip>
      </div>
    </div>

    <div class="log-body" ref="bodyRef">
      <div v-if="logLines.length === 0" class="empty">
        <DatabaseOutlined class="empty-icon" />
        <div class="empty-title">{{ t("logs.empty_title") }}</div>
        <div class="empty-sub">{{ t("logs.empty_sub", { status: bridgeRunning ? t("logs.running") : t("logs.stopped") }) }}</div>
      </div>
      <div
        v-for="(line, i) in logLines"
        :key="i"
        class="log-line"
        :class="lineClass(line)"
      >
        <span class="line-no">{{ String(i + 1).padStart(3, "0") }}</span>
        <span class="line-content">{{ line }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue"
import { useLocale } from "../composables/useLocale"
import {
  ReloadOutlined,
  ClearOutlined,
  DatabaseOutlined,
} from "@ant-design/icons-vue"
import { getBridgeStatus, getBridgeLogs } from "../api/bridge"

const { t } = useLocale()

const props = defineProps<{
  bridgeRunning: boolean
}>()

type Status = "starting" | "running" | "stopped" | "error"
const status = ref<Status>(props.bridgeRunning ? "starting" : "stopped")
const logLines = ref<string[]>([])
const autoScroll = ref(true)
const bodyRef = ref<HTMLElement | null>(null)

const statusLabel = computed(() => {
  switch (status.value) {
    case "starting": return t("logs.label_starting")
    case "running": return t("logs.label_running")
    case "error": return t("logs.label_error")
    default: return t("logs.label_stopped")
  }
})

const statusColor = computed(() => {
  switch (status.value) {
    case "running": return "success"
    case "starting": return "processing"
    case "error": return "error"
    default: return "default"
  }
})

function lineClass(line: string) {
  const l = line.toLowerCase()
  if (l.includes("error") || l.includes("fail")) return "err"
  if (l.includes("warn")) return "warn"
  if (l.includes("request") || l.includes("POST") || l.includes("GET")) return "info"
  return ""
}

let statusInterval: ReturnType<typeof setInterval> | null = null

async function pollStatus() {
  try {
    const s = await getBridgeStatus()
    status.value = s === "running" ? "running" : "stopped"
    if (s === "running") {
      logLines.value = await getBridgeLogs()
      if (autoScroll.value) {
        await nextTick()
        const el = bodyRef.value
        if (el) el.scrollTop = el.scrollHeight
      }
    }
  } catch {
    status.value = "error"
  }
}

function startPolling() {
  status.value = "starting"
  logLines.value = []
  pollStatus()
  statusInterval = setInterval(pollStatus, 1000)
}

function stopPolling() {
  if (statusInterval) { clearInterval(statusInterval); statusInterval = null }
  status.value = "stopped"
  logLines.value = []
}

watch(() => props.bridgeRunning, (running) => {
  if (running) startPolling()
  else stopPolling()
})

onMounted(() => {
  if (props.bridgeRunning) startPolling()
})
onUnmounted(stopPolling)
</script>

<style scoped>
.log-card { padding: 0; overflow: hidden; }

.log-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: linear-gradient(180deg, rgba(255,255,255,0.02), transparent);
}
.title { display: inline-flex; align-items: center; gap: 10px; color: var(--text-1); font-weight: 600; }
.title .bar {
  width: 3px; height: 14px; border-radius: 2px;
  background: linear-gradient(180deg, var(--brand-400), var(--brand-700));
}
.status-tag { margin-left: 4px; }

.actions { display: inline-flex; align-items: center; gap: 8px; }
.icon-btn { color: var(--text-3); width: 30px; height: 30px; border-radius: 8px; }
.icon-btn:hover { color: var(--text-1); background: var(--bg-elev-3); }

.log-body {
  height: 360px;
  overflow-y: auto;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.7;
  background: linear-gradient(180deg, var(--bg-elev-1), var(--bg-elev-2));
  padding: 6px 0;
}

.empty {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 100%; color: var(--text-3); gap: 6px;
}
.empty-icon { font-size: 28px; color: var(--text-4); }
.empty-title { font-size: 14px; color: var(--text-2); }
.empty-sub { font-size: 12px; color: var(--text-4); }

.log-line {
  display: flex; gap: 12px; padding: 1px 14px;
  color: var(--text-2); white-space: pre-wrap; word-break: break-all;
  transition: background .12s ease;
}
.log-line:hover { background: rgba(255,255,255,0.03); }
.line-no { color: var(--text-4); user-select: none; min-width: 36px; }
.log-line.info { color: #cfe1ff; }
.log-line.warn { color: #fde68a; background: rgba(251,191,36,0.04); }
.log-line.err  { color: #fecaca; background: rgba(248,113,113,0.06); }
</style>
