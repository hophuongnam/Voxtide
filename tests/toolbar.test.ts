import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ModeToggle from '../src/components/toolbar/ModeToggle.svelte';
import PrimaryBtn from '../src/components/toolbar/PrimaryBtn.svelte';

describe('ModeToggle', () => {
  it('marks the active mode with aria-pressed', () => {
    const { getByText } = render(ModeToggle, { props: { mode: 'meeting', oninput: () => {} } });
    expect(getByText('Meeting').getAttribute('aria-pressed')).toBe('true');
    expect(getByText('Conversation').getAttribute('aria-pressed')).toBe('false');
  });
  it('fires oninput when switching', async () => {
    const fn = vi.fn();
    const { getByText } = render(ModeToggle, { props: { mode: 'meeting', oninput: fn } });
    await fireEvent.click(getByText('Conversation'));
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
