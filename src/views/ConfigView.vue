<template>
  <div class="config-view">
    <header class="header">
      <router-link to="/" class="back-link">
        <LeftOutlined /> Back
      </router-link>
      <h2>Configuration</h2>
    </header>

    <a-form
      ref="formRef"
      :model="formState"
      layout="vertical"
      class="config-form"
      @finish="handleSave"
    >
      <a-divider orientation="left">General</a-divider>

      <!-- Provider section -->
      <a-form-item label="Provider" name="providerId">
        <div class="provider-row">
          <a-select
            v-model:value="formState.providerId"
            placeholder="Choose a provider..."
            style="flex:1"
            @change="onProviderChange"
          >
            <a-select-option value="">-- Choose --</a-select-option>
            <a-select-option v-for="id in providerIds" :key="id" :value="id">{{ id }}</a-select-option>
          </a-select>
          <a-input
            v-model:value="newProviderName"
            placeholder="New name"
            style="width: 120px"
          />
          <a-button type="default" :disabled="!newProviderName" @click="addProvider">+ Add</a-button>
        </div>
        <template #extra>
          <span class="field-hint">Or enter a name above and click Add to create a new provider.</span>
        </template>
      </a-form-item>

      <!-- Quick presets -->
      <div class="presets" v-if="!formState.providerId || newProviderName">
        <span class="presets-label">Quick start templates:</span>
        <a-button size="small" @click="applyPreset('openai_chat')">OpenAI Chat (Chat Completions)</a-button>
        <a-button size="small" @click="applyPreset('anthropic')">Anthropic (Messages API)</a-button>
      </div>

      <a-row :gutter="12">
        <a-col :span="12">
          <a-form-item label="Context Window" name="contextWindow">
            <a-input-number
              v-model:value="formState.contextWindow"
              :min="1"
              style="width:100%"
            />
            <template #extra><span class="field-hint">Max context size for the model.</span></template>
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="Auto Compact Limit" name="compactLimit">
            <a-input-number
              v-model:value="formState.compactLimit"
              :min="1"
              style="width:100%"
            />
            <template #extra><span class="field-hint">Start compacting when tokens exceed this.</span></template>
          </a-form-item>
        </a-col>
      </a-row>

      <!-- Provider config - only shows when provider is selected -->
      <template v-if="formState.providerId">
        <a-divider orientation="left">
          Provider: {{ formState.providerId }}
          <a-popconfirm title="Remove this provider?" ok-text="Yes" cancel-text="No" @confirm="removeProvider">
            <DeleteOutlined class="delete-icon" />
          </a-popconfirm>
        </a-divider>

        <a-form-item
          label="Model"
          name="model"
          :rules="[{ required: true, message: 'Please enter the model name' }]"
        >
          <a-input v-model:value="formState.model" placeholder="e.g. MiniMax-M2.7, gpt-4.1" />
        </a-form-item>

        <a-form-item
          label="Base URL"
          name="baseUrl"
          :rules="[{ required: true, message: 'Please enter the base URL' }]"
        >
          <a-input v-model:value="formState.baseUrl" placeholder="https://api.example.com/v1" />
        </a-form-item>

        <a-form-item label="API Key" name="apiKey">
          <a-input-password v-model:value="formState.apiKey" placeholder="Your API key" />
        </a-form-item>

        <a-row :gutter="12">
          <a-col :span="12">
            <a-form-item label="Wire API" name="wireApi">
              <a-tooltip title="Which API protocol this provider exposes: Anthropic (messages), Chat Completions, or OpenAI (responses)">
                <a-select v-model:value="formState.wireApi">
                  <a-select-option value="anthropic">Anthropic (/v1/messages)</a-select-option>
                  <a-select-option value="chat_completions">Chat Completions (/v1/chat)</a-select-option>
                  <a-select-option value="openai">OpenAI Responses (/responses)</a-select-option>
                </a-select>
              </a-tooltip>
            </a-form-item>
          </a-col>
          <a-col :span="12">
            <a-form-item label="API Key Header" name="apiKeyHeader">
              <a-tooltip title="The HTTP header name for the API key, e.g. X-Api-Key or Authorization">
                <a-input v-model:value="formState.apiKeyHeader" placeholder="X-Api-Key" />
              </a-tooltip>
            </a-form-item>
          </a-col>
        </a-row>
      </template>

      <a-form-item v-if="!formState.providerId">
        <a-alert type="info" message="Select or create a provider above to configure its settings." show-icon />
      </a-form-item>

      <a-form-item>
        <a-button
          type="primary"
          html-type="submit"
          block
          :loading="saving"
          :disabled="!formState.providerId"
        >
          Save Config
        </a-button>
      </a-form-item>

      <a-alert
        v-if="msg.text"
        :type="msg.type"
        :message="msg.text"
        show-icon
        closable
        @close="msg.text = ''"
      />
    </a-form>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, onMounted } from 'vue'
import { LeftOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import { readConfig, writeConfig } from '../api/bridge'

interface FormState {
  providerId: string
  contextWindow: number
  compactLimit: number
  wireApi: string
  baseUrl: string
  model: string
  apiKey: string
  apiKeyHeader: string
}

const PRESETS: Record<string, Partial<FormState>> = {
  openai_chat: {
    wireApi: 'chat_completions',
    baseUrl: 'https://api.openai.com/v1',
    model: 'gpt-4.1',
    apiKeyHeader: 'Authorization',
  },
  anthropic: {
    wireApi: 'anthropic',
    baseUrl: 'https://api.anthropic.com',
    model: 'claude-3-5-sonnet-latest',
    apiKeyHeader: 'X-Api-Key',
  },
}

const DEFAULT_CONTEXT_WINDOW = 256000
const DEFAULT_COMPACT_LIMIT = 220000

const providerIds = ref<string[]>([])
const newProviderName = ref('')
const saving = ref(false)
const msg = reactive({ text: '', type: 'success' as 'success' | 'error' | 'warning' })

const formState = reactive<FormState>({
  providerId: '',
  contextWindow: DEFAULT_CONTEXT_WINDOW,
  compactLimit: DEFAULT_COMPACT_LIMIT,
  wireApi: 'anthropic',
  baseUrl: '',
  model: '',
  apiKey: '',
  apiKeyHeader: 'X-Api-Key',
})

function applyPreset(name: string) {
  const preset = PRESETS[name]
  if (!preset) return
  if (!providerIds.value.includes(name)) {
    providerIds.value.push(name)
  }
  formState.providerId = name
  Object.assign(formState, preset)
}

function addProvider() {
  const name = newProviderName.value.trim()
  if (name && !providerIds.value.includes(name)) {
    providerIds.value.push(name)
    formState.providerId = name
    newProviderName.value = ''
    onProviderChange()
  }
}

function removeProvider() {
  const idx = providerIds.value.indexOf(formState.providerId)
  if (idx > -1) providerIds.value.splice(idx, 1)
  formState.providerId = ''
  onProviderChange()
}

function onProviderChange() {
  formState.model = ''
  formState.baseUrl = ''
  formState.apiKey = ''
  formState.wireApi = 'anthropic'
  formState.apiKeyHeader = 'X-Api-Key'
}

function parseConfig(text: string) {
  providerIds.value = []
  if (!text) return

  const lines = text.split('\n')
  let currentProvider = ''
  let inProviderSection = false

  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue

    if (trimmed.startsWith('provider = ')) {
      formState.providerId = trimmed.replace('provider = ', '').replace(/"/g, '')
    } else if (trimmed.startsWith('model_context_window = ')) {
      const val = parseInt(trimmed.replace('model_context_window = ', ''))
      if (!isNaN(val)) formState.contextWindow = val
    } else if (trimmed.startsWith('model_auto_compact_token_limit = ')) {
      const val = parseInt(trimmed.replace('model_auto_compact_token_limit = ', ''))
      if (!isNaN(val)) formState.compactLimit = val
    } else if (trimmed.startsWith('[providers.')) {
      currentProvider = trimmed.replace('[providers.', '').replace(']', '')
      inProviderSection = true
      if (!providerIds.value.includes(currentProvider)) {
        providerIds.value.push(currentProvider)
      }
    } else if (trimmed.startsWith('[') && trimmed !== '[providers.' + currentProvider + ']') {
      inProviderSection = false
    } else if (inProviderSection && currentProvider === formState.providerId) {
      if (trimmed.startsWith('wire_api = ')) {
        formState.wireApi = trimmed.replace('wire_api = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('base_url = ')) {
        formState.baseUrl = trimmed.replace('base_url = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('model = ')) {
        formState.model = trimmed.replace('model = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('api_key = ')) {
        formState.apiKey = trimmed.replace('api_key = ', '').replace(/"/g, '')
      } else if (trimmed.startsWith('api_key_header = ')) {
        formState.apiKeyHeader = trimmed.replace('api_key_header = ', '').replace(/"/g, '')
      }
    }
  }
}

function buildConfig(): string {
  const ctx = formState.contextWindow || DEFAULT_CONTEXT_WINDOW
  const compact = formState.compactLimit || DEFAULT_COMPACT_LIMIT
  const lines: string[] = [
    '# evocode bridge config',
    '# Read by evocode-cli, not by upstream Codex directly.',
    '',
    `provider = "${formState.providerId}"`,
    '',
    `model_context_window = ${ctx}`,
    `model_auto_compact_token_limit = ${compact}`,
    '',
    `[providers.${formState.providerId}]`,
    `wire_api = "${formState.wireApi}"`,
    `base_url = "${formState.baseUrl}"`,
    `model = "${formState.model}"`,
    `api_key = "${formState.apiKey}"`,
    `api_key_header = "${formState.apiKeyHeader}"`,
  ]
  return lines.join('\n')
}

async function handleSave() {
  if (!formState.providerId) {
    msg.text = 'Please select or add a provider first.'
    msg.type = 'error'
    return
  }
  if (!formState.model || !formState.baseUrl) {
    msg.text = 'Please fill in Model and Base URL for the provider.'
    msg.type = 'error'
    return
  }
  saving.value = true
  try {
    const content = buildConfig()
    await writeConfig(content)
    msg.text = 'Config saved! Restart the bridge to apply changes.'
    msg.type = 'success'
    setTimeout(() => { msg.text = '' }, 4000)
  } catch (e: any) {
    msg.text = 'Failed to save: ' + e.message
    msg.type = 'error'
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  try {
    const text = await readConfig()
    parseConfig(text)
  } catch {}
})
</script>

<style scoped>
.config-view {
  max-width: 640px;
  margin: 0 auto;
  padding: 24px 20px;
}

.header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}

.back-link {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #888;
  text-decoration: none;
  font-size: 14px;
  transition: color 0.2s;
}

.back-link:hover {
  color: #60a5fa;
}

.header h2 {
  font-size: 18px;
  font-weight: 500;
  color: #e0e0e0;
}

.config-form {
  background: #1a1a1a;
  border-radius: 10px;
  padding: 16px;
}

.provider-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.field-hint {
  font-size: 11px;
  color: #666;
}

.presets {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
}

.presets-label {
  font-size: 12px;
  color: #666;
}

.delete-icon {
  margin-left: 8px;
  color: #ef4444;
  cursor: pointer;
  font-size: 14px;
}

:deep(.ant-divider) {
  color: #666;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

:deep(.ant-form-item-label > label) {
  color: #888;
  font-size: 12px;
}
</style>