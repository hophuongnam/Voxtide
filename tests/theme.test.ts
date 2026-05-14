import { describe, it, expect, beforeEach } from 'vitest';
import { applyTheme, currentResolvedTheme } from '../src/theme/theme';

describe('theme', () => {
  beforeEach(() => {
    document.documentElement.className = '';
    document.body.className = '';
  });

  it('applies dark class to body when set to dark', () => {
    applyTheme('dark');
    expect(document.body.classList.contains('vt-theme-dark')).toBe(true);
    expect(currentResolvedTheme()).toBe('dark');
  });

  it('applies light class when set to light', () => {
    applyTheme('light');
    expect(document.body.classList.contains('vt-theme-light')).toBe(true);
  });

  it('system mode resolves to dark when matchMedia says dark', () => {
    (window.matchMedia as any) = (q: string) => ({ matches: q.includes('dark') });
    applyTheme('system');
    expect(document.body.classList.contains('vt-theme-dark')).toBe(true);
  });
});
