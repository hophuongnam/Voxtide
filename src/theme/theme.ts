import type { Theme } from '../types';

const DARK = 'vt-theme-dark';
const LIGHT = 'vt-theme-light';

export type ResolvedTheme = 'dark' | 'light';

function systemResolved(): ResolvedTheme {
  if (typeof window === 'undefined') return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(theme: Theme) {
  const resolved: ResolvedTheme = theme === 'system' ? systemResolved() : theme;
  const body = document.body;
  body.classList.remove(DARK, LIGHT);
  body.classList.add(resolved === 'dark' ? DARK : LIGHT);
}

export function currentResolvedTheme(): ResolvedTheme {
  return document.body.classList.contains(DARK) ? 'dark' : 'light';
}
