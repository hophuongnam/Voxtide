import type { Mode } from '../types';

// User-facing labels for the capture modes. The internal enum stays
// 'meeting' | 'conversation' (config + DB); only the display text differs.
export const MODE_LABELS: Record<Mode, string> = {
  meeting: 'System Audio',
  conversation: 'Microphone',
};
