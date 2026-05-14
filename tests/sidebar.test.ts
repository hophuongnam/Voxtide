import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
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
});
