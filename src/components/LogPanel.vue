<template>
  <a-card class="log-panel" :bordered="false">
    <template #title>
      <div class="log-header">
        <span class="log-title">Bridge Logs</span>
        <div class="health-badge" :class="status">
          <LoadingOutlined v-if="status === 'starting'" />
          <CheckCircleOutlined v-else-if="status === 'running'" />
          <CloseCircleOutlined v-else />
          <span>{{ statusLabel }}</span>
        </div>
      </div>
    </template>

    <div class="log-output">
      <div v-for="(line, i) in logLines" :key="i" class="log-line">{{ line }}</div>
      <div v-if="logLines.length === 0" class="log-empty">
        Bridge is {{ bridgeRunning ? 'running' : 'stopped' }}
      </div>
    </div>
  </a-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import {
  CheckCircleOutlined,
  CloseCircleOutlined,
  LoadingOutlined,
} from '@ant-design/icons-vue'
import { getBridgeStatus, getBridgeLogs } from '../api/bridge'

const props = defineProps<{
  bridgeRunning: boolean
}>()

type Status = 'starting' | 'running' | 'stopped' | 'error'
const status = ref<Status>(props.bridgeRunning ? 'starting' : 'stopped')
const logLines = ref<string[]>([])

const statusLabel = computed(() => {
  switch (status.value) {
    case 'starting': return 'Starting...'
    case 'running': return 'Running'
    case 'error': return 'Error'
    default: return 'Stopped'
  }
})

let statusInterval: ReturnType<typeof setInterval> | null = null

async function pollStatus() {
  try {
    const s = await getBridgeStatus()
    status.value = s === 'running' ? 'running' : 'stopped'
    if (s === 'running') {
      const lines = await getBridgeLogs()
      logLines.value = lines
    }
  } catch {
    status.value = 'error'
  }
}

function startPolling() {
  status.value = 'starting'
  logLines.value = []
  pollStatus()
  statusInterval = setInterval(pollStatus, 1000)
}

function stopPolling() {
  if (statusInterval) { clearInterval(statusInterval); statusInterval = null }
  status.value = 'stopped'
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
.log-panel {
  margin-top: 12px;
  background: #1a1a1a;
}

.log-panel :deep(.ant-card-head) {
  background: #141414;
  border-bottom: 1px solid #333;
  min-height: 40px;
}

.log-panel :deep(.ant-card-head-title) {
  padding: 8px 0;
  font-size: 13px;
}

.log-panel :deep(.ant-card-body) {
  padding: 0;
}

.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.health-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #888;
}

.health-badge.running { color: #52c41a; }
.health-badge.error { color: #ff4d4f; }
.health-badge.starting { color: #faad14; }
.health-badge.stopped { color: #888; }

.log-output {
  height: 320px;
  overflow-y: auto;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.6;
  background: #0d0d0d;
}

.log-empty {
  padding: 24px;
  text-align: center;
  color: #555;
}

.log-line {
  padding: 2px 12px;
  color: #b0b0b0;
  white-space: pre-wrap;
  word-break: break-all;
}

.log-line:hover {
  background: #1a1a1a;
}
</style>