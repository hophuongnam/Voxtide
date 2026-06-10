export const VERSION = '0.1.0';

export type Mode = 'meeting' | 'conversation';
export type TranslationStatus = 'original' | 'translation' | 'none';
export type Theme = 'light' | 'dark' | 'system';
export type FontSize = 'xs' | 's' | 'm' | 'l' | 'xl';

export interface AppConfig {
  language_a: string;
  language_b: string;
  hotkey: string;
  theme: Theme;
  default_meeting_source: string | null;
  default_mic: string | null;
  mode: Mode;
  font_size: FontSize;
  show_pinyin: boolean;
}

export interface SessionRow {
  id: number;
  started_at: number;
  ended_at: number | null;
  mode: string;
  lang_a: string;
  lang_b: string;
  device_label: string | null;
  duration_ms: number | null;
}

export interface TranscriptLine {
  ts_ms: number;
  status: TranslationStatus;
  text: string;
  language: string | null;
  chip: string | null;          // 'A'..'D'
  live: boolean;
}

export interface ConnectionState {
  state: 'active' | 'reconnecting' | 'idle';
  attempt: number | null;
  retry_in_ms: number | null;
}

// Structured rejection payload from the `start_session` command. `kind` routes
// the UI response (permission banner vs. plain error strip); `message` is the
// raw detail. Mirrors `StartError` in src-tauri/src/commands/lifecycle.rs.
export type StartError = {
  kind: 'device-missing' | 'mic-permission' | 'capture-permission' | 'other';
  message: string;
};
