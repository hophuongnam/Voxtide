import { describe, it, expect } from 'vitest';
import { coalesceTokens, createTranscriptStore } from '../src/lib/stores.svelte';

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

describe('coalesceTokens (past-session viewer)', () => {
  it('coalesces same-speaker tokens across sentence boundaries, matches live store', () => {
    const out = coalesceTokens([
      { id: 1, session_id: 1, ts_ms: 1, text: 'First.',  language: 'en', status: 'original',    speaker: '1' },
      { id: 2, session_id: 1, ts_ms: 2, text: ' Second.', language: 'en', status: 'original',    speaker: '1' },
      { id: 3, session_id: 1, ts_ms: 1, text: 'Một.',     language: 'vi', status: 'translation', speaker: '1' },
      { id: 4, session_id: 1, ts_ms: 2, text: ' Hai.',    language: 'vi', status: 'translation', speaker: '1' },
    ]);
    expect(out.original).toHaveLength(1);
    expect(out.translation).toHaveLength(1);
    expect(out.original[0]!.text).toBe('First. Second.');
  });
});
