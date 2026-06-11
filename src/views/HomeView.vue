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
        <span class="title">{{ t("session.title") }}</span>
        <span class="count">{{ sessionsTotal }} total</span>
        <div class="search-box">
          <input v-model="searchQuery" type="text" class="search-input" placeholder="搜索会话..." />
        </div>
      </div>
      <div class="sessions-grid">
        <div
          v-for="s in filteredSessions"
          :key="s.id"
          class="session-card-wrapper"
          @click="openSession(s.id)"
        >
          <ContextGrid :session="s" />
        </div>
      </div>
      <div v-if="sessionsTotal > pageSize" class="pagination">
        <a-button
          size="small"
          :disabled="currentPage === 0"
          @click="prevPage"
        >
          <LeftOutlined />
        </a-button>
        <span class="page-info">{{ currentPage + 1 }} / {{ totalPages }}</span>
        <a-button
          size="small"
          :disabled="(currentPage + 1) * pageSize >= sessionsTotal"
          @click="nextPage"
        >
          <RightOutlined />
        </a-button>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useLocale } from "../composables/useLocale"
import { useRouter } from "vue-router"

import { startBridge, stopBridge, getBridgeStatus, readConfig, getAppVersion, getSessions } from "../api/bridge"
import type { SessionInfo } from "../api/bridge"
import { LeftOutlined, RightOutlined } from "@ant-design/icons-vue"
import ContextGrid from "../components/ContextGrid.vue"
import BridgeStatus from "../components/BridgeStatus.vue"

const router = useRouter()
const bridgeStatus = ref("stopped")
const loading = ref(false)
const sessions = ref<SessionInfo[]>([])
const currentVersion = ref("")
const activeProvider = ref("")
const sessionsTotal = ref(0)
const currentPage = ref(0)
const pageSize = 8
const { t } = useLocale()
const searchQuery = ref("")

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

const totalPages = computed(() => Math.max(1, Math.ceil(sessionsTotal.value / pageSize)))

async function fetchSessions() {
  const result = await getSessions(currentPage.value * pageSize, pageSize)
  sessions.value = result.sessions
  sessionsTotal.value = result.total
}

const filteredSessions = computed(() => {
  if (!searchQuery.value.trim()) return sessions.value
  const q = searchQuery.value.toLowerCase()
  return sessions.value.filter(s =>
    s.name.toLowerCase().includes(q) ||
    s.model.toLowerCase().includes(q)
  )
})

function nextPage() {
  currentPage.value++
  fetchSessions()
}

function prevPage() {
  currentPage.value--
  fetchSessions()
}

function openSession(id: string) {
  router.push('/session/' + id)
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
    await fetchSessions()
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

.search-box {
  margin-left: auto;
}
.search-input {
  background: var(--bg-elev-3);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 4px 10px;
  font-size: 12px;
  color: var(--text-1);
  outline: none;
  width: 180px;
  transition: border-color .15s;
}
.search-input:focus {
  border-color: var(--brand-400);
}
.search-input::placeholder {
  color: var(--text-4);
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
.session-card-wrapper {
  cursor: pointer;
  width: 200px;
  flex-shrink: 0;
  transition: transform .12s ease, box-shadow .12s ease;
}
.session-card-wrapper:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-glow);
}

.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 8px 0;
}
.page-info {
  font-size: 12px;
  color: var(--text-3);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}
</style>







