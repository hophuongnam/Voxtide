<script lang="ts">
  import LangChip from './LangChip.svelte';
  import Icon from '../icons/Icon.svelte';
  import type { WhichLang } from '../../types';
  interface Props {
    a: { code: string; name: string };
    b: { code: string; name: string };
    mine: WhichLang;
    onswap?: () => void;
  }
  const { a, b, mine, onswap }: Props = $props();
</script>

<div class="inline-flex items-center gap-[6px]">
  <LangChip {...a} mine={mine === 'a'} />
  <button
    type="button" aria-label="Swap languages"
    class="w-[22px] h-[22px] rounded-md bg-transparent border-0 cursor-pointer inline-flex items-center justify-center"
    style:color="var(--vt-subtle)"
    onclick={onswap}>
    <Icon name="swap" size={14} stroke={1.4} />
  </button>
  <LangChip {...b} mine={mine === 'b'} />
</div>
