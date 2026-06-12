<template>
  <div class="session-grid-card glass fade-up">
    <!-- Header -->
    <div class="card-head">
      <div class="session-info">
        <div class="session-name">{{ session.name }}</div>
        <div class="session-model">{{ session.model }}</div>
      </div>
      <div class="session-stats">
        <span class="pct">{{ Math.min(100, Math.round(session.used / session.total * 100)) }}%</span>
      </div>
    </div>

    <!-- Grid -->
    <div class="grid" :style="gridStyle">
      <div
        v-for="(cell, idx) in cells"
        :key="idx"
        class="cell"
        :class="cell.cls"
        :style="{ '--d': cell.delay }"
        :title="`${session.name} 路 ${((idx + 1) * 10).toLocaleString()}K / ${(session.total * 10).toLocaleString()}K tokens`"
      />
    </div>

    <!-- Footer: token usage -->
    <div class="card-foot">
      <span class="foot-tokens">{{ tokensUsed }} / {{ tokensTotal }} tokens</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

export interface SessionData {
  id: string
  name: string
  model: string
  /** total cells (100 cells = 1M tokens) */
  total: number
  /** used cells */
  used: number
  /** Exact context window in tokens; preferred over `total * 10000`. */
  total_tokens?: number
  /** Exact current window in tokens; preferred over `used * 10000`. */
  used_tokens?: number
  /** Session-wide cumulative token usage (survives compacts). */
  accumulated?: number
  columns?: number
}

const props = withDefaults(defineProps<{
  session: SessionData
}>(), {})

const tokensUsed = computed(() => {
    // Prefer the exact token count when the backend supplied it; otherwise
    // fall back to cell * 10K (older builds). Never round the displayed
    // value back up to the next 10K boundary by recomputing from cells.
    if (props.session.used_tokens != null) {
      return props.session.used_tokens.toLocaleString()
    }
    if (props.session.total === 0) return '0'
    return (props.session.used * 10000).toLocaleString()
  })
const tokensTotal = computed(() => {
    if (props.session.total_tokens != null) {
      return props.session.total_tokens.toLocaleString()
    }
    return (props.session.total * 10000).toLocaleString()
  })

// Auto-calculate columns for a visually balanced grid
const cols = computed(() => {
  if (props.session.columns) return props.session.columns
  const t = props.session.total
  if (t <= 10) return 5
  if (t <= 25) return 5
  if (t <= 50) return 10
  if (t <= 100) return 10
  if (t <= 200) return 20
  return 20
})

const cells = computed(() => {
  const n = props.session.total
  const filled = Math.min(props.session.used, n)
  const colCount = cols.value
  const arr = []
  for (let i = 0; i < n; i++) {
    const ri = Math.floor(i / colCount)
    const ci = i % colCount
    const isFilled = i < filled
    const seed = ((ri * 7 + ci * 13) % 5)
    const level = isFilled ? Math.max(1, Math.min(4, seed)) : 0
    const delay = `${(ci * 0.02 + ri * 0.008).toFixed(3)}s`
    arr.push({
      filled: isFilled,
      level,
      delay,
      cls: isFilled ? `l${level}` : '',
    })
  }
  return arr
})

const gridStyle = computed(() => ({
  gridTemplateColumns: `repeat(${cols.value}, 1fr)`,
}))
</script>

<style scoped>
.session-grid-card {
  padding: 10px 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 200px;
}

/* Header */
.card-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}
.session-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}
.session-name {
  font-weight: 600;
  font-size: 11px;
  color: var(--text-1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.session-model {
  font-size: 10px;
  color: var(--text-4);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.session-stats {
  flex-shrink: 0;
}
.session-stats .pct {
  font-weight: 700;
  font-size: 13px;
  color: var(--text-1);
}

/* Grid */
.grid {
  display: grid;
  gap: 1.5px;
}

/* Cell */
.cell {
  aspect-ratio: 1;
  border-radius: 1px;
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  transition: background 0.2s ease, border-color 0.2s ease;
  animation: cellIn 0.3s ease both;
  animation-delay: var(--d);
  cursor: default;
}
.cell:hover {
  border-color: var(--text-3);
}

/* Filled cell levels */
.cell.l1 {
  background: rgba(77, 125, 255, 0.25);
  border-color: rgba(77, 125, 255, 0.15);
}
.cell.l2 {
  background: rgba(77, 125, 255, 0.45);
  border-color: rgba(77, 125, 255, 0.25);
}
.cell.l3 {
  background: rgba(34, 211, 238, 0.45);
  border-color: rgba(34, 211, 238, 0.25);
}
.cell.l4 {
  background: rgba(34, 211, 238, 0.70);
  border-color: rgba(34, 211, 238, 0.40);
}

:global(html[data-theme="light"]) .cell {
  background: var(--bg-elev-2);
  border-color: var(--border);
}
:global(html[data-theme="light"]) .cell.l1 {
  background: rgba(77, 125, 255, 0.15);
  border-color: rgba(77, 125, 255, 0.10);
}
:global(html[data-theme="light"]) .cell.l2 {
  background: rgba(77, 125, 255, 0.30);
  border-color: rgba(77, 125, 255, 0.18);
}
:global(html[data-theme="light"]) .cell.l3 {
  background: rgba(34, 211, 238, 0.30);
  border-color: rgba(34, 211, 238, 0.18);
}
:global(html[data-theme="light"]) .cell.l4 {
  background: rgba(34, 211, 238, 0.55);
  border-color: rgba(34, 211, 238, 0.30);
}

/* Footer */
.card-foot {
  display: flex;
  justify-content: flex-end;
}
.foot-tokens {
  font-size: 10px;
  color: var(--text-4);
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, Menlo, Consolas, monospace;
}

@keyframes cellIn {
  from {
    opacity: 0;
    transform: scale(0.5);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
