<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  interface Props { recording: boolean; onclick: () => void; }
  const { recording, onclick }: Props = $props();
</script>

<button type="button" {onclick}
  class="inline-flex items-center gap-[7px] pl-[10px] pr-3 py-[6px] rounded-md cursor-pointer font-semibold text-xs"
  style:background={recording ? 'var(--vt-rec)' : 'var(--vt-accent)'}
  style:color={recording ? '#fff' : 'var(--vt-accent-ink)'}
  style:box-shadow="0 1px 0 var(--vt-btn-inner-hi) inset, 0 6px 14px -6px rgba(0,0,0,0.35)">
  {#if recording}
    <span class="block w-2 h-2 rounded-sm" style:background="#fff"></span>
    Stop
  {:else}
    <Icon name="play" size={11} />
    Start
  {/if}
</button>
