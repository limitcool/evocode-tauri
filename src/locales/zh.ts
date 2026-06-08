export const zh = {
  // Sidebar
  "dashboard": "仪表盘",
  "configuration": "配置",
  "collapse": "收起",
  "expand": "展开",

  // Header
  "home": "首页",
  "theme.dark": "切换亮色模式",
  "theme.light": "切换深色模式",

  // Home - Hero
  "hero.eyebrow": "本地多协议桥接",
  "hero.lead": "将 Codex 指向 {url}，让 {name} 处理协议转换。启动下方的桥接服务，即可在一个本地端点上同时暴露 {c}、{m} 和 {r}。",
  "hero.configure": "配置 Provider",
  "hero.logs": "查看实时日志",
  "hero.default_port": "默认端口",
  "hero.bridge_version": "桥接版本",

  // Home - Codex Connection
  "codex.title": "Codex 连接",
  "codex.live": "在线",
  "codex.idle": "空闲",
  "codex.status": "状态",
  "codex.base_url": "基础 URL",
  "codex.catalog": "目录",
  "codex.provider": "Provider",
  "codex.manage": "在配置中管理 →",
  "codex.copy": "复制 Codex config.toml 配置片段",
  "codex.copied": "已复制！",

  // BridgeStatus
  "bridge.start": "启动桥接",
  "bridge.stop": "停止桥接",
  "bridge.running": "运行中",
  "bridge.stopped": "已停止",
  "bridge.port": "端口",
  "bridge.pid": "进程 ID",
  "bridge.uptime": "运行时长",

    // BridgeStatus (additional)
  "bridge.online": "桥接在线",
  "bridge.offline": "桥接离线",
  "bridge.starting": "启动中",
  "bridge.serving": "正在处理请求",
  "bridge.booting": "正在启动...",
  "bridge.idle": "空闲",
  "bridge.copy": "复制",
  "bridge.url": "URL",


  // Config - General
  "config.settings": "设置",
  "config.title": "配置",
  "config.desc": "选择预设 Provider，填入凭证，保存即可。可存储多个 Provider，随时切换。",
  "config.back": "返回仪表盘",

  // Config - Providers
  "config.providers": "Providers",
  "config.providers.desc": "可添加任意数量的 Provider，激活的 Provider 供桥接服务使用。",
  "config.providers.remove_title": "移除此 Provider？",
  "config.providers.ok": "是",
  "config.providers.cancel": "否",
  "config.providers.placeholder": "新 provider id",
  "config.providers.add": "添加",
  "config.providers.active": "当前激活：",

  // Config - Wire API
  "config.wire_api": "Wire API",
  "config.wire_api.desc": "选择协议。Model 和 Base URL 在下方分别填写。",
  "config.wire_api.tooltip": "上游 Provider 使用哪种 API 协议",

  // Config - Settings form
  "config.form.title": "Provider 设置",
  "config.form.desc": "这些值保存在",
  "config.form.reset": "重置",
  "config.form.empty": "请在上方添加 Provider。",
  "config.form.model": "模型",
  "config.form.model_placeholder": "例如 MiniMax-M3",
  "config.form.wire_api": "Wire API",
  "config.form.base_url": "Base URL",
  "config.form.base_url_placeholder": "https://api.example.com/v1",
  "config.form.api_key": "API 密钥",
  "config.form.api_key_placeholder": "您的 API 密钥",
  "config.form.api_key_header": "API 密钥 Header",
  "config.form.api_key_header_tooltip": "API 密钥的 HTTP 头部名称，例如 X-Api-Key 或 Authorization",
  "config.form.api_key_header_placeholder": "X-Api-Key",

  // Config - Wire options
  "config.wire.anthropic": "Anthropic（/v1/messages）",
  "config.wire.chat": "Chat Completions（/v1/chat）",
  "config.wire.openai": "OpenAI Responses（/responses）",

  // Config - Tabs
  "config.tab.connection": "连接",
  "config.tab.limits": "模型限制",

  // Config - Limits
  "config.limits.title": "上下文与自动压缩",
  "config.limits.desc": "拖动滑块或点击预设值。",
  "config.limits.context": "上下文窗口",
  "config.limits.compact": "自动压缩限制",
  "config.limits.tokens": "tokens",
  "config.limits.of": "占",
  "config.limits.preset_small": "小",
  "config.limits.preset_standard": "标准",
  "config.limits.preset_large": "大",
  "config.limits.preset_huge": "巨大",
  "config.limits.preset_128k": "小",
  "config.limits.preset_256k": "标准",
  "config.limits.preset_512k": "大",
  "config.limits.preset_1m": "巨大",

  // Config - Buttons
  "config.save": "保存配置",
  "config.sync": "同步到 Codex",

  // Logs
  "logs.title": "桥接日志",
  "logs.empty_title": "暂无日志",
  "logs.empty_sub": "桥接服务{status}。",
  "logs.running": "运行中",
  "logs.stopped": "已停止",
  "logs.refresh": "刷新",
  "logs.clear": "清空",
  "logs.auto_scroll": "自动滚动",
  "logs.label_starting": "启动中",
  "logs.label_running": "运行中",
  "logs.label_error": "错误",
  "logs.label_stopped": "已停止",

}
