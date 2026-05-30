<template>
  <div class="bridge-status">
    <div class="status-info">
      <div class="status-row">
        <span class="dot" :class="status" />
        <span class="label">{{ status }}</span>
      </div>
      <div class="url-row">
        <span class="url-label">URL:</span>
        <code>http://127.0.0.1:17761</code>
      </div>
    </div>
    <button class="toggle-btn" :class="status" :disabled="loading" @click="$emit('toggle')">
      {{ loading ? '...' : status === 'running' ? 'Stop' : 'Serve' }}
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  status: string
  loading: boolean
}>()

defineEmits<{
  toggle: []
}>()
</script>

<style scoped>
.bridge-status {
  background: #1a1a1a;
  border-radius: 12px;
  padding: 20px;
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ef4444;
}

.dot.running {
  background: #22c55e;
}

.label {
  font-size: 15px;
  text-transform: capitalize;
  color: #e0e0e0;
}

.url-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.url-label {
  color: #888;
}

.url-row code {
  color: #60a5fa;
  font-family: monospace;
}

.toggle-btn {
  width: 100%;
  padding: 12px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  color: white;
  background: #22c55e;
  cursor: pointer;
  transition: opacity 0.2s;
}

.toggle-btn.running {
  background: #ef4444;
}

.toggle-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
