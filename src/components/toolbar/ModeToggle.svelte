<script lang="ts">
  import type { Mode } from '../../types';
  interface Props { mode: Mode; oninput: (m: Mode) => void; }
  const { mode, oninput }: Props = $props();
  const options: { id: Mode; label: string }[] = [
    { id: 'meeting', label: 'Meeting' },
    { id: 'conversation', label: 'Conversation' },
  ];
</script>

<div class="inline-flex p-[2px] rounded-lg"
     style:background="var(--vt-surface)"
     style:border="0.5px solid var(--vt-border)">
  {#each options as o}
    {@const active = o.id === mode}
    <button
      class="px-3 py-[5px] rounded-md text-xs font-medium border-0 cursor-pointer transition-colors"
      style:background={active ? 'var(--vt-surface3)' : 'transparent'}
      style:color={active ? 'var(--vt-text)' : 'var(--vt-muted)'}
      style:box-shadow={active ? '0 0 0 0.5px var(--vt-border-hi), 0 1px 0 var(--vt-btn-inner-hi) inset' : ''}
      aria-pressed={active}
      onclick={() => oninput(o.id)}
    >{o.label}</button>
  {/each}
</div>
