<template>
  <div class="session-detail">
    <section class="page-header">
      <a-button type="text" class="back-btn" @click="router.back()">
        <template #icon><LeftOutlined /></template>
        {{ t("session.back") }}
      </a-button>
      <div class="page-title">
        <span class="bar" />
        <span>{{ sessionTitle || t("session.detail") }}</span>
      </div>
    </section>

    <section class="session-meta" v-if="sessionInfo">
      <div class="meta-item">
        <span class="meta-label">{{ t("session.model") }}</span>
        <span class="meta-value">{{ sessionInfo.model }}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">{{ t("session.tokens") }}</span>
        <span class="meta-value">{{ (sessionInfo.used * 10000).toLocaleString() }} / {{ (sessionInfo.total * 10000).toLocaleString() }}</span>
      </div>
    </section>

    <pre class="raw-content">{{ rawContent }}</pre>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { LeftOutlined } from "@ant-design/icons-vue"
import { useLocale } from "../composables/useLocale"
import { getSessionContent, getSessions } from "../api/bridge"
import type { SessionInfo } from "../api/bridge"

const { t } = useLocale()
const route = useRoute()
const router = useRouter()

const sessionId = route.params.id as string
const rawContent = ref("")
const sessionInfo = ref<SessionInfo | null>(null)
const sessionTitle = ref("")

onMounted(async () => {
  try {
    const allSessions = await getSessions(0, 1000)
    const found = allSessions.sessions.find(s => s.id === sessionId)
    if (found) {
      sessionInfo.value = found
      sessionTitle.value = found.name
    }
  } catch {}
  try {
    const msgs = await getSessionContent(sessionId)
    rawContent.value = msgs.map(m => m.raw).join("\n")
  } catch (e: any) {
    console.error("Failed to load session:", e)
  }
})
</script>

<style scoped>
.session-detail {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.page-header {
  display: flex;
  align-items: center;
  gap: 8px;
}
.back-btn { color: var(--text-2); }
.back-btn:hover { color: var(--brand-300); }
.page-title {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  font-size: 16px;
  color: var(--text-1);
}
.page-title .bar {
  width: 3px; height: 16px; border-radius: 2px;
  background: linear-gradient(180deg, var(--brand-400), var(--brand-700));
}

.session-meta {
  display: flex; gap: 24px;
  padding: 12px 16px;
  background: var(--bg-elev-2);
  border-radius: var(--r-lg);
  border: 1px solid var(--border);
}
.meta-item { display: flex; gap: 8px; align-items: center; }
.meta-label { font-size: 12px; color: var(--text-3); }
.meta-value {
  font-size: 12px; color: var(--text-1);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}

.raw-content {
  margin: 0;
  padding: 16px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-2);
  max-height: 72vh;
  overflow: auto;
}
</style>
