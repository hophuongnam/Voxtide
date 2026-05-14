<script lang="ts">
  import { speakerVar } from '../../lib/speakerColor';
  interface Props { letter: string; lang?: string | null; name?: string | null; }
  const { letter, lang, name }: Props = $props();
</script>

<div class="inline-flex items-center gap-1.5 mb-[5px]">
  <div class="w-4 h-4 rounded-[4px] inline-flex items-center justify-center text-[9.5px] font-bold"
       style:background={speakerVar(letter)} style:color="var(--vt-speaker-ink)">{letter}</div>
  {#if name}
    <span class="text-[11px] font-medium" style:color="var(--vt-muted)">{name}</span>
  {/if}
  {#if lang}
    <span class="text-[9.5px] tracking-wide"
          style:color="var(--vt-subtle)" style:font-family="'Geist Mono Variable', monospace">{lang}</span>
  {/if}
</div>
