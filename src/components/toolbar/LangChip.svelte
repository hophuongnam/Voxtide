<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  interface Props { code: string; name: string; onclick?: () => void; }
  const { code, name, onclick }: Props = $props();
</script>

<button
  type="button" onclick={onclick}
  class="relative inline-flex items-center gap-2 pl-2 pr-[10px] py-[5px] rounded-md cursor-pointer text-xs"
  style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)">
  <span class="font-semibold tracking-wider px-1.5 py-[2px] rounded text-[10px]"
        style:background="var(--vt-surface3)" style:color="var(--vt-muted)" style:font-family="var(--vt-mono, 'Geist Mono Variable')">{code}</span>
  <span>{name}</span>
  <Icon name="chevron" size={11} color="var(--vt-muted)" />
</button>
