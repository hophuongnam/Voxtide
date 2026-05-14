<script lang="ts">
  import { LANG_NAMES, LANG_CODES } from '../../lib/languages';

  interface Props {
    current: string;
    disabledCode?: string;
    onpick: (code: string) => void;
  }
  const { current, disabledCode, onpick }: Props = $props();

  function click(code: string) {
    if (code === disabledCode) return;
    onpick(code);
  }
</script>

<div role="listbox" aria-label="Pick language"
     class="absolute top-full left-0 mt-1 z-20 py-1 rounded-md min-w-[170px]"
     style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
  {#each LANG_CODES as code}
    {@const isDisabled = code === disabledCode}
    <button type="button"
            role="option"
            aria-selected={code === current}
            aria-disabled={isDisabled}
            disabled={isDisabled}
            title={isDisabled ? 'Already used by the other language' : undefined}
            class="w-full text-left px-3 py-1.5 text-xs border-0 flex items-center gap-2"
            style:background={code === current ? 'var(--vt-surface3)' : 'transparent'}
            style:color="var(--vt-text)"
            style:opacity={isDisabled ? '0.4' : '1'}
            style:cursor={isDisabled ? 'not-allowed' : 'pointer'}
            onclick={() => click(code)}>
      <span class="font-semibold tracking-wider px-1.5 py-[1px] rounded text-[9px]"
            style:background="var(--vt-surface3)" style:color="var(--vt-muted)"
            style:font-family="var(--vt-mono, 'Geist Mono Variable')">{code.toUpperCase()}</span>
      <span>{LANG_NAMES[code]}</span>
    </button>
  {/each}
</div>
