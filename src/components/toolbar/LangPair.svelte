<script lang="ts">
  import LangChip from './LangChip.svelte';
  import LangPicker from './LangPicker.svelte';
  import Icon from '../icons/Icon.svelte';

  type Slot = 'a' | 'b';
  interface Props {
    a: { code: string; name: string };
    b: { code: string; name: string };
    onswap?: () => void;
    onpick?: (which: Slot, code: string) => void;
  }
  const { a, b, onswap, onpick }: Props = $props();

  let open = $state<Slot | null>(null);

  function pick(which: Slot, code: string) {
    onpick?.(which, code);
    open = null;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape' && open !== null) open = null;
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="inline-flex items-center gap-[6px]">
  <div class="relative">
    <LangChip {...a} onclick={() => (open = open === 'a' ? null : 'a')} />
    {#if open === 'a'}
      <LangPicker current={a.code.toLowerCase()} disabledCode={b.code.toLowerCase()} onpick={(c) => pick('a', c)} />
    {/if}
  </div>
  <button
    type="button" aria-label="Swap languages" title="Swap source and target"
    class="w-[22px] h-[22px] rounded-md cursor-pointer inline-flex items-center justify-center transition-colors hover:bg-[var(--vt-surface3)]"
    style:color="var(--vt-subtle)" style:border="0.5px solid var(--vt-border)"
    onclick={onswap}>
    <Icon name="swap" size={14} stroke={1.4} />
  </button>
  <div class="relative">
    <LangChip {...b} onclick={() => (open = open === 'b' ? null : 'b')} />
    {#if open === 'b'}
      <LangPicker current={b.code.toLowerCase()} disabledCode={a.code.toLowerCase()} onpick={(c) => pick('b', c)} />
    {/if}
  </div>
</div>
