<script lang="ts">
  interface Props {
    version: string | null;
    busy: boolean;
    progress: number | null;
    error: string | null;
    oninstall: () => void;
    ondismiss: () => void;
  }
  const { version, busy, progress, error, oninstall, ondismiss }: Props = $props();
</script>

{#if version || error}
  <div class="px-4 py-3 flex items-center gap-3"
       style:background={error ? 'var(--vt-warn-tint)' : 'var(--vt-accent-tint-10)'}
       style:border-bottom={error
         ? '0.5px solid var(--vt-warn-border)'
         : '0.5px solid var(--vt-accent-tint-25)'}>
    <div class="flex-1">
      {#if error}
        <div class="text-[12px] font-semibold" style:color="var(--vt-text)">Update failed</div>
        <div class="text-[11.5px] mt-0.5" style:color="var(--vt-muted)">{error}</div>
      {:else if busy}
        <div class="text-[12px] font-semibold" style:color="var(--vt-text)">Installing Voxtide {version}…</div>
        <div class="text-[11.5px] mt-0.5" style:color="var(--vt-muted)">
          {progress !== null ? `${progress}% downloaded — the app will relaunch when ready.` : 'Downloading…'}
        </div>
      {:else}
        <div class="text-[12px] font-semibold" style:color="var(--vt-text)">Voxtide {version} is available</div>
        <div class="text-[11.5px] mt-0.5" style:color="var(--vt-muted)">
          Install now and Voxtide will relaunch with the new version.
        </div>
      {/if}
    </div>
    {#if !busy && !error}
      <button type="button" onclick={oninstall}
              class="px-3 py-1.5 rounded text-[11.5px] cursor-pointer font-semibold"
              style:background="var(--vt-accent)" style:color="var(--vt-bg)" style:border="none">
        Install &amp; restart
      </button>
    {/if}
    <button type="button" onclick={ondismiss}
            class="text-[11.5px] bg-transparent border-0 cursor-pointer px-2 py-1.5"
            style:color="var(--vt-muted)"
            disabled={busy}>Dismiss</button>
  </div>
{/if}
