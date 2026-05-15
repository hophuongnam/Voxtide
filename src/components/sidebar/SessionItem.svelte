<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  import type { Mode, SessionRow } from '../../types';
  import { MODE_LABELS } from '../../lib/modes';
  import { formatDuration, formatTime } from '../../lib/format';

  interface Props {
    row: SessionRow;
    active: boolean;
    /** True only when this row is the session recording *right now*. Drives
     *  the rec dot and the delete gate. NOT derived from `ended_at`: an
     *  orphaned row (killed/quit mid-record) also has ended_at == null but is
     *  not live, and must stay deletable. */
    live?: boolean;
    onclick: () => void;
    ondelete?: (row: SessionRow) => void;
    preview?: string;
  }
  const { row, active, live = false, onclick, ondelete, preview = '' }: Props = $props();
  const time = $derived(formatTime(row.started_at));
  const modeLabel = $derived(MODE_LABELS[row.mode as Mode] ?? row.mode);
  const dur = $derived(row.duration_ms ? formatDuration(row.duration_ms) : '—');
  const canDelete = $derived(!live);

  function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    ondelete?.(row);
  }
</script>

<div class="group relative">
  <button
    type="button"
    data-active={active}
    class="px-[9px] py-2 rounded-md mb-[2px] cursor-pointer w-full text-left"
    style:background={active ? 'var(--vt-surface2)' : 'transparent'}
    style:border={`0.5px solid ${active ? 'var(--vt-border)' : 'transparent'}`}
    style:outline="none"
    onclick={onclick}>
    <div class="flex items-center justify-between mb-1">
      <span class="text-[11px]" style:color="var(--vt-muted)">{time}</span>
      <span class="text-[10px] transition-opacity {canDelete && ondelete ? 'group-hover:opacity-0' : ''}"
            style:color="var(--vt-subtle)"
            style:font-family="'Geist Mono Variable', monospace">{dur}</span>
    </div>
    <div class="flex items-center gap-1.5 mb-1.5">
      <span class="px-1 rounded text-[10px] font-semibold tracking-wide"
            style:background="var(--vt-surface)" style:color="var(--vt-muted)"
            style:border="0.5px solid var(--vt-border)"
            style:font-family="'Geist Mono Variable', monospace">{row.lang_a.toUpperCase()}</span>
      <Icon name="arrow" size={10} color="var(--vt-subtle)" />
      <span class="px-1 rounded text-[10px] font-semibold tracking-wide"
            style:background="var(--vt-surface)" style:color="var(--vt-muted)"
            style:border="0.5px solid var(--vt-border)"
            style:font-family="'Geist Mono Variable', monospace">{row.lang_b.toUpperCase()}</span>
      <span class="ml-auto text-[9.5px] uppercase tracking-wide"
            style:color="var(--vt-subtle)">{modeLabel}</span>
      {#if live}
        <span data-testid="rec-dot" class="block w-1.5 h-1.5 rounded-full"
              style:background="var(--vt-rec)"
              style:box-shadow="0 0 0 2px var(--vt-rec-glow)"></span>
      {/if}
    </div>
    {#if preview}
      <div class="text-[11px] leading-snug overflow-hidden"
           style:color="var(--vt-muted)"
           style:display="-webkit-box"
           style:-webkit-line-clamp="2"
           style:-webkit-box-orient="vertical">{preview}</div>
    {/if}
  </button>
  {#if canDelete && ondelete}
    <button
      type="button"
      data-testid="delete-session"
      aria-label="Delete transcript"
      onclick={handleDelete}
      class="absolute top-1 right-1.5 w-7 h-7 rounded inline-flex items-center justify-center bg-transparent border-0 cursor-pointer opacity-0 group-hover:opacity-100 focus:opacity-100 transition-opacity"
      style:color="var(--vt-subtle)">
      <Icon name="trash" size={16} />
    </button>
  {/if}
</div>
