<template>
  <div class="logs-page">
    <section class="page-header">
      <div class="page-title">
        <span class="bar" />
        <span>{{ t("logs.title") }}</span>
      </div>
    </section>
    <section>
      <LogPanel :bridge-running="bridgeStatus === 'running'" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useLocale } from "../composables/useLocale"
import { getBridgeStatus } from "../api/bridge"
import LogPanel from "../components/LogPanel.vue"

const { t } = useLocale()
const bridgeStatus = ref("stopped")

onMounted(async () => {
  try {
    bridgeStatus.value = await getBridgeStatus()
  } catch {}
})
</script>

<style scoped>
.logs-page {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.page-title {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  font-weight: 600;
  font-size: 16px;
  color: var(--text-1);
}
.page-title .bar {
  width: 3px;
  height: 16px;
  border-radius: 2px;
  background: linear-gradient(180deg, var(--brand-400), var(--brand-700));
}
</style>
