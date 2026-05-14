import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import Icon from '../src/components/icons/Icon.svelte';
import WaveGlyph from '../src/components/icons/WaveGlyph.svelte';
import { PATHS } from '../src/components/icons/paths';

describe('Icon', () => {
  it('renders a path attribute matching the named path', () => {
    const { container } = render(Icon, { props: { name: 'mic', size: 16 } });
    const path = container.querySelector('path');
    expect(path?.getAttribute('d')).toBe(PATHS.mic);
    const svg = container.querySelector('svg');
    expect(svg?.getAttribute('width')).toBe('16');
  });
});

describe('WaveGlyph', () => {
  it('renders 5 bars by default', () => {
    const { container } = render(WaveGlyph, { props: { size: 32, color: 'red' } });
    expect(container.querySelectorAll('rect').length).toBe(5);
  });
  it('renders 3 bars when bars=3', () => {
    const { container } = render(WaveGlyph, { props: { size: 32, color: 'red', bars: 3 } });
    expect(container.querySelectorAll('rect').length).toBe(3);
  });
});
