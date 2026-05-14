<script lang="ts">
  import SearchBox from './SearchBox.svelte';
  import SessionItem from './SessionItem.svelte';
  import Icon from '../icons/Icon.svelte';
  import { groupByDate } from '../../lib/format';
  import type { SessionRow } from '../../types';

  interface Props {
    sessions: SessionRow[];
    activeId: number | null;
    onselect: (id: number) => void;
    onsearch: (q: string) => void;
    query: string;
    previews?: Record<number, string>;
    ondeleterequest?: (row: SessionRow) => void;
  }
  const { sessions, activeId, onselect, onsearch, query, previews = {}, ondeleterequest }: Props = $props();
  const groups = $derived(groupByDate(sessions, s => s.started_at));
</script>

<aside class="flex flex-col w-60 flex-shrink-0"
       style:border-right="0.5px solid var(--vt-border)"
       style:background="var(--vt-bg-deep)">
  <div class="p-3 pb-2"><SearchBox value={query} oninput={onsearch} /></div>
  <div class="px-3.5 pb-1.5 flex items-center justify-between">
    <span class="text-[10px] font-semibold uppercase tracking-wide"
          style:color="var(--vt-subtle)">History</span>
    <button type="button" class="w-[18px] h-[18px] rounded bg-transparent border-0 cursor-pointer inline-flex items-center justify-center"
            style:color="var(--vt-subtle)" title="New session"><Icon name="plus" size={12} /></button>
  </div>
  <div class="flex-1 overflow-auto px-2 pb-2 relative">
    {#each groups as g}
      <div>
        <div class="sticky top-0 z-10 px-1.5 pt-1.5 pb-1 text-[9px] font-semibold uppercase tracking-wide"
             style:color="var(--vt-subtle)"
             style:background="linear-gradient(var(--vt-bg-deep) 70%, transparent 100%)"
             style:font-family="'Geist Mono Variable', monospace">{g.label}</div>
        {#each g.items as row}
          <SessionItem
            {row}
            active={row.id === activeId}
            preview={previews[row.id] ?? ''}
            onclick={() => onselect(row.id)}
            {...(ondeleterequest ? { ondelete: ondeleterequest } : {})} />
        {/each}
      </div>
    {/each}
  </div>
</aside>
