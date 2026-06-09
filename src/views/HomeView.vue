<template>
<div class="home">
    

    <section class="row">
      <BridgeStatus
        class="status-card"
        :status="bridgeStatus"
        :loading="loading"
        @toggle="toggleBridge"
      />
      <div class="stat-card glass fade-up">
        <div class="stat-head">
          <div class="title">
            <span class="bar" />
            <span>{{ t("codex.title") }}</span>
          </div>
          <a-tag v-if="bridgeStatus === 'running'" class="ver ok">{{ t("codex.live") }}</a-tag>
          <a-tag v-else class="ver off">{{ t("codex.idle") }}</a-tag>
        </div>
        <div class="kv">
          <div class="k">{{ t("codex.status") }}</div>
          <div class="v" :class="bridgeStatus">
            <span class="dot" /> {{ bridgeStatus }}
          </div>
          <div class="k">{{ t("codex.base_url") }}</div>
          <div class="v mono">http://127.0.0.1:17761</div>
          <div class="k">{{ t("codex.catalog") }}</div>
          <div class="v mono">~/.codex/evocode-model-catalog.json</div>
          <div class="k">{{ t("codex.provider") }}</div>
          <div class="v">
            <router-link to="/config" class="link">{{ t("codex.manage") }}</router-link>
          </div>
        </div>
        <a-button
          class="copy-btn"
          block
          @click="copySnippet"
        >
          <template #icon><CopyOutlined /></template>
          {{ copied ? t('codex.copied') : t('codex.copy') }}
        </a-button>
      </div>
    </section>

    <section ref="logsRef" class="logs">
      <LogPanel :bridge-running="bridgeStatus === 'running'" />
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useLocale } from "../composables/useLocale"
import { useRouter } from "vue-router"
import { CopyOutlined } from "@ant-design/icons-vue"
import { startBridge, stopBridge, getBridgeStatus, readConfig, getAppVersion } from "../api/bridge"
import BridgeStatus from "../components/BridgeStatus.vue"
import LogPanel from "../components/LogPanel.vue"

const router = useRouter()
const { t } = useLocale()
const bridgeStatus = ref("stopped")
const loading = ref(false)
const currentVersion = ref("")
const copied = ref(false)

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

const SNIPPET = `model = "MiniMax-M3"
model_provider = "anthropic"
model_context_window = 256000
model_auto_compact_token_limit = 220000
model_catalog_json = "~/.codex/evocode-model-catalog.json"

[model_providers.anthropic]
name = "minimax"
wire_api = "responses"
requires_openai_auth = true
base_url = "http://127.0.0.1:17761"`

function copySnippet() {
  navigator.clipboard?.writeText(SNIPPET).then(() => {
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  }).catch(() => {})
}

onMounted(async () => {
  await updateStatus()
  currentVersion.value = await getAppVersion()
})
</script>

<style scoped>
.home { display: flex; flex-direction: column; gap: 18px; }

.hero {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 20px;
  padding: 28px 28px;
  border-radius: var(--r-xl);
  position: relative;
  overflow: hidden;
}
.hero-content { display: flex; flex-direction: column; gap: 14px; }
.eyebrow {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 4px 10px; border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-3); font-size: 12px; width: max-content;
  border: 1px solid var(--border);
}
.eyebrow .dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--ok); box-shadow: 0 0 8px var(--ok);
}
.hero h1 {
  font-size: clamp(26px, 3.6vw, 36px);
  line-height: 1.15; font-weight: 700; margin: 4px 0 0;
  color: var(--text-1);
}
.lead { color: var(--text-3); max-width: 58ch; }
.lead code { color: var(--brand-300); background: var(--bg-elev-3); padding: 1px 6px; border-radius: 6px; font-size: 12.5px; }

.hero-cta { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 4px; }
.cta { box-shadow: var(--shadow-glow); }
.cta-secondary { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.cta-secondary:hover { border-color: var(--border-strong); }

.hero-stats {
  display: inline-flex; align-items: center; gap: 18px;
  margin-top: 6px; padding: 10px 14px; border-radius: var(--r-md);
  background: var(--bg-elev-2); border: 1px solid var(--border); width: max-content;
}
.stat { display: flex; flex-direction: column; align-items: flex-start; }
.stat .num { font-size: 18px; font-weight: 700; color: var(--text-1); }
.stat .lbl { font-size: 11px; color: var(--text-3); text-transform: uppercase; letter-spacing: .8px; }
.sep { width: 1px; height: 24px; background: var(--border); }

/* Hero art */
.hero-art {
  position: relative; min-height: 220px; border-radius: var(--r-lg);
  background: linear-gradient(135deg, rgba(77,125,255,0.12), rgba(139,92,246,0.08));
  border: 1px solid var(--border); overflow: hidden;
}
.grid-bg {
  position: absolute; inset: 0;
  background-image:
    linear-gradient(rgba(255,255,255,0.05) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255,255,255,0.05) 1px, transparent 1px);
  background-size: 22px 22px;
  mask-image: radial-gradient(circle at 60% 50%, black 0%, transparent 70%);
}
.orb { position: absolute; border-radius: 50%; filter: blur(40px); opacity: .6; }
.orb-a { width: 180px; height: 180px; background: #4d7dff; top: -40px; right: -20px; }
.orb-b { width: 140px; height: 140px; background: #8b5cf6; bottom: -30px; left: 10px; }
.orb-c { width: 100px; height: 100px; background: #22d3ee; top: 30%; left: 40%; opacity: .35; }

.card-stack {
  position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: center;
  gap: 10px; padding: 18px 22px;
}
.mini-card {
  display: flex; align-items: center; justify-content: space-between; gap: 12px;
  padding: 10px 12px; border-radius: 10px;
  background: var(--bg-glass); border: 1px solid var(--border);
  backdrop-filter: blur(10px);
  animation: float 6s ease-in-out infinite;
}
.mini-card:nth-child(2) { margin-left: 28px; animation-delay: .6s; }
.mini-card:nth-child(3) { margin-left: 14px; animation-delay: 1.2s; }
.mini-card .kv-key {
  font-size: 11px; color: var(--text-3);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  text-transform: uppercase; letter-spacing: .8px;
}
.mini-card code { color: var(--text-1); font-size: 12.5px; }
@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-4px); }
}

/* Row */
.row {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 16px;
}
.stat-card { padding: 18px 20px; display: flex; flex-direction: column; gap: 12px; }
.stat-head { display: flex; align-items: center; justify-content: space-between; }
.stat-head .title { display: inline-flex; align-items: center; gap: 10px; font-weight: 600; color: var(--text-1); }
.stat-head .bar {
  width: 3px; height: 14px; border-radius: 2px;
  background: linear-gradient(180deg, #22d3ee, #4d7dff);
}
.ver { border-radius: 999px; }
.ver.ok { background: rgba(52,211,153,0.12); border-color: rgba(52,211,153,0.35); color: #34d399; }
.ver.off { background: var(--bg-elev-3); border-color: var(--border); color: var(--text-3); }

.kv {
  display: grid;
  grid-template-columns: 100px 1fr;
  row-gap: 10px;
  column-gap: 12px;
  align-items: center;
}
.kv .k { color: var(--text-3); font-size: 12px; }
.kv .v { color: var(--text-1); font-size: 13px; display: inline-flex; align-items: center; gap: 6px; }
.kv .v .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--text-4); }
.kv .v.running { color: var(--ok); }
.kv .v.running .dot { background: var(--ok); box-shadow: 0 0 8px var(--ok); }
.kv .v.stopped .dot { background: var(--text-4); }
.link { color: var(--brand-300); }
.link:hover { color: var(--brand-200); }

.copy-btn { margin-top: 4px; background: var(--bg-elev-3); border-color: var(--border); color: var(--text-1); }
.copy-btn:hover { border-color: var(--border-strong); }

@media (max-width: 880px) {
  .hero { grid-template-columns: 1fr; }
  .hero-art { min-height: 180px; }
  .row { grid-template-columns: 1fr; }
}
</style>