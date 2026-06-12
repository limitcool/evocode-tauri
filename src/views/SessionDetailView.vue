<template>
  <div class="session-detail">
    <section class="page-header">
      <a-button type="text" class="back-btn" @click="router.back()">
        <template #icon><LeftOutlined /></template>
        {{ t("session.back") }}
      </a-button>
      <div class="page-title">
        <span class="bar" />
        <span :title="sessionTitle">{{ sessionTitle || t("session.detail") }}</span>
      </div>
      <div class="header-spacer" />
      <a-segmented
        v-model:value="viewMode"
        :options="viewOptions"
        size="small"
      />
    </section>

    <section class="session-meta" v-if="sessionInfo">
      <div class="meta-item">
        <span class="meta-label">{{ t("session.model") }}</span>
        <span class="meta-value">{{ sessionInfo.model }}</span>
      </div>
      <div class="meta-item">
        <span class="meta-label">{{ t("session.tokens") }}</span>
        <span class="meta-value">{{ displayTokens(sessionInfo.used_tokens, sessionInfo.used) }} / {{ displayTokens(sessionInfo.total_tokens, sessionInfo.total) }}</span>
      </div>
      <div class="meta-item" v-if="entries.length">
        <span class="meta-label">{{ t("session.entries") || "Entries" }}</span>
        <span class="meta-value">{{ entries.length }}</span>
      </div>
    </section>

    <!-- Chat / think thread view -->
    <section v-if="viewMode === 'chat'" class="thread" v-loading="loading">
      <template v-for="(item, idx) in zippedEntries" :key="idx">
        <!-- User prompt -->
        <div
          v-if="item.kind === 'user'"
          class="msg-row user"
        >
          <div class="bubble user-bubble">
            <div class="bubble-meta">
              <span class="role-tag user-tag">USER</span>
              <span class="ts" v-if="item.timestamp">{{ formatTs(item.timestamp) }}</span>
            </div>
            <div class="bubble-body md" v-html="renderMarkdown(item.text)" />
          </div>
        </div>

        <!-- Assistant text -->
        <div
          v-else-if="item.kind === 'assistant'"
          class="msg-row assistant"
        >
          <div class="bubble assistant-bubble">
            <div class="bubble-meta">
              <span class="role-tag assistant-tag">ASSISTANT</span>
              <span class="ts" v-if="item.timestamp">{{ formatTs(item.timestamp) }}</span>
            </div>
            <div class="bubble-body md" v-html="renderMarkdown(item.text)" />
          </div>
        </div>

        <!-- Reasoning / thinking -->
        <details
          v-else-if="item.kind === 'reasoning'"
          class="think-block"
        >
          <summary>
            <span class="think-icon">💭</span>
            <span class="think-label">思考</span>
            <span class="think-preview">{{ truncate(item.text, 80) }}</span>
            <span class="ts" v-if="item.timestamp">{{ formatTs(item.timestamp) }}</span>
          </summary>
          <div class="think-body md" v-html="renderMarkdown(item.text)" />
        </details>

        <!-- Tool call (function or custom) -->
        <div
          v-else-if="item.kind === 'tool_call'"
          class="tool-card"
          :class="{ expanded: expandedToolCalls.has(item.call_id) }"
        >
          <div class="tool-head" @click="toggleTool(item.call_id)">
            <span class="tool-icon">{{ item.tool_kind === 'custom' ? '🛠' : '⚙' }}</span>
            <span class="tool-kind" :class="`kind-${item.tool_kind}`">{{ item.tool_kind === 'custom' ? 'CUSTOM' : 'FN' }}</span>
            <span class="tool-name">{{ item.name }}</span>
            <span class="tool-callid" :title="item.call_id">{{ shortId(item.call_id) }}</span>
            <span class="tool-chevron">{{ expandedToolCalls.has(item.call_id) ? '▾' : '▸' }}</span>
            <span class="ts" v-if="item.timestamp">{{ formatTs(item.timestamp) }}</span>
          </div>
          <div v-if="expandedToolCalls.has(item.call_id)" class="tool-body">
            <div v-if="formatToolArgs(item).length" class="tool-args">
              <div class="tool-sublabel">arguments</div>
              <pre><code>{{ formatToolArgs(item) }}</code></pre>
            </div>
            <div v-if="toolOutputs.has(item.call_id)" class="tool-output">
              <div class="tool-sublabel">
                output
                <span v-if="getOutputTruncated(item.call_id)" class="truncated-tag">truncated</span>
              </div>
              <pre><code>{{ toolOutputs.get(item.call_id) }}</code></pre>
            </div>
            <div v-else class="tool-output tool-output-pending">
              <div class="tool-sublabel">output</div>
              <pre><code>⏳ waiting for tool result…</code></pre>
            </div>
          </div>
        </div>

        <!-- Patch end (only emitted if there is no matching custom_tool_call_output) -->
        <div
          v-else-if="item.kind === 'patch_end'"
          class="patch-card"
          :class="{ ok: item.success, fail: !item.success }"
        >
          <div class="patch-head">
            <span class="patch-icon">{{ item.success ? '✓' : '✗' }}</span>
            <span class="patch-title">
              {{ item.success ? 'apply_patch' : 'apply_patch failed' }}
            </span>
            <span class="ts" v-if="item.timestamp">{{ formatTs(item.timestamp) }}</span>
          </div>
          <div v-if="item.stdout" class="patch-stdout md"><code>{{ item.stdout.trim() }}</code></div>
          <div v-if="item.stderr" class="patch-stderr md"><code>{{ item.stderr.trim() }}</code></div>
          <details v-for="d in item.diffs" :key="d.path" class="patch-diff">
            <summary>{{ d.path }}</summary>
            <pre v-if="d.diff"><code>{{ d.diff }}</code></pre>
            <div v-else class="diff-empty">no diff available</div>
          </details>
        </div>

        <!-- Turn boundary -->
        <div v-else-if="item.kind === 'turn_boundary'" class="turn-boundary">
          <span class="turn-line" />
          <span class="turn-label">
            turn · {{ (item.duration_ms / 1000).toFixed(1) }}s
            <span v-if="item.last_message" class="turn-tail"> · {{ truncate(item.last_message, 60) }}</span>
          </span>
          <span class="turn-line" />
        </div>
      </template>

      <div v-if="!loading && entries.length === 0" class="empty">
        No conversation entries in this session.
      </div>
    </section>

    <!-- Raw fallback view -->
    <section v-else class="raw-pane">
      <pre class="raw-content">{{ rawContent }}</pre>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useRoute, useRouter } from "vue-router"
import { LeftOutlined } from "@ant-design/icons-vue"
import { useLocale } from "../composables/useLocale"
import { formatTokens } from "../utils/format"
import { getSessionContent, getSessions } from "../api/bridge"
import type { SessionInfo, SessionEntry } from "../api/bridge"

const { t } = useLocale()
const route = useRoute()
const router = useRouter()

const sessionId = route.params.id as string
const sessionInfo = ref<SessionInfo | null>(null)
const sessionTitle = ref("")
const entries = ref<SessionEntry[]>([])
const loading = ref(false)
const viewMode = ref<"chat" | "raw">("chat")
const viewOptions = [
  { label: "Chat", value: "chat" },
  { label: "Raw", value: "raw" },
]
const expandedToolCalls = ref<Set<string>>(new Set())
const rawContent = ref("")

// Prefer the exact token count from the backend; fall back to
// cells * 10K for older builds that don't yet emit the precise fields.
function displayTokens(precise: number | undefined, cells: number | undefined): string {
  if (precise != null) return formatTokens(precise)
  if (cells == null) return "0"
  return formatTokens(cells * 10000)
}

// Pair every `tool_call` with the matching `tool_output` so the call card
// can render the result inline. The Rust layer emits them in chronological
// order, so a single forward pass is enough.
const toolOutputs = computed(() => {
  const m = new Map<string, string>()
  for (const e of entries.value) {
    if (e.kind === "tool_output") m.set(e.call_id, e.output)
  }
  return m
})

function getOutputTruncated(callId: string): boolean {
  for (const e of entries.value) {
    if (e.kind === "tool_output" && e.call_id === callId) return e.truncated
  }
  return false
}

function toggleTool(callId: string) {
  const next = new Set(expandedToolCalls.value)
  if (next.has(callId)) next.delete(callId)
  else next.add(callId)
  // Force re-render of the Set
  expandedToolCalls.value = next
}

// Mostly pass-through; just provide a stable key/value shape.
const zippedEntries = computed(() => entries.value)

function formatTs(ts: string): string {
  if (!ts) return ""
  // ISO timestamp → "HH:MM:SS" in local time. Compact enough for the meta
  // line; the full date isn't useful inside one session.
  const d = new Date(ts)
  if (Number.isNaN(d.getTime())) return ts
  return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" })
}

function shortId(id: string): string {
  if (!id) return ""
  return id.length > 16 ? id.slice(0, 8) + "…" + id.slice(-4) : id
}

function truncate(s: string, n: number): string {
  if (!s) return ""
  return s.length > n ? s.slice(0, n) + "…" : s
}

function formatToolArgs(call: Extract<SessionEntry, { kind: "tool_call" }>): string {
  if (!call.arguments) return ""
  // Try to pretty-print JSON; if the payload isn't valid JSON, fall back
  // to the raw text (this is how `apply_patch` shows up).
  try {
    return JSON.stringify(JSON.parse(call.arguments), null, 2)
  } catch {
    return call.arguments
  }
}

// Lightweight markdown rendering: just fenced code blocks + inline
// `backticks` + paragraph breaks. We intentionally avoid pulling in a
// full markdown lib for this view.
function renderMarkdown(text: string): string {
  if (!text) return ""
  const escape = (s: string) =>
    s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
  let src = escape(text)
  // Fenced code blocks ``` ... ```
  src = src.replace(/```([\s\S]*?)```/g, (_, body) => {
    return `<pre class="md-pre"><code>${body.replace(/^\n/, "").replace(/\n$/, "")}</code></pre>`
  })
  // Inline code `...`
  src = src.replace(/`([^`\n]+)`/g, "<code class=\"md-code\">$1</code>")
  // Bold **...**
  src = src.replace(/\*\*([^*\n]+)\*\*/g, "<strong>$1</strong>")
  // Line breaks: blank line → new paragraph, single \n → <br>
  src = src
    .split(/\n{2,}/)
    .map((p) => {
      if (p.startsWith("<pre")) return p
      return `<p>${p.replace(/\n/g, "<br>")}</p>`
    })
    .join("")
  return src
}

onMounted(async () => {
  try {
    const allSessions = await getSessions(0, 1000)
    const found = allSessions.sessions.find(s => s.id === sessionId)
    if (found) {
      sessionInfo.value = found
      sessionTitle.value = found.name
    }
  } catch {}
  loading.value = true
  try {
    entries.value = await getSessionContent(sessionId)
  } catch (e: any) {
    console.error("Failed to load session:", e)
    entries.value = []
  } finally {
    loading.value = false
  }
  // Build the raw view lazily — it's only for the "Raw" toggle.
  rawContent.value = entries.value
    .map((e) => JSON.stringify(e))
    .join("\n")
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
  min-width: 0;
}
.page-title > span:last-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.page-title .bar {
  width: 3px; height: 16px; border-radius: 2px;
  background: linear-gradient(180deg, var(--brand-400), var(--brand-700));
}
.header-spacer { flex: 1; }

.session-meta {
  display: flex; gap: 24px; flex-wrap: wrap;
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

/* ============ Chat / thread ============ */
.thread {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 76vh;
  overflow-y: auto;
  padding: 12px 4px;
}

.msg-row {
  display: flex;
  width: 100%;
}
.msg-row.user { justify-content: flex-end; }
.msg-row.assistant { justify-content: flex-start; }

.bubble {
  max-width: min(720px, 88%);
  padding: 10px 14px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--bg-elev-2);
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-1);
  word-wrap: break-word;
  overflow-wrap: anywhere;
}
.user-bubble {
  background: linear-gradient(180deg, rgba(77, 125, 255, 0.18), rgba(77, 125, 255, 0.10));
  border-color: rgba(77, 125, 255, 0.30);
}
.assistant-bubble {
  background: var(--bg-elev-2);
  border-color: var(--border);
}
.bubble-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 10px;
  color: var(--text-4);
  margin-bottom: 4px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}
.role-tag {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.04em;
  border: 1px solid var(--border);
}
.user-tag { background: rgba(77, 125, 255, 0.18); color: #4d7dff; }
.assistant-tag { background: rgba(34, 211, 238, 0.18); color: #22d3ee; }
.bubble-body { white-space: normal; }
.bubble-body :deep(p) { margin: 4px 0; }
.bubble-body :deep(.md-pre) {
  background: rgba(0, 0, 0, 0.30);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 10px;
  margin: 6px 0;
  overflow-x: auto;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
}
.bubble-body :deep(.md-code) {
  background: rgba(0, 0, 0, 0.20);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
}

/* ============ Thinking ============ */
.think-block {
  border: 1px dashed var(--border);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.02);
  padding: 0;
  font-size: 12px;
  color: var(--text-3);
}
.think-block summary {
  list-style: none;
  cursor: pointer;
  padding: 6px 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
}
.think-block summary::-webkit-details-marker { display: none; }
.think-icon { font-size: 13px; }
.think-label { font-weight: 600; color: var(--text-2); }
.think-preview {
  flex: 1;
  color: var(--text-4);
  font-style: italic;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 11px;
}
.think-body {
  padding: 6px 14px 10px;
  border-top: 1px dashed var(--border);
  white-space: normal;
  color: var(--text-2);
  font-size: 12px;
  line-height: 1.55;
}
.think-body :deep(p) { margin: 4px 0; }

/* ============ Tool cards ============ */
.tool-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-elev-2);
  font-size: 12px;
  overflow: hidden;
}
.tool-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  cursor: pointer;
  user-select: none;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}
.tool-head:hover { background: rgba(255, 255, 255, 0.02); }
.tool-icon { font-size: 13px; }
.tool-kind {
  display: inline-block;
  padding: 0 6px;
  border-radius: 3px;
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.04em;
}
.kind-function { background: rgba(77, 125, 255, 0.18); color: #4d7dff; }
.kind-custom { background: rgba(34, 211, 238, 0.18); color: #22d3ee; }
.tool-name { font-weight: 600; color: var(--text-1); }
.tool-callid { color: var(--text-4); font-size: 10px; }
.tool-chevron { color: var(--text-3); }
.tool-body {
  border-top: 1px solid var(--border);
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.tool-sublabel {
  font-size: 10px;
  text-transform: uppercase;
  color: var(--text-4);
  letter-spacing: 0.06em;
  margin-bottom: 4px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}
.tool-args pre,
.tool-output pre {
  margin: 0;
  background: rgba(0, 0, 0, 0.30);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 10px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 11.5px;
  color: var(--text-1);
  max-height: 360px;
  overflow-y: auto;
}
.tool-output-pending pre { opacity: 0.5; font-style: italic; }
.truncated-tag {
  display: inline-block;
  margin-left: 6px;
  padding: 0 4px;
  border-radius: 3px;
  font-size: 9px;
  background: rgba(255, 165, 0, 0.18);
  color: #f59e0b;
  text-transform: none;
  letter-spacing: 0;
}

/* ============ Patch cards ============ */
.patch-card {
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 7px 12px;
  font-size: 12px;
  background: var(--bg-elev-2);
}
.patch-card.ok { border-color: rgba(34, 197, 94, 0.30); }
.patch-card.fail { border-color: rgba(239, 68, 68, 0.30); }
.patch-head {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-weight: 600;
}
.patch-card.ok .patch-icon { color: #22c55e; }
.patch-card.fail .patch-icon { color: #ef4444; }
.patch-stdout, .patch-stderr {
  margin-top: 4px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 11.5px;
  color: var(--text-2);
  white-space: pre-wrap;
}
.patch-stderr { color: #fca5a5; }
.patch-diff {
  margin-top: 4px;
  border-top: 1px dashed var(--border);
  padding-top: 4px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  font-size: 11.5px;
}
.patch-diff summary {
  cursor: pointer;
  color: var(--text-2);
  list-style: none;
  padding: 2px 0;
}
.patch-diff summary::-webkit-details-marker { display: none; }
.patch-diff pre {
  margin: 4px 0 0;
  background: rgba(0, 0, 0, 0.30);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 6px 10px;
  overflow-x: auto;
  white-space: pre;
  color: var(--text-1);
}
.diff-empty { color: var(--text-4); font-style: italic; padding: 2px 0; }

/* ============ Turn boundary ============ */
.turn-boundary {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 6px 0;
  color: var(--text-4);
  font-size: 11px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}
.turn-line {
  flex: 1;
  height: 1px;
  background: var(--border);
}
.turn-label { white-space: nowrap; }
.turn-tail { color: var(--text-3); }

/* ============ Empty state ============ */
.empty {
  text-align: center;
  color: var(--text-4);
  padding: 40px 20px;
  font-size: 13px;
}

.ts {
  font-size: 10px;
  color: var(--text-4);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  margin-left: auto;
}

/* ============ Raw pane ============ */
.raw-pane { max-height: 76vh; overflow: auto; }
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
}
</style>
