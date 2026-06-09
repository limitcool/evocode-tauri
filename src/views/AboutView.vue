<template>
  <div class="about-view">
    <!-- About evocode -->
    <section class="card glass fade-up">
      <div class="card-head">
        <div class="eyebrow">
          <span class="dot" />
          <span>{{ t("about.title") }}</span>
        </div>
        <p class="card-desc">{{ t("about.desc") }}</p>
      </div>

      <!-- Core Features -->
      <div class="feature-grid">
        <div class="feature-item">
          <ApiOutlined class="feat-icon" />
          <span class="feat-text">{{ t("about.feat_messages") }}</span>
        </div>
        <div class="feature-item">
          <ApiOutlined class="feat-icon" />
          <span class="feat-text">{{ t("about.feat_chat") }}</span>
        </div>
        <div class="feature-item">
          <ApiOutlined class="feat-icon" />
          <span class="feat-text">{{ t("about.feat_responses") }}</span>
        </div>
        <div class="feature-item">
          <CheckCircleOutlined class="feat-icon ok" />
          <span class="feat-text">{{ t("about.feat_unified") }}</span>
        </div>
      </div>

      <!-- Technical Info -->
      <div class="tech-row">
        <div class="tech-item">
          <span class="tech-label">{{ t("about.port") }}</span>
          <code class="tech-value mono">17761</code>
        </div>
        <div class="tech-sep" />
        <div class="tech-item">
          <span class="tech-label">{{ t("about.version") }}</span>
          <code class="tech-value mono">v{{ currentVersion || "0.0.0" }}</code>
        </div>
      </div>
    </section>

    <!-- Team -->
    <section class="card glass fade-up">
      <div class="card-head">
        <div class="eyebrow">
          <span class="dot" />
          <span>{{ t("about.team") }}</span>
        </div>
        <p class="card-desc">{{ t("about.team_desc") }}</p>
      </div>

      <div class="team-body">
        <div class="team-info">
          <TeamOutlined class="team-icon" />
          <p>{{ t("about.team_what") }}</p>
        </div>
        <div class="team-contact">
          <h3>{{ t("about.contact") }}</h3>
          <div class="contact-row">
            <MailOutlined class="contact-icon" />
            <span class="contact-label">{{ t("about.email") }}:</span>
            <a href="https://mail.qq.com/cgi-bin/write?to=zhuxiujia@qq.com" target="_blank" class="contact-value">zhuxiujia@qq.com</a>
          </div>
          <div class="contact-row">
            <GithubOutlined class="contact-icon" />
            <span class="contact-label">GitHub:</span>
            <a href="https://github.com/zhuxiujia" target="_blank" class="contact-value">github.com/zhuxiujia</a>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue"
import { TeamOutlined, ApiOutlined, CheckCircleOutlined, MailOutlined, GithubOutlined } from "@ant-design/icons-vue"
import { useLocale } from "../composables/useLocale"
import { getAppVersion } from "../api/bridge"

const { t } = useLocale()
const currentVersion = ref("")

onMounted(async () => {
  try {
    currentVersion.value = await getAppVersion()
  } catch {}
})
</script>

<style scoped>
.about-view {
  max-width: 900px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.card {
  padding: 28px;
  border-radius: var(--r-xl);
}
.card-head { margin-bottom: 20px; }
.eyebrow {
  display: inline-flex; align-items: center; gap: 8px;
  padding: 4px 10px; border-radius: 999px;
  background: var(--bg-elev-3);
  color: var(--text-3); font-size: 12px; width: max-content;
  border: 1px solid var(--border); margin-bottom: 10px;
}
.eyebrow .dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--ok); box-shadow: 0 0 8px var(--ok);
}
.card-desc {
  color: var(--text-3);
  font-size: 13px;
  line-height: 1.7;
  margin: 0;
  max-width: 68ch;
}

/* Feature grid */
.feature-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 20px;
}
.feature-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 14px;
  border-radius: var(--r-md);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
}
.feat-icon {
  font-size: 16px;
  color: var(--brand-300);
  flex-shrink: 0;
}
.feat-icon.ok { color: var(--ok); }
.feat-text {
  font-size: 13px;
  color: var(--text-2);
  line-height: 1.4;
}

/* Tech info */
.tech-row {
  display: inline-flex;
  align-items: center;
  gap: 18px;
  padding: 12px 18px;
  border-radius: var(--r-md);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
  width: max-content;
}
.tech-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.tech-label {
  font-size: 11px;
  color: var(--text-3);
  text-transform: uppercase;
  letter-spacing: .8px;
}
.tech-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-1);
}
.tech-sep {
  width: 1px;
  height: 28px;
  background: var(--border);
}

/* Team section */
.team-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}
.team-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.team-icon {
  font-size: 28px;
  color: var(--brand-300);
}
.team-info p {
  color: var(--text-3);
  font-size: 13px;
  line-height: 1.7;
  margin: 0;
}
.team-contact h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-1);
  margin-bottom: 12px;
}
.contact-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: var(--r-md);
  background: var(--bg-elev-2);
  border: 1px solid var(--border);
}
.contact-icon {
  font-size: 14px;
  color: var(--text-3);
}
.contact-label {
  color: var(--text-3);
  font-size: 12px;
}
.contact-value {
  color: var(--brand-300);
  font-size: 13px;
  font-family: "JetBrains Mono", "SFMono-Regular", ui-monospace, monospace;
  text-decoration: none;
}
.contact-value:hover { text-decoration: underline; }

@media (max-width: 780px) {
  .feature-grid { grid-template-columns: 1fr; }
  .team-body { grid-template-columns: 1fr; }
}
</style>
