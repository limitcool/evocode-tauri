<template>
  <div class="home">
    <a-card class="intro-card" :bordered="false">
      <div class="intro">
        <div class="intro-text">
          <h1>evocode</h1>
          <p>Local multi-protocol bridge for Codex-style coding workflows.</p>
          <div class="endpoints">
            <span class="method">POST</span><code>/v1/chat/completions</code>
            <span class="method">POST</span><code>/v1/messages</code>
            <span class="method">POST</span><code>/responses</code>
          </div>
        </div>
        <div class="version-badge">v0.1.2</div>
      </div>
    </a-card>

    <BridgeStatus :status="bridgeStatus" :loading="loading" @toggle="toggleBridge" />

    <LogPanel :bridge-running="bridgeStatus === 'running'" />

    <div class="footer-hint">
      Configure via the <router-link to="/config">settings</router-link> icon above.
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { startBridge, stopBridge, getBridgeStatus, readConfig } from '../api/bridge'
import BridgeStatus from '../components/BridgeStatus.vue'
import LogPanel from '../components/LogPanel.vue'

const router = useRouter()
const bridgeStatus = ref('stopped')
const loading = ref(false)

async function updateStatus() {
  bridgeStatus.value = await getBridgeStatus()
}

async function toggleBridge() {
  if (bridgeStatus.value === 'running') {
    loading.value = true
    try {
      await stopBridge()
      await updateStatus()
    } finally {
      loading.value = false
    }
    return
  }

  // check provider config before starting
  try {
    const text = await readConfig()
    const hasProvider = text.includes('provider = "') && text.includes('[providers.')
    if (!hasProvider) {
      router.push('/config')
      return
    }
  } catch {}

  loading.value = true
  try {
    await startBridge()
    await updateStatus()
  } finally {
    loading.value = false
  }
}

onMounted(updateStatus)
</script>

<style scoped>
.home {
  max-width: 640px;
  margin: 0 auto;
  padding: 24px 20px;
}

.intro-card {
  margin-bottom: 16px;
}

.intro-card :deep(.ant-card-body) {
  padding: 20px;
}

.intro {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.intro-text h1 {
  font-size: 22px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 4px;
}

.intro-text p {
  font-size: 13px;
  color: #888;
  margin: 0 0 12px;
}

.endpoints {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.method {
  color: #60a5fa;
  font-family: monospace;
  font-size: 11px;
}

code {
  color: #b0b0b0;
  font-family: monospace;
}

.version-badge {
  font-size: 12px;
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.1);
  padding: 4px 10px;
  border-radius: 12px;
  border: 1px solid rgba(96, 165, 250, 0.2);
  white-space: nowrap;
}

.footer-hint {
  margin-top: 16px;
  text-align: center;
  font-size: 12px;
  color: #555;
}

.footer-hint a {
  color: #60a5fa;
  text-decoration: none;
}
</style>