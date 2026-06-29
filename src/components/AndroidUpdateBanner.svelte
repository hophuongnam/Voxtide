<script lang="ts">
  import { onMount } from 'svelte';
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { isNewer } from '../lib/version';

  // Android has no in-app updater (the desktop tauri-updater is cfg-gated off),
  // so this checks GitHub Releases and opens the APK in the browser to install.
  // A plain GET needs no auth/headers: api.github.com sends
  // Access-Control-Allow-Origin: * so the WebView fetch isn't CORS-blocked, and
  // 60 req/hr/IP is ample for one check per launch. Any failure (offline, rate
  // limit, bad JSON) stays silent — an update check must never break the app.
  const RELEASES_API = 'https://api.github.com/repos/hophuongnam/Voxtide/releases';

  let latest = $state<string | null>(null);
  let url = $state<string | null>(null);
  let dismissed = $state(false);

  onMount(async () => {
    // jsdom guard (matches the rest of the app): getVersion() needs the bridge.
    if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) return;
    try {
      const current = await getVersion();
      // List releases (newest first) and take the newest that ships an .apk —
      // /releases/latest can be a desktop-only release with no Android asset.
      const res = await fetch(`${RELEASES_API}?per_page=30`);
      if (!res.ok) return;
      const releases: Array<{
        draft?: boolean; prerelease?: boolean; tag_name?: string;
        assets?: { name?: string; browser_download_url?: string }[];
      }> = await res.json();
      for (const rel of releases) {
        if (rel.draft || rel.prerelease) continue;
        const apk = (rel.assets ?? []).find((a) => String(a.name ?? '').endsWith('.apk'));
        if (!apk) continue;
        // Strip any tag prefix (v / android-v) down to the bare semver.
        const version = String(rel.tag_name ?? '').replace(/^[^\d]*/, '');
        if (version && isNewer(version, current)) {
          latest = version;
          url = apk.browser_download_url ?? null;
        }
        break; // the first .apk-bearing release is the newest Android release
      }
    } catch (e) {
      console.warn('update check failed', e);
    }
  });
</script>

{#if latest && url && !dismissed}
  <div class="ub" data-testid="update-banner">
    <span class="ub-txt">Update available — v{latest}</span>
    <button class="ub-go" type="button"
            onclick={() => url && openUrl(url).catch((e) => console.error('open update failed', e))}>
      Update
    </button>
    <button class="ub-x" type="button" onclick={() => (dismissed = true)} aria-label="Dismiss">✕</button>
  </div>
{/if}

<style>
  .ub {
    display: flex; align-items: center; gap: 8px;
    padding: 8px 10px; margin-bottom: 8px; border-radius: 8px;
    background: var(--vt-warn-tint); border: 0.5px solid var(--vt-warn-border);
  }
  .ub-txt { flex: 1; font-size: 12.5px; font-weight: 600; color: var(--vt-text); }
  .ub-go {
    padding: 6px 12px; border-radius: 6px; border: none; font-size: 12px; font-weight: 600;
    background: var(--vt-warn); color: var(--vt-bg); cursor: pointer;
  }
  .ub-x { background: none; border: none; color: var(--vt-muted); font-size: 13px; padding: 4px 6px; cursor: pointer; }
</style>
