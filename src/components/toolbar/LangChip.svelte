<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  interface Props { code: string; name: string; mine?: boolean; onclick?: () => void; }
  const { code, name, mine = false, onclick }: Props = $props();
</script>

<button
  type="button" onclick={onclick}
  class="relative inline-flex items-center gap-2 pl-2 pr-[10px] py-[5px] rounded-md cursor-pointer text-xs"
  style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)">
  <span class="font-semibold tracking-wider px-1.5 py-[2px] rounded text-[10px]"
        style:background="var(--vt-surface3)" style:color="var(--vt-muted)" style:font-family="var(--vt-mono, 'Geist Mono Variable')">{code}</span>
  <span>{name}</span>
  {#if mine}
    <span class="absolute -top-1.5 -right-1.5 px-[4px] py-[1px] rounded text-[8.5px] font-bold tracking-wide"
          style:background="var(--vt-accent)" style:color="var(--vt-accent-ink)">YOU</span>
  {/if}
  <Icon name="chevron" size={11} color="var(--vt-muted)" />
</button>
