import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import Sidebar from '../src/components/sidebar/Sidebar.svelte';

describe('Sidebar', () => {
  const sessions = [
    { id: 1, started_at: Date.now(),                ended_at: null, mode: 'meeting',
      lang_a: 'en', lang_b: 'vi', device_label: 'Zoom', duration_ms: 38 * 60_000 },
    { id: 2, started_at: Date.now() - 25 * 3600_000, ended_at: null, mode: 'conversation',
      lang_a: 'en', lang_b: 'ja', device_label: null, duration_ms: 8 * 60_000 },
  ];

  it('renders date-group headers for non-empty buckets only', () => {
    const { getAllByText, queryByText } = render(Sidebar, {
      props: { sessions, activeId: 1, onselect: () => {}, onsearch: () => {}, query: '' },
    });
    expect(getAllByText('Today').length).toBeGreaterThan(0);
    expect(getAllByText('Yesterday').length).toBeGreaterThan(0);
    expect(queryByText('Earlier')).toBeNull();
  });

  it('marks the active session', () => {
    const { container } = render(Sidebar, {
      props: { sessions, activeId: 2, onselect: () => {}, onsearch: () => {}, query: '' },
    });
    expect(container.querySelector('[data-active="true"]')).toBeTruthy();
  });

  it('shows the rec dot and hides trash ONLY for the live session (by liveId, not ended_at)', () => {
    // The genuinely-recording session: ended_at is transiently null AND its id
    // equals liveId. It must show the dot and be undeletable.
    const live = [{ ...sessions[0]!, ended_at: null, duration_ms: null }];
    const { container } = render(Sidebar, {
      props: {
        sessions: live, activeId: live[0]!.id, liveId: live[0]!.id,
        onselect: () => {}, onsearch: () => {}, query: '',
        ondeleterequest: () => {},
      },
    });
    expect(container.querySelector('[data-testid="rec-dot"]')).toBeTruthy();
    expect(container.querySelector('[data-testid="delete-session"]')).toBeNull();
  });

  it('an orphan (ended_at null, NOT live) is deletable and shows no rec dot', async () => {
    // Exactly the screenshot bug: a session stuck ended_at=null after a
    // kill/quit. Nothing is recording (liveId null), so it must be a normal
    // deletable row with no "recording" indicator.
    const orphan = [{ ...sessions[0]!, ended_at: null, duration_ms: null }];
    const ondeleterequest = vi.fn();
    const { container } = render(Sidebar, {
      props: {
        sessions: orphan, activeId: orphan[0]!.id, liveId: null,
        onselect: () => {}, onsearch: () => {}, query: '',
        ondeleterequest,
      },
    });
    expect(container.querySelector('[data-testid="rec-dot"]')).toBeNull();
    const btn = container.querySelector('[data-testid="delete-session"]') as HTMLButtonElement;
    expect(btn).toBeTruthy();
    await fireEvent.click(btn);
    expect(ondeleterequest).toHaveBeenCalledWith(orphan[0]);
  });

  it('shows the trash button on past rows and routes clicks to ondeleterequest', async () => {
    const past = [{ ...sessions[0]!, ended_at: sessions[0]!.started_at + 60_000 }];
    const ondeleterequest = vi.fn();
    const { container } = render(Sidebar, {
      props: {
        sessions: past, activeId: past[0]!.id, liveId: null,
        onselect: () => {}, onsearch: () => {}, query: '',
        ondeleterequest,
      },
    });
    const btn = container.querySelector('[data-testid="delete-session"]') as HTMLButtonElement;
    expect(btn).toBeTruthy();
    await fireEvent.click(btn);
    expect(ondeleterequest).toHaveBeenCalledWith(past[0]);
  });
});
