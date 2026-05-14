import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import OverlayWindow from '../src/components/overlay/OverlayWindow.svelte';

describe('OverlayWindow', () => {
  const base = {
    lines: [
      'Được rồi, hãy bắt đầu.',
      'Chúng tôi đang đi trước tiến độ.',
      'Độ trễ tổng từ đầu đến cuối hiện đang khoảng hai trăm sáu mươi mili-giây.',
      'Con số đó thấp hơn nhiều so với mục tiêu dưới một giây.',
      'Cột mốc tiếp theo là phát hành bản nguyên mẫu vào cuối tuần.',
    ],
    state: 'active' as const,
    connectionLabel: 'EN → VI',
    onclose: () => {},
  };

  it('renders all 5 lines when active', () => {
    const { container } = render(OverlayWindow, { props: { ...base, hover: false } });
    const text = container.textContent ?? '';
    for (const l of base.lines) expect(text).toContain(l);
  });

  it('reveals the hover strip when hover is true', () => {
    const { container } = render(OverlayWindow, { props: { ...base, hover: true } });
    expect(container.querySelector('[data-strip="visible"]')).toBeTruthy();
  });

  it('shows reconnecting copy when state=reconnecting', () => {
    const { getByText } = render(OverlayWindow, {
      props: { ...base, state: 'reconnecting', hover: false, attempt: 2, retryInMs: 1000 },
    });
    expect(getByText(/Đang kết nối lại/)).toBeTruthy();
  });
});
