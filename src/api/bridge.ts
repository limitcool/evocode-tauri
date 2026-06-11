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

export async function syncToCodex(): Promise<void> {
  return invoke<void>('sync_to_codex')
}

export async function getBridgeLogs(): Promise<string[]> {
  return invoke<string[]>('get_bridge_logs')
}

export async function getAppVersion(): Promise<string> {
  return invoke<string>('get_app_version')
}

export { checkUpdate, type CheckUpdateResult } from './check_update'
export async function openUrl(url: string): Promise<void> {
  return invoke<void>('open_url', { url })
}



export interface SessionInfo {
  id: string
  name: string
  model: string
  total: number
  used: number
  rollout_path: string
}

export interface SessionsResponse {
  sessions: SessionInfo[]
  total: number
}

export interface SessionMessage {
  timestamp: string
  text: string
  raw: string
}

export async function getSessions(offset: number, limit: number): Promise<SessionsResponse> {
  return invoke<SessionsResponse>('get_sessions', { offset, limit })
}

export async function getSessionContent(id: string): Promise<SessionMessage[]> {
  return invoke<SessionMessage[]>('get_session_content', { id })
}


