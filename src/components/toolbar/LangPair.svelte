<script lang="ts">
  import LangChip from './LangChip.svelte';
  import LangPicker from './LangPicker.svelte';
  import Icon from '../icons/Icon.svelte';
  import type { WhichLang } from '../../types';

  interface Props {
    a: { code: string; name: string };
    b: { code: string; name: string };
    mine: WhichLang;
    onswap?: () => void;
    onpick?: (which: WhichLang, code: string) => void;
  }
  const { a, b, mine, onswap, onpick }: Props = $props();

  let open = $state<WhichLang | null>(null);

  function pick(which: WhichLang, code: string) {
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
    <LangChip {...a} mine={mine === 'a'} onclick={() => (open = open === 'a' ? null : 'a')} />
    {#if open === 'a'}
      <LangPicker current={a.code.toLowerCase()} disabledCode={b.code.toLowerCase()} onpick={(c) => pick('a', c)} />
    {/if}
  </div>
  <button
    type="button" aria-label="Swap languages"
    class="w-[22px] h-[22px] rounded-md bg-transparent border-0 cursor-pointer inline-flex items-center justify-center"
    style:color="var(--vt-subtle)"
    onclick={onswap}>
    <Icon name="swap" size={14} stroke={1.4} />
  </button>
  <div class="relative">
    <LangChip {...b} mine={mine === 'b'} onclick={() => (open = open === 'b' ? null : 'b')} />
    {#if open === 'b'}
      <LangPicker current={b.code.toLowerCase()} disabledCode={a.code.toLowerCase()} onpick={(c) => pick('b', c)} />
    {/if}
  </div>
</div>
