import type { AppConfig, TranscriptLine, TranslationStatus } from '../types';
import type { DeviceEntry, TokenRow } from './ipc';

const LETTERS = ['A', 'B', 'C', 'D'];

function asStatus(s: string): TranslationStatus {
  return s === 'original' || s === 'translation' ? s : 'none';
}

/**
 * Convert persisted DB tokens into the coalesced two-column shape the
 * live store produces. Same coalesce rules: same-speaker consecutive
 * tokens merge into one line; break on sentence-end punctuation or
 * speaker change. Filters Soniox angle-bracket control markers that
 * may have been persisted before the core-side filter landed.
 */
export function coalesceTokens(tokens: TokenRow[]): {
  original: TranscriptLine[];
  translation: TranscriptLine[];
} {
  const speakerChip = new Map<string, string>();
  const chipFor = (speaker: string | null | undefined): string | null => {
    if (!speaker) return null;
    const existing = speakerChip.get(speaker);
    if (existing) return existing;
    const next = LETTERS[speakerChip.size % LETTERS.length]!;
    speakerChip.set(speaker, next);
    return next;
  };

  const out: { original: TranscriptLine[]; translation: TranscriptLine[] } = {
    original: [],
    translation: [],
  };
  for (const t of tokens) {
    if (t.text.startsWith('<') && t.text.endsWith('>')) continue;
    const status = asStatus(t.status);
    const list = status === 'translation' ? out.translation : out.original;
    const last = list[list.length - 1];
    const sentenceEnd = last && /[.!?]\s*$/.test(last.text);
    const chip = chipFor(t.speaker);
    if (last && last.chip === chip && !sentenceEnd) {
      list[list.length - 1] = { ...last, text: last.text + t.text };
      continue;
    }
    list.push({
      ts_ms: t.ts_ms,
      status,
      text: t.text,
      language: t.language,
      chip,
      live: false,
    });
  }
  return out;
}

export interface LiveInput {
  status: TranslationStatus;
  text: string;
  chip: string | null;
  language: string | null;
}
export interface FinalInput extends LiveInput { ts_ms: number; }

export interface TranscriptStore {
  readonly original: TranscriptLine[];
  readonly translation: TranscriptLine[];
  readonly liveOriginal: string;
  readonly liveTranslation: string;
  live(input: LiveInput): void;
  final(input: FinalInput): void;
  reset(): void;
}

export function createTranscriptStore(): TranscriptStore {
  let original = $state<TranscriptLine[]>([]);
  let translation = $state<TranscriptLine[]>([]);
  let liveOriginal = $state('');
  let liveTranslation = $state('');

  return {
    get original() { return original; },
    get translation() { return translation; },
    get liveOriginal() { return liveOriginal; },
    get liveTranslation() { return liveTranslation; },
    live(input) {
      if (input.status === 'translation') liveTranslation = input.text;
      else liveOriginal = input.text;
    },
    final(input) {
      const list = input.status === 'translation' ? translation : original;
      const last = list[list.length - 1];
      const sentenceEnd = last && /[.!?]\s*$/.test(last.text);
      if (last && last.chip === input.chip && !sentenceEnd) {
        const merged: TranscriptLine = { ...last, text: last.text + input.text };
        const next = list.slice(0, -1).concat(merged);
        if (input.status === 'translation') { translation = next; liveTranslation = ''; }
        else { original = next; liveOriginal = ''; }
        return;
      }
      const line: TranscriptLine = {
        ts_ms: input.ts_ms,
        status: input.status,
        text: input.text,
        language: input.language,
        chip: input.chip,
        live: false,
      };
      if (input.status === 'translation') {
        translation = [...translation, line];
        liveTranslation = '';
      } else {
        original = [...original, line];
        liveOriginal = '';
      }
    },
    reset() {
      original = [];
      translation = [];
      liveOriginal = '';
      liveTranslation = '';
    },
  };
}

// --- session + connection -------------------------------------------------
export interface SessionStore {
  readonly recording: boolean;
  readonly sessionId: number | null;
  readonly startedAt: number | null;
  readonly connection: { state: 'active' | 'reconnecting' | 'idle'; attempt: number | null; retry_in_ms: number | null };
  readonly latencyMs: number | null;
  start(id: number, startedAt: number): void;
  stop(): void;
  setConnection(state: 'active' | 'reconnecting' | 'idle', attempt: number | null, retry_in_ms: number | null): void;
  setLatency(ms: number): void;
}

export function createSessionStore(): SessionStore {
  let recording = $state(false);
  let sessionId = $state<number | null>(null);
  let startedAt = $state<number | null>(null);
  let connection = $state<{ state: 'active' | 'reconnecting' | 'idle'; attempt: number | null; retry_in_ms: number | null }>({ state: 'idle', attempt: null, retry_in_ms: null });
  let latencyMs = $state<number | null>(null);

  return {
    get recording() { return recording; },
    get sessionId() { return sessionId; },
    get startedAt() { return startedAt; },
    get connection() { return connection; },
    get latencyMs() { return latencyMs; },
    start(id, started) {
      sessionId = id; startedAt = started; recording = true;
    },
    stop() {
      recording = false; sessionId = null; startedAt = null;
      connection = { state: 'idle', attempt: null, retry_in_ms: null };
    },
    setConnection(s, a, r) { connection = { state: s, attempt: a, retry_in_ms: r }; },
    setLatency(ms) { latencyMs = ms; },
  };
}

// Singletons — one set per window. Overlay window subscribes to events through the IPC layer
// independently; it does not share these instances.
export const transcript = createTranscriptStore();
export const session = createSessionStore();

export interface ConfigStore {
  readonly config: AppConfig | null;
  readonly hasApiKey: boolean;
  readonly apiKeyAccount: string;
  setConfig(c: AppConfig | null): void;
  setHasApiKey(v: boolean): void;
}

export function createConfigStore(): ConfigStore {
  let config = $state<AppConfig | null>(null);
  let hasApiKey = $state(false);
  const apiKeyAccount = 'default';
  return {
    get config() { return config; },
    get hasApiKey() { return hasApiKey; },
    get apiKeyAccount() { return apiKeyAccount; },
    setConfig(c) { config = c; },
    setHasApiKey(v) { hasApiKey = v; },
  };
}

export interface DevicesStore {
  readonly mics: DeviceEntry[];
  readonly loopbacks: DeviceEntry[];
  setMics(v: DeviceEntry[]): void;
  setLoopbacks(v: DeviceEntry[]): void;
}

export function createDevicesStore(): DevicesStore {
  let mics = $state<DeviceEntry[]>([]);
  let loopbacks = $state<DeviceEntry[]>([]);
  return {
    get mics() { return mics; },
    get loopbacks() { return loopbacks; },
    setMics(v) { mics = v; },
    setLoopbacks(v) { loopbacks = v; },
  };
}

export const config = createConfigStore();
export const devices = createDevicesStore();
