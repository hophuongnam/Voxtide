import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import ConfirmDeleteSheet from '../src/components/sidebar/ConfirmDeleteSheet.svelte';
import type { SessionRow } from '../src/types';

const row: SessionRow = {
  id: 7,
  started_at: new Date('2026-05-14T10:30:00Z').getTime(),
  ended_at: new Date('2026-05-14T10:35:00Z').getTime(),
  mode: 'meeting',
  lang_a: 'en',
  lang_b: 'vi',
  device_label: null,
  duration_ms: 5 * 60_000,
};

describe('ConfirmDeleteSheet', () => {
  it('does not render when closed', () => {
    const { container } = render(ConfirmDeleteSheet, {
      props: {
        open: false, target: row, busy: false, error: null,
        onconfirm: () => {}, oncancel: () => {},
      },
    });
    expect(container.querySelector('[role="dialog"]')).toBeNull();
  });

  it('renders the row metadata when open', () => {
    const { getByText } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false, error: null,
        onconfirm: () => {}, oncancel: () => {},
      },
    });
    expect(getByText(/en/i)).toBeTruthy();
    expect(getByText(/vi/i)).toBeTruthy();
    expect(getByText(/cannot be undone/i)).toBeTruthy();
  });

  it('fires oncancel on Cancel click', async () => {
    const oncancel = vi.fn();
    const { getByText } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false, error: null,
        onconfirm: () => {}, oncancel,
      },
    });
    await fireEvent.click(getByText('Cancel'));
    expect(oncancel).toHaveBeenCalledOnce();
  });

  it('fires onconfirm on Delete click', async () => {
    const onconfirm = vi.fn();
    const { getByText } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false, error: null,
        onconfirm, oncancel: () => {},
      },
    });
    await fireEvent.click(getByText('Delete'));
    expect(onconfirm).toHaveBeenCalledOnce();
  });

  it('Esc fires oncancel when not busy', async () => {
    const oncancel = vi.fn();
    const { container } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false, error: null,
        onconfirm: () => {}, oncancel,
      },
    });
    const overlay = container.querySelector('[role="presentation"]')!;
    await fireEvent.keyDown(overlay, { key: 'Escape' });
    expect(oncancel).toHaveBeenCalledOnce();
  });

  it('busy=true disables buttons and ignores Esc/overlay click', async () => {
    const oncancel = vi.fn();
    const onconfirm = vi.fn();
    const { container, getByText } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: true, error: null,
        onconfirm, oncancel,
      },
    });
    const cancel = getByText('Cancel') as HTMLButtonElement;
    const del = getByText('Deleting…') as HTMLButtonElement;
    expect(cancel.disabled).toBe(true);
    expect(del.disabled).toBe(true);
    const overlay = container.querySelector('[role="presentation"]')!;
    await fireEvent.click(overlay);
    await fireEvent.keyDown(overlay, { key: 'Escape' });
    expect(oncancel).not.toHaveBeenCalled();
    expect(onconfirm).not.toHaveBeenCalled();
  });

  it('Esc from inside the dialog also fires oncancel', async () => {
    const oncancel = vi.fn();
    const { container } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false, error: null,
        onconfirm: () => {}, oncancel,
      },
    });
    const dialog = container.querySelector('[role="dialog"]')!;
    await fireEvent.keyDown(dialog, { key: 'Escape', bubbles: true });
    expect(oncancel).toHaveBeenCalledOnce();
  });

  it('renders error text when error is set', () => {
    const { getByText } = render(ConfirmDeleteSheet, {
      props: {
        open: true, target: row, busy: false,
        error: 'cannot delete an active session',
        onconfirm: () => {}, oncancel: () => {},
      },
    });
    expect(getByText(/cannot delete an active session/)).toBeTruthy();
  });
});
