import { describe, it, expect } from 'vitest';
import { createTranscriptStore } from '../src/lib/stores.svelte';

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
});
