<template>
  <div class="bridge-card glass fade-up">
    <div class="ring" :class="status">
      <span class="core" />
      <span class="pulse" />
    </div>
    <div class="meta">
      <div class="label">{{ statusLabel }}</div>
      <div class="sub">{{ status === 'running' ? t('bridge.serving') : status === 'starting' ? t('bridge.booting') : t('bridge.idle') }}</div>
      <div class="provider">
        <span class="dot" :class="status" />
        <span>{{ status }}</span>
        <span class="sep" />
        <span class="mono">{{ props.provider || '-' }}</span>
      </div>
    </div>
    <div class="url mono">
      <span class="url-label">{{ t("bridge.url") }}</span>
      <code>http://127.0.0.1:17761</code>
      <a-tooltip :title="t('bridge.copy')">
        <CopyOutlined class="copy" @click="copyUrl" />
      </a-tooltip>
    </div>
    <div class="btn-row">
      <a-button
        class="toggle"
        :type="status === 'running' ? 'default' : 'primary'"
        :danger="status === 'running'"
        :loading="loading"
        size="large"
        @click="$emit('toggle')"
      >
        <template #icon>
          <PoweroffOutlined v-if="status !== 'running'" />
          <PauseOutlined v-else />
        </template>
        {{ status === 'running' ? t('bridge.stop') : t('bridge.start') }}
      </a-button>
      <a-button
        class="logs-btn"
        type="default"
        size="large"
        @click="goToLogs"
      >
        <template #icon><CodeOutlined /></template>
        {{ t("logs.title") }}
      </a-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useLocale } from '../composables/useLocale'
import { PoweroffOutlined, PauseOutlined, CopyOutlined, CodeOutlined } from '@ant-design/icons-vue'

const { t } = useLocale()
const router = useRouter()

const props = defineProps<{
  status: string
  loading: boolean
  provider?: string
}>()

defineEmits<{
  toggle: []
}>()

const statusLabel = computed(() => {
  switch (props.status) {
    case 'running': return t('bridge.online')
    case 'starting': return t('bridge.starting')
    case 'error': return 'Error'
    default: return t('bridge.offline')
  }
})

function copyUrl() {
  navigator.clipboard?.writeText('http://127.0.0.1:17761').catch(() => {})
}

function goToLogs() {
  router.push('/logs')
}
</script>

<style scoped>
.bridge-card {
  display: grid;
  grid-template-columns: auto 1fr auto;
  grid-template-areas:
    "ring meta url"
    "toggle toggle toggle";
  align-items: center;
  gap: 16px 20px;
  padding: 20px 22px;
  border-radius: var(--r-lg);
}

.ring {
  grid-area: ring;
  position: relative;
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: rgba(248, 113, 113, 0.12);
  border: 1px solid rgba(248, 113, 113, 0.3);
}
.ring .core {
  width: 14px; height: 14px; border-radius: 50%;
  background: var(--err); box-shadow: 0 0 12px var(--err);
}
.ring .pulse {
  position: absolute; inset: 0; border-radius: 50%;
  border: 2px solid var(--err);
  opacity: .5;
  animation: pulse 2s ease-out infinite;
}

.ring.running {
  background: rgba(52, 211, 153, 0.12);
  border-color: rgba(52, 211, 153, 0.35);
}
.ring.running .core { background: var(--ok); box-shadow: 0 0 14px var(--ok); }
.ring.running .pulse { border-color: var(--ok); }

.ring.starting {
  background: rgba(251, 191, 36, 0.12);
  border-color: rgba(251, 191, 36, 0.35);
}
.ring.starting .core { background: var(--warn); box-shadow: 0 0 12px var(--warn); }
.ring.starting .pulse { border-color: var(--warn); }

@keyframes pulse {
  0% { transform: scale(0.85); opacity: .7; }
  100% { transform: scale(1.6); opacity: 0; }
}

.meta { grid-area: meta; min-width: 0; }
.meta .label { font-size: 16px; font-weight: 600; color: var(--text-1); }
.meta .sub { font-size: 12px; color: var(--text-3); margin-top: 2px; }
.meta .provider {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-3);
}
.meta .provider .dot {
  width: 6px; height: 6px; border-radius: 50%; display: inline-block;
}
.meta .provider .dot.running { background: var(--ok); }
.meta .provider .dot.stopped { background: var(--text-4); }
.meta .provider .sep { width: 1px; height: 12px; background: var(--border); display: inline-block; }
.meta .provider .mono { font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace; color: var(--text-2); }

.url {
  grid-area: url;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-2);
  font-size: 12px;
  white-space: nowrap;
}
.url .url-label { color: var(--text-3); font-size: 10px; text-transform: uppercase; letter-spacing: .8px; }
.url code { color: var(--brand-300); }
.url .copy { color: var(--text-3); cursor: pointer; padding: 2px; }
.url .copy:hover { color: var(--brand-300); }

.toggle { grid-area: toggle; }
.btn-row {
  grid-area: toggle;
  display: flex;
  gap: 10px;
}
.btn-row .toggle {
  flex: 1;
}
.btn-row .logs-btn {
  flex-shrink: 0;
}

@media (max-width: 640px) {
  .bridge-card {
    grid-template-columns: 1fr;
    grid-template-areas:
      "ring"
      "meta"
      "url"
      "toggle";
    text-align: center;
    justify-items: center;
  }
  .url { font-size: 11px; }
}
</style>
