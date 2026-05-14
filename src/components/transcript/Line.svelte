<script lang="ts">
  import SpeakerChip from './SpeakerChip.svelte';
  import { formatTime } from '../../lib/format';
  import type { TranscriptLine } from '../../types';
  interface Props { line: TranscriptLine; translated?: boolean; }
  const { line, translated = false }: Props = $props();
  const ts = formatTime(line.ts_ms);
</script>

<div class="py-2.5 px-4 grid gap-2.5"
     style:grid-template-columns="52px 1fr"
     style:border-top="0.5px solid var(--vt-line-sep)">
  <div class="text-[10px] pt-[3px]"
       style:color="var(--vt-dim)" style:font-family="'Geist Mono Variable', monospace">{ts}</div>
  <div>
    {#if line.chip}<SpeakerChip letter={line.chip} lang={line.language?.toUpperCase() ?? null} />{/if}
    <div class="text-[13.5px] leading-relaxed"
         style:color={line.live ? 'var(--vt-muted)' : 'var(--vt-text)'}>
      {line.text}
      {#if line.live}
        <span class="inline-block w-2 h-[14px] ml-[2px] align-middle"
              style:background={translated ? 'var(--vt-accent)' : 'var(--vt-muted)'}
              style:animation="vt-blink 0.9s steps(2) infinite"></span>
      {/if}
    </div>
  </div>
</div>
