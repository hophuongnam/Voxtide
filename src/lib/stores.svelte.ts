import type { AppConfig, TranscriptLine, TranslationStatus } from '../types';
import { setConfig as persistConfig } from './ipc';
import type { DeviceEntry, TokenRow } from './ipc';

// Mirrors voxtide-core's SpeakerMap table: 26 chips before any wrap, so
// replay re-letters >4-speaker sessions the same way the live path did.
const LETTERS = Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i));

function asStatus(s: string): TranslationStatus {
  return s === 'original' || s === 'translation' ? s : 'none';
}

/**
 * Convert persisted DB tokens into the coalesced two-column shape the
 * live store produces. Same grouping rules as the live store (via the
 * shared {@link appendFinal} reducer): break on speaker-chip change or a
 * persisted utterance-break row — never on punctuation. Filters Soniox
 * angle-bracket control markers that may have been persisted before the
 * core-side filter landed.
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
  // Mirrors the live store's per-column pending-break flags: a break row arms
  // both columns; each column's next final consumes its own flag.
  let breakOriginal = false;
  let breakTranslation = false;
  for (const t of tokens) {
    if (t.is_break) {
      breakOriginal = true;
      breakTranslation = true;
      continue;
    }
    // Belt-and-braces marker filter (same '<…>' semantics as the backend's
    // open-time purge, which deletes these legacy rows for good). Keep for
    // one release past v0.1.6, then drop.
    if (t.text.startsWith('<') && t.text.endsWith('>')) continue;
    const status = asStatus(t.status);
    const isTrans = status === 'translation';
    const pendingBreak = isTrans ? breakTranslation : breakOriginal;
    if (isTrans) breakTranslation = false;
    else breakOriginal = false;
    appendFinal(
      isTrans ? out.translation : out.original,
      { status, text: t.text, chip: chipFor(t.speaker), language: t.language, ts_ms: t.ts_ms },
      pendingBreak,
    );
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

/**
 * THE row-grouping rule, shared by the live store and replay
 * ({@link coalesceTokens}) so the two can never drift: merge into the last
 * row when the speaker chip is unchanged AND no pause break is pending;
 * otherwise start a new row. We intentionally do NOT break on sentence-end
 * punctuation: ASCII `.!?` vs CJK `。！？` would tokenize asymmetrically
 * across languages, so the columns would desync. Speaker change and an
 * utterance break are the only row boundaries — both apply symmetrically
 * to both columns, preserving alignment.
 *
 * Mutates `list` in place and returns it. A merged row keeps the first
 * final's ts_ms (row timestamp = start of the speaker turn).
 */
export function appendFinal(
  list: TranscriptLine[],
  input: FinalInput,
  pendingBreak: boolean,
): TranscriptLine[] {
  const last = list[list.length - 1];
  if (last && last.chip === input.chip && !pendingBreak) {
    // In-place append: the arrays are deep $state proxies (live store) or
    // plain locals (replay) — either way no object/array copy per merge.
    last.text += input.text;
    return list;
  }
  list.push({
    ts_ms: input.ts_ms,
    status: input.status,
    text: input.text,
    language: input.language,
    chip: input.chip,
    live: false,
  });
  return list;
}

/** Normalize a language tag/code for comparison: lowercase, drop any region or
 *  script suffix (`zh-CN` → `zh`). Soniox echoes the configured 2-letter codes,
 *  but this insulates the face-to-face split from casing/region drift. */
export function normLang(l: string | null | undefined): string {
  return (l ?? '').toLowerCase().split('-')[0]!;
}

/**
 * Partition transcript lines into the two face-to-face panes by spoken language.
 * `far` = lines whose language matches `aCode`; `near` = EVERYTHING else.
 *
 * `near` is a deliberate catch-all (not a strict `bCode` match): a line carrying
 * an unexpected Soniox language tag can then never silently vanish — it stays
 * visible in the near pane instead of disappearing from both. Lines are merged
 * across the original+translation columns (each reader wants one monolingual
 * stream — their own speech plus translations INTO their language) and ordered
 * by ts_ms. Each line keeps its `status`, so the renderer can still mark
 * translated lines distinctly.
 */
export function splitByLanguage(
  lines: TranscriptLine[],
  aCode: string,
): { far: TranscriptLine[]; near: TranscriptLine[] } {
  const a = normLang(aCode);
  const far: TranscriptLine[] = [];
  const near: TranscriptLine[] = [];
  for (const l of [...lines].sort((x, y) => x.ts_ms - y.ts_ms)) {
    (normLang(l.language) === a ? far : near).push(l);
  }
  return { far, near };
}

export interface TranscriptStore {
  readonly original: TranscriptLine[];
  readonly translation: TranscriptLine[];
  readonly liveOriginal: string;
  readonly liveTranslation: string;
  /** Detected language of the in-flight partials, straight from the wire
   *  event (the column default is only a fallback — live zh under an EN
   *  column still gets its pinyin). */
  readonly liveOriginalLang: string | null;
  readonly liveTranslationLang: string | null;
  live(input: LiveInput): void;
  final(input: FinalInput): void;
  /** Speech pause detected — the next final in each column starts a new row
   *  even if the speaker is unchanged, so a long monologue is chunked by
   *  utterance instead of growing into one unreadable block. */
  utteranceBreak(): void;
  /** Drop the in-flight partials (e.g. on session stop — a leftover live
   *  line otherwise blinks forever under the committed transcript). */
  clearLive(): void;
  reset(): void;
}

export function createTranscriptStore(): TranscriptStore {
  let original = $state<TranscriptLine[]>([]);
  let translation = $state<TranscriptLine[]>([]);
  let liveOriginal = $state('');
  let liveTranslation = $state('');
  let liveOriginalLang = $state<string | null>(null);
  let liveTranslationLang = $state<string | null>(null);
  // Set by utteranceBreak(); consumed by the next final() in each column.
  // NOTE (suspected live-only desync edge): Soniox's translation final for the
  // PRE-pause utterance can arrive after the <end> break event. That late
  // final then consumes the translation column's pending flag, and the
  // post-pause translation merges into the old row — original splits,
  // translation doesn't. Replay is immune (break rows are positioned in the
  // persisted stream); confirm/deny during the live smoke.
  let breakOriginal = false;
  let breakTranslation = false;

  return {
    get original() { return original; },
    get translation() { return translation; },
    get liveOriginal() { return liveOriginal; },
    get liveTranslation() { return liveTranslation; },
    get liveOriginalLang() { return liveOriginalLang; },
    get liveTranslationLang() { return liveTranslationLang; },
    live(input) {
      if (input.status === 'translation') {
        liveTranslation = input.text;
        liveTranslationLang = input.language;
      } else {
        liveOriginal = input.text;
        liveOriginalLang = input.language;
      }
    },
    final(input) {
      const isTrans = input.status === 'translation';
      // A pending pause break is consumed by this final regardless of outcome.
      const pendingBreak = isTrans ? breakTranslation : breakOriginal;
      if (isTrans) breakTranslation = false; else breakOriginal = false;
      // Grouping lives in the shared appendFinal reducer (same rule as replay).
      // The $state arrays are deep proxies, so in-place mutation is reactive.
      appendFinal(isTrans ? translation : original, input, pendingBreak);
      if (isTrans) liveTranslation = '';
      else liveOriginal = '';
    },
    utteranceBreak() {
      breakOriginal = true;
      breakTranslation = true;
    },
    clearLive() {
      liveOriginal = '';
      liveTranslation = '';
      liveOriginalLang = null;
      liveTranslationLang = null;
    },
    reset() {
      original = [];
      translation = [];
      liveOriginal = '';
      liveTranslation = '';
      liveOriginalLang = null;
      liveTranslationLang = null;
      breakOriginal = false;
      breakTranslation = false;
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
  /** THE config-persist path: save the patched config to disk first
   *  (pessimistic — a failed save must not leave the UI claiming a state
   *  that didn't stick), then update the local store. No-op until the
   *  initial config has loaded. Rejections propagate to the caller. */
  update(patch: Partial<AppConfig>): Promise<void>;
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
    async update(patch) {
      if (!config) return;
      const next = { ...config, ...patch };
      await persistConfig(next);
      config = next;
    },
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
