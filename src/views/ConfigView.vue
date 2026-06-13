<template>
  <div class="config-view">
    <a-tabs v-model:activeKey="activeKey" class="config-tabs fade-up">
      <a-tab-pane key="general" :tab="t('config.tab.general')">
        <div class="glass panel">
          <GeneralPanel />
        </div>
      </a-tab-pane>
      <a-tab-pane key="provider" :tab="t('config.tab.connection')">
        <ConnectionPanel />
      </a-tab-pane>
    </a-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue"
import { useLocale } from "../composables/useLocale"
import GeneralPanel from "./config_view/GeneralPanel.vue"
import ConnectionPanel from "./config_view/ConnectionPanel.vue"

const { t } = useLocale()

const activeKey = ref("general")
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
.head-actions { display: flex; gap: 8px; }
.ghost { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.ghost:hover { border-color: var(--border-strong); }

.config-tabs :deep(.ant-tabs-nav) { margin-bottom: 14px; }
.config-tabs :deep(.ant-tabs-tab) { padding: 8px 4px; color: var(--text-3); }
.config-tabs :deep(.ant-tabs-tab:hover) { color: var(--text-1); }
.config-tabs :deep(.ant-tabs-tab-active .ant-tabs-tab-btn) { color: var(--text-1) !important; }
.config-tabs :deep(.ant-tabs-ink-bar) { background: linear-gradient(90deg, #4d7dff, #8b5cf6); height: 3px; border-radius: 2px; }

.panel { padding: 20px 22px; }
.glass { background: var(--bg-glass); border: 1px solid var(--border); border-radius: var(--r-lg); backdrop-filter: blur(14px) saturate(140%); }

.actions {
  position: sticky; bottom: 16px; z-index: 5;
  display: flex; gap: 10px; justify-content: flex-end; flex-wrap: wrap;
  padding: 12px 16px; border-radius: var(--r-lg);
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  box-shadow: var(--shadow-md);
}

@media (max-width: 720px) {
  .actions { justify-content: stretch; }
  .actions .ant-btn { flex: 1; }
}
</style>
