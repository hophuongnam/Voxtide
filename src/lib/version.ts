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

export interface GithubRelease {
  draft?: boolean;
  prerelease?: boolean;
  tag_name?: string;
  assets?: { name?: string; browser_download_url?: string }[];
}

/**
 * Pick the highest-semver published release that ships an `.apk` and is strictly
 * newer than `current`, else null. GitHub's /releases list is NOT ordered by
 * version or publish date, and desktop `v*` releases (no Android asset) are
 * interleaved — so scan ALL of them and take the max; never trust list position.
 */
export function pickUpdate(
  releases: GithubRelease[],
  current: string,
): { version: string; url: string } | null {
  let bestVer: string | null = null;
  let bestUrl: string | null = null;
  for (const rel of releases) {
    if (rel.draft || rel.prerelease) continue;
    const apk = (rel.assets ?? []).find((a) => String(a.name ?? '').endsWith('.apk'));
    if (!apk?.browser_download_url) continue;
    // Strip any tag prefix (v / android-v) down to the bare semver.
    const version = String(rel.tag_name ?? '').replace(/^[^\d]*/, '');
    if (version && (!bestVer || isNewer(version, bestVer))) {
      bestVer = version;
      bestUrl = apk.browser_download_url;
    }
  }
  return bestVer && bestUrl && isNewer(bestVer, current) ? { version: bestVer, url: bestUrl } : null;
}
