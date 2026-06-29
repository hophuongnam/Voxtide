/**
 * True if semver string `a` is strictly newer than `b`, compared numerically
 * per component so 0.1.10 > 0.1.9 (a lexical string compare gets that wrong).
 * Tolerates a leading non-digit (e.g. a "v" git-tag prefix) and missing parts.
 */
export function isNewer(a: string, b: string): boolean {
  const parts = (v: string) => v.replace(/^[^\d]*/, '').split('.').map((n) => parseInt(n, 10) || 0);
  const pa = parts(a);
  const pb = parts(b);
  for (let i = 0; i < 3; i++) {
    const x = pa[i] || 0;
    const y = pb[i] || 0;
    if (x !== y) return x > y;
  }
  return false;
}
