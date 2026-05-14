<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  import type { Mode } from '../../types';
  import type { DeviceEntry } from '../../lib/ipc';

  interface Props {
    mode: Mode;
    selected: DeviceEntry | null;
    options: DeviceEntry[];
    onselect: (d: DeviceEntry) => void;
  }
  const { mode, selected, options, onselect }: Props = $props();
  let open = $state(false);
  const iconName = mode === 'meeting' ? 'speaker' : 'mic';
</script>

<div class="relative inline-block">
  <button type="button"
    class="inline-flex items-center gap-2 px-[10px] py-[5px] rounded-md cursor-pointer text-xs max-w-[200px]"
    style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)"
    onclick={() => open = !open}>
    <Icon name={iconName} size={13} color="var(--vt-muted)" />
    <span class="whitespace-nowrap overflow-hidden text-ellipsis">{selected?.label ?? '—'}</span>
    <Icon name="chevron" size={11} color="var(--vt-muted)" />
  </button>
  {#if open}
    <div class="absolute right-0 mt-1 z-20 rounded-md min-w-[220px] py-1"
         style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
      {#each options as o}
        <button type="button"
          class="block w-full text-left px-3 py-1.5 text-xs cursor-pointer"
          style:color="var(--vt-text)"
          onclick={() => { onselect(o); open = false; }}>{o.label}{o.default ? ' (default)' : ''}</button>
      {/each}
    </div>
  {/if}
</div>
