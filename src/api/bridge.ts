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

export interface ConnectivityResult {
  ok: boolean
  status: number
  latency_ms: number
  message: string
}

export async function testProviderConnectivity(
  baseUrl: string,
  apiKey: string,
  wireApi: string,
  apiKeyHeader?: string,
  model?: string,
): Promise<ConnectivityResult> {
  return invoke<ConnectivityResult>('test_provider_connectivity', {
    baseUrl,
    apiKey,
    wireApi,
    apiKeyHeader,
    model,
  })
}

export async function openConfigDir(): Promise<string> {
  return invoke<string>('open_config_dir')
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
  /** Exact context window in tokens. Optional for back-compat with older builds. */
  total_tokens?: number
  /** Exact current window in tokens. Optional for back-compat with older builds. */
  used_tokens?: number
  /** Session-wide cumulative token usage (kept across compacts). */
  accumulated?: number
  rollout_path: string
}

export interface SessionsResponse {
  sessions: SessionInfo[]
  total: number
}

/**
 * Discriminated union mirroring the Rust `SessionEntry` enum. The UI
 * switches on `kind` to pick the right renderer (chat bubble, thinking
 * block, tool call card, tool output, patch status, turn boundary).
 */
export type SessionEntry =
  | { kind: "user"; timestamp: string; text: string }
  | { kind: "assistant"; timestamp: string; text: string }
  | { kind: "reasoning"; timestamp: string; text: string }
  | {
      kind: "tool_call"
      timestamp: string
      tool_kind: "function" | "custom"
      name: string
      call_id: string
      /** JSON-encoded arguments / input. The UI pretty-prints this. */
      arguments: string
    }
  | {
      kind: "tool_output"
      timestamp: string
      call_id: string
      output: string
      truncated: boolean
    }
  | {
      kind: "patch_end"
      timestamp: string
      call_id: string | null
      success: boolean
      stdout: string
      stderr: string
      diffs: Array<{ path: string; diff: string | null }>
    }
  | {
      kind: "turn_boundary"
      timestamp: string
      last_message: string
      duration_ms: number
    }

export async function getSessions(offset: number, limit: number): Promise<SessionsResponse> {
  return invoke<SessionsResponse>('get_sessions', { offset, limit })
}

export async function getSessionContent(id: string): Promise<SessionEntry[]> {
  return invoke<SessionEntry[]>('get_session_content', { id })
}
