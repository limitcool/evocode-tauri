import { invoke } from '@tauri-apps/api/core'

export async function startBridge(): Promise<string> {
  return invoke<string>('start_bridge')
}

export async function stopBridge(): Promise<string> {
  return invoke<string>('stop_bridge')
}

export async function getBridgeStatus(): Promise<string> {
  return invoke<string>('bridge_status')
}

export async function getBridgeUrl(): Promise<string> {
  return invoke<string>('get_bridge_url')
}

export async function readConfig(): Promise<string> {
  return invoke<string>('read_config')
}

export async function writeConfig(content: string): Promise<void> {
  return invoke<void>('write_config', { content })
}

export async function getBridgeLogs(): Promise<string[]> {
  return invoke<string[]>('get_bridge_logs')
}

export async function getAppVersion(): Promise<string> {
  return invoke<string>('get_app_version')
}
