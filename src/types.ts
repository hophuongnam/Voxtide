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
  // System Audio mode only: also capture the local mic (blended in → two-way).
  meeting_capture_mic: boolean;
  // Android face-to-face mic input gain (GainNode multiplier; 1.0 = unity).
  mic_gain: number;
  // Android: browser automatic gain control on the mic (off by default).
  mic_agc: boolean;
  // Optional free-text context (names, jargon, domain) sent to Soniox to bias
  // recognition and translation. Empty by default.
  context: string;
  // Saved library of named context presets (desktop only), managed in
  // Settings and picked per-session on the main screen. Empty by default.
  contexts: ContextPreset[];
  // The `id` of the currently selected preset in `contexts`, or `null` for
  // "no context". `null` by default.
  active_context_id: string | null;
}

// A single named, saved context preset (desktop only). `id` is opaque and
// frontend-generated (`crypto.randomUUID`); `text` is the free-text payload
// sent to Soniox as `context.text` when this preset is active. Mirrors
// `ContextPreset` in crates/voxtide-core/src/config.rs.
export interface ContextPreset {
  id: string;
  name: string;
  text: string;
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
