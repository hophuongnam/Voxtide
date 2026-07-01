import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ContextSection from '../src/components/settings/ContextSection.svelte';
import type { AppConfig, ContextPreset } from '../src/types';

const base: AppConfig = {
  language_a: 'en', language_b: 'vi', hotkey: 'Ctrl+Shift+V',
  theme: 'system', default_meeting_source: null, default_mic: null, meeting_capture_mic: false,
  mode: 'conversation', font_size: 'm', show_pinyin: false, mic_gain: 1, mic_agc: false, context: '',
  contexts: [], active_context_id: null,
};

const standup: ContextPreset = { id: 'p1', name: 'Standup', text: 'Speakers: Nam, Yuki.' };
const acme: ContextPreset = { id: 'p2', name: 'Client Acme', text: 'Acme Corp. Topic: Q3 renewal.' };

describe('ContextSection', () => {
  it('renders one row per existing preset with its name and text', () => {
    const { getAllByLabelText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup, acme] }, onchange: vi.fn() },
    });
    const names = getAllByLabelText('Context name') as HTMLInputElement[];
    const texts = getAllByLabelText('Context text') as HTMLTextAreaElement[];
    expect(names).toHaveLength(2);
    expect(texts).toHaveLength(2);
    expect(names[0]!.value).toBe('Standup');
    expect(texts[0]!.value).toBe('Speakers: Nam, Yuki.');
    expect(names[1]!.value).toBe('Client Acme');
    expect(texts[1]!.value).toBe('Acme Corp. Topic: Q3 renewal.');
  });

  it('shows only the description and the Add button when there are no presets', () => {
    const { container, getByText, queryAllByLabelText } = render(ContextSection, {
      props: { cfg: base, onchange: vi.fn() },
    });
    expect(queryAllByLabelText('Context name')).toHaveLength(0);
    expect(queryAllByLabelText('Context text')).toHaveLength(0);
    expect(container.querySelectorAll('textarea')).toHaveLength(0);
    expect(getByText(/improves recognition and translation/i)).toBeTruthy();
    expect(getByText(/Add context/i)).toBeTruthy();
  });

  it('"Add context" appends a new preset with a non-empty id and calls onchange', async () => {
    const onchange = vi.fn();
    const { getByText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup] }, onchange },
    });
    await fireEvent.click(getByText(/Add context/i));

    expect(onchange).toHaveBeenCalledOnce();
    const next = onchange.mock.calls[0][0] as AppConfig;
    expect(next.contexts).toHaveLength(2);
    expect(next.contexts[0]).toEqual(standup);
    expect(next.contexts[1]!.id).toBeTruthy();
    expect(next.contexts[1]!.name).toBe('');
    expect(next.contexts[1]!.text).toBe('');
  });

  it('editing a name and blurring commits just that preset, others unchanged', async () => {
    const onchange = vi.fn();
    const { getAllByLabelText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup, acme] }, onchange },
    });
    const nameInput = getAllByLabelText('Context name')[0] as HTMLInputElement;
    await fireEvent.input(nameInput, { target: { value: '  Daily Standup  ' } });
    await fireEvent.blur(nameInput);

    expect(onchange).toHaveBeenCalledOnce();
    const next = onchange.mock.calls[0][0] as AppConfig;
    expect(next.contexts[0]).toEqual({ ...standup, name: 'Daily Standup' });
    expect(next.contexts[1]).toEqual(acme);
  });

  it('editing text and blurring commits just that preset, others unchanged', async () => {
    const onchange = vi.fn();
    const { getAllByLabelText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup, acme] }, onchange },
    });
    const textArea = getAllByLabelText('Context text')[1] as HTMLTextAreaElement;
    await fireEvent.input(textArea, { target: { value: '  Acme Corp. New scope.  ' } });
    await fireEvent.blur(textArea);

    expect(onchange).toHaveBeenCalledOnce();
    const next = onchange.mock.calls[0][0] as AppConfig;
    expect(next.contexts[0]).toEqual(standup);
    expect(next.contexts[1]).toEqual({ ...acme, text: 'Acme Corp. New scope.' });
  });

  it('does not call onchange when a field is blurred without being edited (no redundant write)', async () => {
    const onchange = vi.fn();
    const { getAllByLabelText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup] }, onchange },
    });
    await fireEvent.blur(getAllByLabelText('Context name')[0]!);
    await fireEvent.blur(getAllByLabelText('Context text')[0]!);
    expect(onchange).not.toHaveBeenCalled();
  });

  it('delete removes that preset and calls onchange with it filtered out, others unchanged', async () => {
    const onchange = vi.fn();
    const { getAllByLabelText } = render(ContextSection, {
      props: { cfg: { ...base, contexts: [standup, acme] }, onchange },
    });
    const deleteButtons = getAllByLabelText('Delete context');
    expect(deleteButtons).toHaveLength(2);
    await fireEvent.click(deleteButtons[0]!);

    expect(onchange).toHaveBeenCalledOnce();
    const next = onchange.mock.calls[0][0] as AppConfig;
    expect(next.contexts).toEqual([acme]);
  });

  it('a committed edit survives a follow-up Add (parent re-renders the committed cfg between events, as it does in the real Settings sheet)', async () => {
    let latest: AppConfig = { ...base, contexts: [standup] };
    const onchange = vi.fn((next: AppConfig) => { latest = next; });
    const { getAllByLabelText, getByText, rerender } = render(ContextSection, {
      props: { cfg: latest, onchange },
    });

    const nameInput = getAllByLabelText('Context name')[0] as HTMLInputElement;
    await fireEvent.input(nameInput, { target: { value: 'Daily Standup' } });
    // Clicking a different control shifts focus, so the browser fires blur on
    // the still-focused input (committing the edit) before the click handler
    // for that control runs. The parent applies the commit synchronously
    // (SettingsSheet's onChange sets its `cfg` state as its first statement),
    // so by the time Add's click handler runs, props already reflect it.
    await fireEvent.blur(nameInput);
    await rerender({ cfg: latest, onchange });

    await fireEvent.click(getByText(/Add context/i));

    expect(onchange).toHaveBeenCalledTimes(2);
    expect(latest.contexts).toHaveLength(2);
    expect(latest.contexts[0]!.name).toBe('Daily Standup');
    expect(latest.contexts[1]!.name).toBe('');
  });
});
