import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import RubyText from '../src/components/transcript/RubyText.svelte';

describe('RubyText', () => {
  it('wraps each Han char in <ruby> with an <rt> reading', () => {
    const { container } = render(RubyText, { props: { text: '你好' } });
    const rubies = container.querySelectorAll('ruby');
    expect(rubies.length).toBe(2);
    expect(rubies[0]!.querySelector('rt')!.textContent).toBe('nǐ');
    expect(rubies[0]!.textContent).toContain('你');
  });

  it('renders non-Han input as plain text with no ruby', () => {
    const { container } = render(RubyText, { props: { text: 'Hello' } });
    expect(container.querySelectorAll('ruby').length).toBe(0);
    expect(container.textContent).toBe('Hello');
  });

  it('mixes plain and ruby for mixed input', () => {
    const { container } = render(RubyText, { props: { text: 'Hi世界' } });
    expect(container.querySelectorAll('ruby').length).toBe(2);
    expect(container.textContent).toContain('Hi');
  });

  it('renders nothing for empty text', () => {
    const { container } = render(RubyText, { props: { text: '' } });
    expect(container.querySelectorAll('ruby').length).toBe(0);
  });
});
