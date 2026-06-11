<template>

<div class="home">
    <section class="bridge-section">
      <BridgeStatus
        :status="bridgeStatus"
        :loading="loading"
        :provider="activeProvider"
        @toggle="toggleBridge"
      />
    </section>



    <section v-if="sessions.length > 0" class="sessions-section">
      <div class="sessions-header">
        <span class="bar" />
        <span class="title">Sessions</span>
        <span class="count">{{ sessions.length }} active</span>
      </div>
      <div class="sessions-grid">
        <ContextGrid v-for="s in sessions" :key="s.id" :session="s" />
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useRouter } from "vue-router"

import { startBridge, stopBridge, getBridgeStatus, readConfig, getAppVersion, getSessions } from "../api/bridge"
import type { SessionInfo } from "../api/bridge"
import ContextGrid from "../components/ContextGrid.vue"
import BridgeStatus from "../components/BridgeStatus.vue"

const router = useRouter()
const bridgeStatus = ref("stopped")
const loading = ref(false)
const sessions = ref<SessionInfo[]>([])
const currentVersion = ref("")
const activeProvider = ref("")

async function updateStatus() {
  bridgeStatus.value = await getBridgeStatus()
}

async function toggleBridge() {
  if (bridgeStatus.value === "running") {
    loading.value = true
    try { await stopBridge(); await updateStatus() }
    finally { loading.value = false }
    return
  }
  try {
    const text = await readConfig()
    const hasProvider = text.includes('provider = "') && text.includes("[providers.")
    if (!hasProvider) { router.push("/config"); return }
  } catch {}
  loading.value = true
  try { await startBridge(); await updateStatus() }
  finally { loading.value = false }
}

onMounted(async () => {
  await updateStatus()
  currentVersion.value = await getAppVersion()
  try {
    const cfg = await readConfig()
    // Parse provider name from config
    const lines = cfg.split('\n')
    for (const line of lines) {
      const trimmed = line.trim()
      if (trimmed.startsWith('provider = ')) {
        const match = trimmed.match(/provider = ["'](.+?)["']/)
        if (match) activeProvider.value = match[1]
        break
      }
    }
  } catch {}
  try {
    const result = await getSessions()
    sessions.value = result
  } catch (e) {
    console.error("Failed to fetch sessions:", e)
  }
})
</script>

<style scoped>
.home { display: flex; flex-direction: column; gap: 18px; }

/* Row */
.row {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 16px;
}

.sessions-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.sessions-header {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 0 4px;
}
.sessions-header .bar {
  width: 3px;
  height: 14px;
  border-radius: 2px;
  background: linear-gradient(180deg, #22d3ee, #4d7dff);
}
.sessions-header .title {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-1);
}
.sessions-header .count {
  font-size: 11px;
  color: var(--text-4);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  background: var(--bg-elev-3);
  padding: 1px 8px;
  border-radius: 999px;
}
.sessions-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
</style>