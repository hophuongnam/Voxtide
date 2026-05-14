<script lang="ts">
  import WaveGlyph from '../icons/WaveGlyph.svelte';
  import ModeToggle from './ModeToggle.svelte';
  import LangPair from './LangPair.svelte';
  import AudioSourcePicker from './AudioSourcePicker.svelte';
  import IconBtn from './IconBtn.svelte';
  import PrimaryBtn from './PrimaryBtn.svelte';
  import type { DeviceEntry } from '../../lib/ipc';
  import type { Mode, WhichLang } from '../../types';

  interface Props {
    mode: Mode;
    onmode: (m: Mode) => void;
    recording: boolean;
    onstart: () => void;
    onstop: () => void;
    onsettings: () => void;
    onoverlay: () => void;
    overlayShown: boolean;
    a: { code: string; name: string };
    b: { code: string; name: string };
    mine: WhichLang;
    onswap: () => void;
    onlangpick: (which: WhichLang, code: string) => void;
    source: DeviceEntry | null;
    sourceOptions: DeviceEntry[];
    onsource: (d: DeviceEntry) => void;
  }
  const p: Props = $props();
</script>

<div class="h-12 flex items-center gap-2.5 px-3"
     style:border-bottom="0.5px solid var(--vt-border)" style:background="var(--vt-bg)">
  <div class="flex items-center gap-[7px]">
    <div class="w-[18px] h-[18px] rounded-[5px] flex items-center justify-center"
         style:background="linear-gradient(135deg, var(--vt-accent), var(--vt-accent-dim))">
      <WaveGlyph size={11} color="var(--vt-accent-ink)" bars={5} />
    </div>
    <span class="text-[13px] font-semibold" style:color="var(--vt-text)">Voxtide</span>
  </div>
  <div class="w-px h-[18px] mx-1" style:background="var(--vt-border)"></div>
  <ModeToggle mode={p.mode} oninput={p.onmode} />
  <div class="ml-2"><LangPair a={p.a} b={p.b} mine={p.mine} onswap={p.onswap} onpick={p.onlangpick} /></div>
  <div class="ml-1.5">
    <AudioSourcePicker mode={p.mode} selected={p.source} options={p.sourceOptions} onselect={p.onsource} />
  </div>
  <div class="flex-1"></div>
  <IconBtn name="overlay" active={p.overlayShown} title="Show overlay" onclick={p.onoverlay} />
  <IconBtn name="cog" title="Settings" onclick={p.onsettings} />
  <PrimaryBtn recording={p.recording} onclick={p.recording ? p.onstop : p.onstart} />
</div>
