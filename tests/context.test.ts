import { describe, it, expect } from 'vitest';
import type { AppConfig, ContextPreset } from '../src/types';
import { resolveActiveContext, seedContextFromLegacy } from '../src/lib/context';

const base: AppConfig = {
  language_a: 'en', language_b: 'vi', hotkey: 'CommandOrControl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
  mode: 'conversation', font_size: 'm', show_pinyin: false, mic_gain: 1, mic_agc: false,
  context: '', contexts: [], active_context_id: null,
};

const preset = (id: string, name: string, text: string): ContextPreset => ({ id, name, text });

describe('resolveActiveContext', () => {
  it('returns the active preset\'s text when active_context_id matches a preset', () => {
    const cfg: AppConfig = {
      ...base,
      contexts: [preset('a1', 'Standup', 'Speakers: Nam, Yuki.'), preset('a2', 'Client Acme', 'Acme Corp.')],
      active_context_id: 'a2',
    };
    expect(resolveActiveContext(cfg)).toBe('Acme Corp.');
  });

  it('returns "" when active_context_id is null', () => {
    const cfg: AppConfig = {
      ...base,
      contexts: [preset('a1', 'Standup', 'Speakers: Nam, Yuki.')],
      active_context_id: null,
    };
    expect(resolveActiveContext(cfg)).toBe('');
  });

  it('returns "" when active_context_id names a preset absent from contexts (deleted-active)', () => {
    const cfg: AppConfig = {
      ...base,
      contexts: [preset('a1', 'Standup', 'Speakers: Nam, Yuki.')],
      active_context_id: 'deleted-id',
    };
    expect(resolveActiveContext(cfg)).toBe('');
  });
});

describe('seedContextFromLegacy', () => {
  it('seeds exactly one preset from a non-empty legacy context, verbatim text, and clears context', () => {
    const cfg: AppConfig = { ...base, contexts: [], active_context_id: null, context: '  Acme Corp  ' };
    const result = seedContextFromLegacy(cfg);
    expect(result).not.toBeNull();
    const migrated = result!;
    expect(migrated.contexts).toHaveLength(1);
    expect(migrated.contexts[0]!.name).toBe('My context');
    // Verbatim: the trim-gate looks at the trimmed value, but the stored text
    // is the original, untrimmed cfg.context (Rust trims downstream).
    expect(migrated.contexts[0]!.text).toBe('  Acme Corp  ');
    expect(migrated.active_context_id).toBe(migrated.contexts[0]!.id);
    expect(migrated.context).toBe('');
  });

  it('is a no-op when contexts is already non-empty, even if legacy context is also set', () => {
    const cfg: AppConfig = {
      ...base,
      contexts: [preset('a1', 'Standup', 'Speakers: Nam, Yuki.')],
      active_context_id: 'a1',
      context: 'legacy leftover',
    };
    expect(seedContextFromLegacy(cfg)).toBeNull();
  });

  it('is a no-op when legacy context is empty or whitespace-only and contexts is empty', () => {
    expect(seedContextFromLegacy({ ...base, contexts: [], active_context_id: null, context: '' })).toBeNull();
    expect(seedContextFromLegacy({ ...base, contexts: [], active_context_id: null, context: '   ' })).toBeNull();
  });
});
