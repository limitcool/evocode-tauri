<template>
  <a-layout class="app-shell">
    <a-layout-sider
      v-model:collapsed="collapsed"
      :trigger="null"
      collapsible
      :width="224"
      :collapsedWidth="72"
      class="app-sider"
    >
      <div class="brand" :class="{ collapsed }">
        <div class="logo">
          <ThunderboltOutlined />
        </div>
        <transition name="brand-fade">
          <span v-if="!collapsed" class="brand-text gradient-text">evocode</span>
        </transition>
      </div>

      <a-menu
        mode="inline"
        :selected-keys="[route.path]"
        class="app-menu"
        @click="onMenuClick"
      >
        <a-menu-item key="/">
          <template #icon><HomeOutlined /></template>
          <span>{{ t("dashboard") }}</span>
        </a-menu-item>
        <a-menu-item key="/config">
          <template #icon><SettingOutlined /></template>
          <span>{{ t("configuration") }}</span>
        </a-menu-item>
        <a-menu-item key="/about">
          <template #icon><InfoCircleOutlined /></template>
          <span>{{ t("about") }}</span>
        </a-menu-item>
      </a-menu>

      <div class="sider-footer">
        <div class="version-pill" v-if="!collapsed">
          <span class="dot ok" />
          <span>v{{ currentVersion || '0.0.0' }}</span>
        </div>
        <a-button
          v-if="!collapsed"
          type="text"
          size="small"
          class="update-btn"
          :loading="checkingUpdate"
          @click="handleCheckUpdate"
        >
          <template #icon><DownloadOutlined v-if="!updateAvailable" /><ArrowDownOutlined v-else style="color: var(--ok)" /></template>
          {{ updateText }}
        </a-button>
        <a-tooltip :title="collapsed ? t('expand') : t('collapse')">
          <a-button
            type="text"
            class="collapse-btn"
            @click="collapsed = !collapsed"
          >
            <MenuUnfoldOutlined v-if="collapsed" />
            <MenuFoldOutlined v-else />
          </a-button>
        </a-tooltip>
      </div>
    </a-layout-sider>

    <a-layout class="app-main">
      <a-layout-header class="app-header">
        <div class="header-left">
          <a-breadcrumb class="crumbs">
            <a-breadcrumb-item>
              <router-link to="/">{{ t("home") }}</router-link>
            </a-breadcrumb-item>
            <a-breadcrumb-item v-if="route.path !== '/'">
              {{ currentTitle }}
            </a-breadcrumb-item>
          </a-breadcrumb>
        </div>
        <div class="header-right">
<a-tooltip :title="locale === 'en' ? '\u4e2d\u6587' : 'English'">
            <a-button type="text" class="icon-btn lang-btn" @click="localeToggle">
              {{ locale === 'en' ? 'EN' : '\u4e2d\u6587' }}
            </a-button>
          </a-tooltip>
        </div>
      </a-layout-header>

      <a-layout-content class="app-content">
        <div class="content-inner fade-up">
          <slot />
        </div>
      </a-layout-content>
    </a-layout>

  <!-- Update modal -->
  <a-modal
    v-model:open="showUpdateModal"
    :title="t('update.found')"
    :footer="null"
    :closable="true"
    width="420px"
    class="update-modal"
    >
    <div class="update-modal-body">
      <ArrowDownOutlined class="update-modal-icon" />
      <p class="update-modal-version">{{ t('update.modal_version') }} <strong>v{{ latestVersion }}</strong></p>
      <p class="update-modal-current">{{ t('update.modal_current') }} v{{ currentVersion }}</p>
      <a-button type="primary" size="large" block @click="downloadUpdate">
        <template #icon><DownloadOutlined /></template>
        {{ t('update.download') }}
      </a-button>
      <a-button size="small" type="text" block class="update-modal-skip" @click="showUpdateModal = false">
        {{ t('update.modal_skip') }}
      </a-button>
    </div>
  </a-modal>
  </a-layout>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  HomeOutlined,
  SettingOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  ThunderboltOutlined,
  DownloadOutlined,
  ArrowDownOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons-vue'
import { useLocale } from '../composables/useLocale'
import { message } from 'ant-design-vue'
import { getAppVersion } from '../api/bridge'
import { checkUpdate } from '../api/check_update'
import { openUrl } from '@tauri-apps/plugin-opener'

const route = useRoute()
const router = useRouter()
const collapsed = ref(false)
const currentVersion = ref('')
const updateAvailable = ref(false)
const updateUrl = ref('')
const latestVersion = ref('')
const checkingUpdate = ref(false)
const showUpdateModal = ref(false)

const { locale, t, toggle: localeToggle } = useLocale()

const titleMap: Record<string, string> = {
  '/': t('dashboard'),
  '/config': t('configuration'),
}
const currentTitle = computed(() => titleMap[route.path] || 'Page')

const updateText = computed(() => {
  if (checkingUpdate.value) return ''
  if (updateAvailable.value) return 'v' + latestVersion.value
  return t('update.check')
})

async function handleCheckUpdate() {
  checkingUpdate.value = true
  try {
    const result = await checkUpdate()
    if (result.hasUpdate) {
      latestVersion.value = result.latestVersion
      updateUrl.value = result.releaseUrl
      updateAvailable.value = true
      showUpdateModal.value = true
    } else {
      latestVersion.value = ''
      updateAvailable.value = false
      message.success(t('update.up_to_date'), 2)
    }
  } catch {
    message.error(t('update.error'), 3)
  } finally {
    checkingUpdate.value = false
  }
}
async function downloadUpdate() {
  try {
    if (!updateUrl.value) { message.error("No update URL available"); return }
    await openUrl(updateUrl.value)
  } catch (e) {
    console.error('Failed to open URL:', e)
  }
  showUpdateModal.value = false
}

function onMenuClick({ key }: { key: string }) {
  if (key !== route.path) router.push(key)
}

onMounted(async () => {
  try {
    currentVersion.value = await getAppVersion()
  } catch {}
  // auto-check update on first launch
  checkingUpdate.value = true
  try {
    const result = await checkUpdate()
    if (result.hasUpdate) {
      latestVersion.value = result.latestVersion
      updateUrl.value = result.releaseUrl
      updateAvailable.value = true
      showUpdateModal.value = true
    }
  } catch {
    // silent fail for auto-check
  } finally {
    checkingUpdate.value = false
  }
})
</script>

<style scoped>
.app-shell {
  min-height: 100vh;
  background: transparent;
}

.app-sider {
  background: var(--bg-elev-1) !important;
  border-right: 1px solid var(--border);
  position: sticky;
  top: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  box-shadow: 1px 0 0 var(--border);
}

.app-sider :deep(.ant-layout-sider-children) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 18px 18px 16px;
  font-weight: 700;
  font-size: 18px;
  letter-spacing: 0.3px;
  height: var(--header-h);
  box-sizing: border-box;
}
.brand.collapsed { padding: 18px 0 16px; justify-content: center; }

.logo {
  width: 32px;
  height: 32px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: linear-gradient(135deg, #4d7dff 0%, #8b5cf6 100%);
  color: #fff;
  font-size: 16px;
  box-shadow: var(--shadow-glow);
  flex-shrink: 0;
}

.brand-text {
  font-size: 17px;
  font-weight: 700;
  white-space: nowrap;
}

.brand-fade-enter-active,
.brand-fade-leave-active {
  transition: opacity .2s ease;
}
.brand-fade-enter-from,
.brand-fade-leave-to {
  opacity: 0;
}

.app-menu {
  background: transparent !important;
  border-inline-end: none !important;
  padding: 8px 10px;
  flex: 1;
}
.app-menu :deep(.ant-menu-item) {
  border-radius: 10px;
  margin: 4px 0;
  height: 40px;
  line-height: 40px;
  color: var(--text-2);
}
.app-menu :deep(.ant-menu-item:hover) {
  color: var(--text-1);
  background: var(--bg-elev-3) !important;
}
.app-menu :deep(.ant-menu-item-selected) {
  background: linear-gradient(135deg, rgba(77,125,255,0.18), rgba(139,92,246,0.12)) !important;
  color: var(--text-1) !important;
  border: 1px solid rgba(77,125,255,0.35);
  box-shadow: inset 0 0 0 1px rgba(77,125,255,0.05);
}
.app-menu :deep(.ant-menu-item-selected .anticon) { color: var(--brand-300); }

.sider-footer {
  border-top: 1px solid var(--border);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: stretch;
}
.version-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-3);
  font-size: 12px;
  width: max-content;
  margin: 0 auto;
}
.version-pill .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--ok); box-shadow: 0 0 8px var(--ok); }
.update-btn {
  width: 100%;
  color: var(--text-3);
  font-size: 12px;
  height: 32px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.update-btn:hover { color: var(--text-1); background: var(--bg-elev-3); }
.collapse-btn {
  width: 100%;
  color: var(--text-3);
  display: flex;
  justify-content: center;
}

.app-main {
  background: transparent;
}

.app-header {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 28px;
  height: var(--header-h);
  background: var(--bg-glass);
  border-bottom: 1px solid var(--border);
  backdrop-filter: blur(14px) saturate(140%);
  -webkit-backdrop-filter: blur(14px) saturate(140%);
}

.crumbs :deep(.ant-breadcrumb-link),
.crumbs :deep(.ant-breadcrumb-separator) {
  color: var(--text-3);
}
.crumbs :deep(a) { color: var(--text-2); }
.crumbs :deep(a:hover) { color: var(--brand-300); }

.icon-btn {
  color: var(--text-2);
  width: 36px;
  height: 36px;
  border-radius: 10px;
}
.icon-btn:hover { color: var(--text-1); background: var(--bg-elev-3); }
.lang-btn { font-weight: 600; font-size: 13px; letter-spacing: 0.5px; width: auto; padding: 0 8px; }

.app-content {
  padding: 24px 28px 40px;
  min-height: calc(100vh - var(--header-h));
}
.content-inner {
  max-width: 1080px;
  margin: 0 auto;
}

@media (max-width: 720px) {
  .app-header { padding: 0 16px; }
  .app-content { padding: 16px 14px 32px; }
  .app-sider { position: fixed; z-index: 20; }
}
</style>

.update-modal-body {
  text-align: center;
  padding: 12px 0;
}
.update-modal-icon {
  font-size: 48px;
  color: var(--ok);
  display: block;
  margin: 0 auto 16px;
}
.update-modal-version {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-1);
  margin-bottom: 4px;
}
.update-modal-current {
  font-size: 13px;
  color: var(--text-3);
  margin-bottom: 20px;
}
.update-modal-skip {
  margin-top: 8px;
  color: var(--text-4);
}
.update-modal :deep(.ant-modal-header) {
  text-align: center;
}










