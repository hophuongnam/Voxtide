import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

import type { AppConfig, ConnectionState, Mode, SessionRow } from '../types';

// --- keychain --------------------------------------------------------------
export const hasApiKey = (account: string) =>
  invoke<boolean>('has_api_key', { account });
export const setApiKey = (account: string, secret: string) =>
  invoke<void>('set_api_key', { account, secret });
export const clearApiKey = (account: string) =>
  invoke<void>('clear_api_key', { account });

// --- config ---------------------------------------------------------------
export const getConfig = () => invoke<AppConfig>('get_config');
export const setConfig = (cfg: AppConfig) => invoke<void>('set_config', { cfg });

/** Static backend facts for the status bar (model, audio format) — sourced
 *  from voxtide-core's real constants instead of frontend literals. */
export interface AppInfo { model: string; sample_rate_hz: number; channels: number; }
export const appInfo = () => invoke<AppInfo>('app_info');

// --- sessions / search ----------------------------------------------------
export const listSessions = (limit = 50) =>
  invoke<SessionRow[]>('list_sessions', { limit });
export interface TokenRow {
  id: number;
  session_id: number;
  ts_ms: number;
  text: string;
  language: string | null;
  status: string;
  speaker: string | null;
  /** 1 = persisted utterance-break marker (empty text; splits BOTH columns
   *  on replay), 0 = ordinary token. */
  is_break: number;
}
export const getSession = (id: number) =>
  invoke<{ session: SessionRow; tokens: TokenRow[] }>('get_session', { id });
/** Full-text search over transcript tokens, returning the matching SESSIONS
 *  (newest first) — independent of any sidebar cache, so matches in old
 *  sessions are reachable. */
export const searchTranscripts = (query: string, limit = 50) =>
  invoke<SessionRow[]>('search_transcripts', { query, limit });
export const deleteSession = (id: number) =>
  invoke<void>('delete_session', { id });

// --- devices --------------------------------------------------------------
export interface DeviceEntry { id: string; label: string; default: boolean; }
export const listMics = () => invoke<DeviceEntry[]>('list_mics');
export const listLoopbackSources = () => invoke<DeviceEntry[]>('list_loopback_sources');

// --- lifecycle ------------------------------------------------------------
export interface StartReq {
  mode: Mode;
  language_a: string;          // source (spoken) language
  language_b: string;          // target (translation) language
  device_id: string;          // mic id for Conversation, loopback id (or "system") for Meeting
  api_key_account: string;
}
export const startSession = (req: StartReq) => invoke<number>('start_session', { req });
export const stopSession  = () => invoke<void>('stop_session');

// --- overlay --------------------------------------------------------------
export const showOverlay = () => invoke<void>('show_overlay');
export const hideOverlay = () => invoke<void>('hide_overlay');
export const setOverlayClickThrough = (clickThrough: boolean) =>
  invoke<void>('set_overlay_click_through', { clickThrough });
/** Fires whenever the overlay window's real visibility changes (the
 *  show/hide commands emit after acting), so UI toggles can track the
 *  actual window state instead of a local guess. */
export function onOverlayVisibility(handler: (visible: boolean) => void): Promise<UnlistenFn> {
  return listen<{ visible: boolean }>('voxtide://overlay', (e) => handler(e.payload.visible));
}

// --- events ---------------------------------------------------------------
export type CoreEvent =
  | { kind: 'session-started'; session_id: number; mode: string }
  | { kind: 'session-stopped'; session_id: number; duration_ms: number }
  | { kind: 'transcript-live'; status: 'original' | 'translation' | 'none';
      text: string; language: string | null; chip: string | null }
  | { kind: 'transcript-final'; status: 'original' | 'translation' | 'none';
      text: string; language: string | null; chip: string | null; ts_ms: number }
  | { kind: 'utterance-break' }
  | { kind: 'connection-state'; state: ConnectionState['state']; attempt: number | null; retry_in_ms: number | null }
  | { kind: 'latency'; median_ms: number }
  | { kind: 'error'; message: string };

export function onCoreEvent(handler: (ev: CoreEvent) => void): Promise<UnlistenFn> {
  return listen<CoreEvent>('voxtide://event', (e) => handler(e.payload));
}

/** Fires with the full persisted config whenever set_config saves — lets
 *  other windows (overlay) re-derive labels/theme/hotkey hints live. */
export function onConfigChanged(handler: (cfg: AppConfig) => void): Promise<UnlistenFn> {
  return listen<AppConfig>('voxtide://config', (e) => handler(e.payload));
}
