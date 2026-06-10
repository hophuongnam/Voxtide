import type { Mode } from '../types';

// User-facing labels for the capture modes. The internal enum stays
// 'meeting' | 'conversation' (config + DB); only the display text differs.
export const MODE_LABELS: Record<Mode, string> = {
  meeting: 'System Audio',
  conversation: 'Microphone',
};

/** The app-default global toggle hotkey. Must stay in sync with
 *  `AppConfig::default()` in crates/voxtide-core/src/config.rs — used as a
 *  display fallback where the live config isn't available. */
export const DEFAULT_HOTKEY = 'CommandOrControl+Shift+V';
