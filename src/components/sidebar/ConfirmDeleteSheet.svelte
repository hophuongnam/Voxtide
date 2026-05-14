<script lang="ts">
  import { formatDuration, formatTime } from '../../lib/format';
  import type { SessionRow } from '../../types';

  interface Props {
    open: boolean;
    target: SessionRow | null;
    busy: boolean;
    error: string | null;
    onconfirm: () => void;
    oncancel: () => void;
  }
  const { open, target, busy, error, onconfirm, oncancel }: Props = $props();

  const subtitle = $derived(target
    ? `${formatTime(target.started_at)} · ${formatDuration(target.duration_ms ?? 0)} · ${target.lang_a.toUpperCase()} → ${target.lang_b.toUpperCase()}`
    : '');

  function onOverlay() { if (!busy) oncancel(); }
  function onKey(e: KeyboardEvent) { if (e.key === 'Escape' && !busy) oncancel(); }
</script>

{#if open && target}
  <div role="presentation"
       class="fixed inset-0 z-50 flex items-center justify-center"
       style:background="rgba(0,0,0,0.45)"
       onclick={onOverlay}
       onkeydown={onKey}>
    <div role="dialog" aria-modal="true" tabindex="-1"
         class="rounded-xl w-[420px] p-6"
         style:background="var(--vt-bg)"
         style:border="0.5px solid var(--vt-border)"
         style:box-shadow="var(--vt-window-shadow)"
         onclick={(e) => e.stopPropagation()}
         onkeydown={(e) => e.stopPropagation()}>
      <div class="text-[14px] font-semibold mb-2" style:color="var(--vt-text)">
        Delete this transcript?
      </div>
      <div class="text-[12px] mb-1.5" style:color="var(--vt-muted)"
           style:font-family="'Geist Mono Variable', monospace">
        {subtitle}
      </div>
      <div class="text-[12px] mb-5" style:color="var(--vt-subtle)">
        This cannot be undone.
      </div>
      <div class="flex items-center justify-end gap-2">
        <button type="button"
                disabled={busy}
                onclick={oncancel}
                class="px-3 py-1.5 rounded-md text-[12px] cursor-pointer bg-transparent"
                style:color="var(--vt-muted)"
                style:border="0.5px solid var(--vt-border)"
                style:opacity={busy ? 0.5 : 1}>
          Cancel
        </button>
        <button type="button"
                disabled={busy}
                onclick={onconfirm}
                class="px-3 py-1.5 rounded-md text-[12px] font-semibold cursor-pointer border-0"
                style:background="var(--vt-danger)"
                style:color="var(--vt-danger-ink)"
                style:opacity={busy ? 0.6 : 1}>
          {busy ? 'Deleting…' : 'Delete'}
        </button>
      </div>
      {#if error}
        <div class="mt-3 text-[11px]" style:color="var(--vt-danger)">{error}</div>
      {/if}
    </div>
  </div>
{/if}
