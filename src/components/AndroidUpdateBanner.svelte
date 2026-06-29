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
  const RELEASES_API = 'https://api.github.com/repos/hophuongnam/Voxtide/releases/latest';

  let latest = $state<string | null>(null);
  let url = $state<string | null>(null);
  let dismissed = $state(false);

  onMount(async () => {
    // jsdom guard (matches the rest of the app): getVersion() needs the bridge.
    if (typeof window === 'undefined' || !(window as any).__TAURI_INTERNALS__) return;
    try {
      const current = await getVersion();
      const res = await fetch(RELEASES_API);
      if (!res.ok) return;
      const rel = await res.json();
      const tag = String(rel.tag_name ?? '').replace(/^v/, '');
      if (!tag || !isNewer(tag, current)) return;
      // Prefer the .apk asset; fall back to the release page if it isn't attached.
      const apk = (rel.assets ?? []).find((a: { name?: string }) => String(a.name ?? '').endsWith('.apk'));
      url = apk?.browser_download_url ?? rel.html_url ?? null;
      if (url) latest = tag;
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
