import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ModeToggle from '../src/components/toolbar/ModeToggle.svelte';
import PrimaryBtn from '../src/components/toolbar/PrimaryBtn.svelte';
import LangPair from '../src/components/toolbar/LangPair.svelte';

describe('ModeToggle', () => {
  it('marks the active mode with aria-pressed', () => {
    const { getByText } = render(ModeToggle, { props: { mode: 'meeting', oninput: () => {} } });
    expect(getByText('System Audio').getAttribute('aria-pressed')).toBe('true');
    expect(getByText('Microphone').getAttribute('aria-pressed')).toBe('false');
  });
  it('fires oninput when switching', async () => {
    const fn = vi.fn();
    const { getByText } = render(ModeToggle, { props: { mode: 'meeting', oninput: fn } });
    await fireEvent.click(getByText('Microphone'));
    expect(fn).toHaveBeenCalledWith('conversation');
  });
});

describe('PrimaryBtn', () => {
  it('labels Start vs Stop based on recording', () => {
    const { getByRole, rerender } = render(PrimaryBtn, { props: { recording: false, onclick: () => {} } });
    expect(getByRole('button').textContent?.trim()).toBe('Start');
    rerender({ recording: true, onclick: () => {} });
    expect(getByRole('button').textContent?.trim()).toBe('Stop');
  });
});

describe('LangPair picker', () => {
  const baseProps = {
    a: { code: 'EN', name: 'English' },
    b: { code: 'VI', name: 'Vietnamese' },
  };

  it('clicking chip A opens a picker listbox with 8 options', async () => {
    const { getByText, queryByRole, getByRole } = render(LangPair, { props: { ...baseProps, onswap: () => {}, onpick: () => {} } });
    expect(queryByRole('listbox')).toBeNull();
    await fireEvent.click(getByText('English'));
    const listbox = getByRole('listbox', { name: 'Pick language' });
    expect(listbox.querySelectorAll('[role="option"]').length).toBe(8);
  });

  it('picking a code in chip A fires onpick("a", <code>) and closes the picker', async () => {
    const onpick = vi.fn();
    const { getByText, queryByRole } = render(LangPair, { props: { ...baseProps, onswap: () => {}, onpick } });
    await fireEvent.click(getByText('English'));            // open
    await fireEvent.click(getByText('Japanese'));           // pick
    expect(onpick).toHaveBeenCalledWith('a', 'ja');
    expect(queryByRole('listbox')).toBeNull();
  });

  it('clicking chip B opens a separate picker and selects against b.code', async () => {
    const onpick = vi.fn();
    const { getByText } = render(LangPair, { props: { ...baseProps, onswap: () => {}, onpick } });
    await fireEvent.click(getByText('Vietnamese'));
    await fireEvent.click(getByText('Korean'));
    expect(onpick).toHaveBeenCalledWith('b', 'ko');
  });
});
