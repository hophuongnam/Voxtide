import { describe, it, expect } from 'vitest';
import { coalesceTokens, createConfigStore, createTranscriptStore } from '../src/lib/stores.svelte';

describe('transcript store', () => {
  it('appends finals and clears live text on commit', () => {
    const t = createTranscriptStore();
    t.live({ status: 'original', text: 'Hel', chip: 'A', language: 'en' });
    t.live({ status: 'original', text: 'Hello', chip: 'A', language: 'en' });
    expect(t.liveOriginal).toBe('Hello');

    t.final({ status: 'original', text: 'Hello.', chip: 'A', language: 'en', ts_ms: 100 });
    expect(t.original).toHaveLength(1);
    expect(t.liveOriginal).toBe('');
  });

  it('translation lives in its own column', () => {
    const t = createTranscriptStore();
    t.live({ status: 'translation', text: 'Xin', chip: 'A', language: 'vi' });
    expect(t.liveTranslation).toBe('Xin');
    expect(t.liveOriginal).toBe('');
    t.final({ status: 'translation', text: 'Xin chào', chip: 'A', language: 'vi', ts_ms: 200 });
    expect(t.translation).toHaveLength(1);
    expect(t.liveTranslation).toBe('');
  });

  it('reset clears everything', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'a', chip: null, language: 'en', ts_ms: 0 });
    t.reset();
    expect(t.original).toHaveLength(0);
  });

  it('same-speaker finals coalesce ACROSS sentence boundaries (ASCII and CJK alike) — one speaker turn = one row', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'First sentence.', chip: 'B', language: 'en', ts_ms: 1 });
    t.final({ status: 'original', text: ' Second sentence.', chip: 'B', language: 'en', ts_ms: 2 });
    t.final({ status: 'original', text: '第三句话。', chip: 'B', language: 'zh', ts_ms: 3 });
    expect(t.original).toHaveLength(1);
    expect(t.original[0]!.text).toBe('First sentence. Second sentence.第三句话。');
  });

  it('speaker change starts a new row', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'A says hi', chip: 'A', language: 'en', ts_ms: 1 });
    t.final({ status: 'original', text: 'B replies', chip: 'B', language: 'en', ts_ms: 2 });
    expect(t.original).toHaveLength(2);
    expect(t.original[0]!.chip).toBe('A');
    expect(t.original[1]!.chip).toBe('B');
  });

  it('utteranceBreak() forces the next same-speaker final into a new row (both columns)', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original',    text: 'went to the store', chip: 'A', language: 'en', ts_ms: 1 });
    t.final({ status: 'translation', text: 'đi ra cửa hàng',     chip: 'A', language: 'vi', ts_ms: 1 });
    // Speech pause detected between utterances.
    t.utteranceBreak();
    t.final({ status: 'original',    text: 'then drove home',   chip: 'A', language: 'en', ts_ms: 2 });
    t.final({ status: 'translation', text: 'rồi lái xe về',      chip: 'A', language: 'vi', ts_ms: 2 });

    expect(t.original).toHaveLength(2);
    expect(t.translation).toHaveLength(2);
    expect(t.original[0]!.text).toBe('went to the store');
    expect(t.original[1]!.text).toBe('then drove home');
    expect(t.translation[1]!.text).toBe('rồi lái xe về');
  });

  it('utteranceBreak() before any final is a no-op (no empty rows)', () => {
    const t = createTranscriptStore();
    t.utteranceBreak();
    t.final({ status: 'original', text: 'hi', chip: 'A', language: 'en', ts_ms: 1 });
    expect(t.original).toHaveLength(1);
    expect(t.original[0]!.text).toBe('hi');
  });

  it('without utteranceBreak, same-speaker finals still coalesce (break is opt-in)', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'a ', chip: 'A', language: 'en', ts_ms: 1 });
    t.final({ status: 'original', text: 'b',  chip: 'A', language: 'en', ts_ms: 2 });
    expect(t.original).toHaveLength(1);
    expect(t.original[0]!.text).toBe('a b');
  });

  it('reset() clears a pending utterance break', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'one', chip: 'A', language: 'en', ts_ms: 1 });
    t.utteranceBreak();
    t.reset();
    t.final({ status: 'original', text: 'two', chip: 'A', language: 'en', ts_ms: 2 });
    t.final({ status: 'original', text: ' three', chip: 'A', language: 'en', ts_ms: 3 });
    // Break was discarded by reset; these coalesce normally.
    expect(t.original).toHaveLength(1);
    expect(t.original[0]!.text).toBe('two three');
  });

  it('clearLive() clears both live strings and keeps committed rows', () => {
    const t = createTranscriptStore();
    t.final({ status: 'original', text: 'kept', chip: 'A', language: 'en', ts_ms: 1 });
    t.live({ status: 'original', text: 'ghost', chip: 'A', language: 'en' });
    t.live({ status: 'translation', text: 'bóng ma', chip: 'A', language: 'vi' });
    t.clearLive();
    expect(t.liveOriginal).toBe('');
    expect(t.liveTranslation).toBe('');
    expect(t.original).toHaveLength(1);
  });

  it('original and translation produce matching row counts for the same speaker sequence', () => {
    const t = createTranscriptStore();
    // Speaker B utters multi-sentence content in both languages
    t.final({ status: 'original',    text: '后来我...', chip: 'B', language: 'zh', ts_ms: 1 });
    t.final({ status: 'original',    text: '改变了我。', chip: 'B', language: 'zh', ts_ms: 2 });
    t.final({ status: 'original',    text: '他说：...', chip: 'B', language: 'zh', ts_ms: 3 });
    t.final({ status: 'translation', text: 'Sau đó...', chip: 'B', language: 'vi', ts_ms: 1 });
    t.final({ status: 'translation', text: 'thay đổi tôi.', chip: 'B', language: 'vi', ts_ms: 2 });
    t.final({ status: 'translation', text: ' Ông ấy nói: ...', chip: 'B', language: 'vi', ts_ms: 3 });
    expect(t.original).toHaveLength(t.translation.length);
    expect(t.original).toHaveLength(1);
  });
});

describe('config store', () => {
  it('update() before the initial config load is a safe no-op (no IPC, no crash)', async () => {
    const c = createConfigStore();
    await expect(c.update({ mode: 'conversation' })).resolves.toBeUndefined();
    expect(c.config).toBeNull();
  });
});

describe('coalesceTokens (past-session viewer)', () => {
  it('coalesces same-speaker tokens across sentence boundaries, matches live store', () => {
    const out = coalesceTokens([
      { id: 1, session_id: 1, ts_ms: 1, text: 'First.',  language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 2, session_id: 1, ts_ms: 2, text: ' Second.', language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 3, session_id: 1, ts_ms: 1, text: 'Một.',     language: 'vi', status: 'translation', speaker: '1', is_break: 0 },
      { id: 4, session_id: 1, ts_ms: 2, text: ' Hai.',    language: 'vi', status: 'translation', speaker: '1', is_break: 0 },
    ]);
    expect(out.original).toHaveLength(1);
    expect(out.translation).toHaveLength(1);
    expect(out.original[0]!.text).toBe('First. Second.');
  });

  it('replays persisted utterance breaks: same-speaker rows split at break rows (both columns)', () => {
    const out = coalesceTokens([
      { id: 1, session_id: 1, ts_ms: 1, text: 'went to the store', language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 2, session_id: 1, ts_ms: 1, text: 'đi ra cửa hàng',    language: 'vi', status: 'translation', speaker: '1', is_break: 0 },
      { id: 3, session_id: 1, ts_ms: 2, text: '',                  language: null, status: 'none',        speaker: null, is_break: 1 },
      { id: 4, session_id: 1, ts_ms: 3, text: 'then drove home',   language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 5, session_id: 1, ts_ms: 3, text: 'rồi lái xe về',     language: 'vi', status: 'translation', speaker: '1', is_break: 0 },
    ]);
    // Same speaker throughout — without the break replay these collapse to 1 row.
    expect(out.original).toHaveLength(2);
    expect(out.translation).toHaveLength(2);
    expect(out.original[0]!.text).toBe('went to the store');
    expect(out.original[1]!.text).toBe('then drove home');
    expect(out.translation[1]!.text).toBe('rồi lái xe về');
  });

  it('break rows render nothing themselves (no empty rows)', () => {
    const out = coalesceTokens([
      { id: 1, session_id: 1, ts_ms: 1, text: '', language: null, status: 'none', speaker: null, is_break: 1 },
      { id: 2, session_id: 1, ts_ms: 2, text: 'hi', language: 'en', status: 'original', speaker: '1', is_break: 0 },
    ]);
    expect(out.original).toHaveLength(1);
    expect(out.original[0]!.text).toBe('hi');
    expect(out.translation).toHaveLength(0);
  });

  it('a break is consumed per column: each column splits once at the boundary', () => {
    const out = coalesceTokens([
      { id: 1, session_id: 1, ts_ms: 1, text: 'a1', language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 2, session_id: 1, ts_ms: 2, text: '',   language: null, status: 'none',        speaker: null, is_break: 1 },
      // Translation of the post-break utterance arrives before its original:
      // both columns must still split exactly once.
      { id: 3, session_id: 1, ts_ms: 3, text: 'b1-trans', language: 'vi', status: 'translation', speaker: '1', is_break: 0 },
      { id: 4, session_id: 1, ts_ms: 4, text: 'b1', language: 'en', status: 'original',    speaker: '1', is_break: 0 },
      { id: 5, session_id: 1, ts_ms: 5, text: ' b2', language: 'en', status: 'original',   speaker: '1', is_break: 0 },
    ]);
    expect(out.original.map((l) => l.text)).toEqual(['a1', 'b1 b2']);
    expect(out.translation.map((l) => l.text)).toEqual(['b1-trans']);
  });
});
