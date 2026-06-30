import { describe, it, expect } from 'vitest';
import { isNewer, pickUpdate, type GithubRelease } from '../src/lib/version';

describe('isNewer', () => {
  it('detects a newer patch', () => expect(isNewer('0.1.8', '0.1.7')).toBe(true));
  it('compares numerically, not lexically (0.1.10 > 0.1.9)', () => expect(isNewer('0.1.10', '0.1.9')).toBe(true));
  it('equal is not newer', () => expect(isNewer('0.1.7', '0.1.7')).toBe(false));
  it('older is not newer', () => expect(isNewer('0.1.6', '0.1.7')).toBe(false));
  it('tolerates a leading v tag', () => expect(isNewer('v0.2.0', '0.1.9')).toBe(true));
  it('minor outranks patch', () => expect(isNewer('0.2.0', '0.1.99')).toBe(true));
});

// Reproduces the real GitHub /releases ordering observed 2026-06-30 — NOT sorted
// by version or date, desktop v* (no apk) interleaved. The old break-after-first
// logic stopped at android-v0.1.9 and never saw 0.1.11; pickUpdate scans all.
const apk = (tag: string): GithubRelease => ({
  tag_name: tag,
  assets: [{ name: `Voxtide_${tag.replace(/^[^\d]*/, '')}_android_arm64.apk`, browser_download_url: `https://x/${tag}.apk` }],
});
const DESKTOP_NO_APK: GithubRelease = {
  tag_name: 'v0.1.10',
  assets: [{ name: 'Voxtide_0.1.10_x64.dmg', browser_download_url: 'https://x/d.dmg' }],
};
const RELEASES: GithubRelease[] = [
  DESKTOP_NO_APK, // interleaved desktop release, no apk
  apk('android-v0.1.9'),
  apk('android-v0.1.8'),
  apk('android-v0.1.11'),
  apk('android-v0.1.10'),
];

describe('pickUpdate', () => {
  it('picks the max-semver apk regardless of list order', () =>
    expect(pickUpdate(RELEASES, '0.1.9')).toEqual({ version: '0.1.11', url: 'https://x/android-v0.1.11.apk' }));
  it('returns null when current is the max', () => expect(pickUpdate(RELEASES, '0.1.11')).toBeNull());
  it('returns null when current is newer than any release', () => expect(pickUpdate(RELEASES, '0.2.0')).toBeNull());
  it('ignores desktop releases that ship no apk', () =>
    expect(pickUpdate([DESKTOP_NO_APK], '0.0.1')).toBeNull());
  it('skips drafts and prereleases', () =>
    expect(pickUpdate([{ ...apk('android-v0.9.9'), draft: true }, { ...apk('android-v0.8.8'), prerelease: true }], '0.1.0')).toBeNull());
});
