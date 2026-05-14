<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  interface Props { value: string; oninput: (v: string) => void; }
  const { value, oninput }: Props = $props();
</script>

<div class="flex items-center gap-[7px] px-[9px] py-1.5 rounded-md"
     style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
  <Icon name="search" size={13} color="var(--vt-subtle)" />
  <input
    type="search"
    {value}
    placeholder="Search transcripts…"
    oninput={(e) => oninput((e.target as HTMLInputElement).value)}
    class="flex-1 bg-transparent border-0 outline-none text-xs min-w-0"
    style:color="var(--vt-text)" />
  <span class="px-1 rounded text-[10px]"
        style:color="var(--vt-subtle)" style:border="0.5px solid var(--vt-border)"
        style:font-family="'Geist Mono Variable', monospace">⌘K</span>
</div>
