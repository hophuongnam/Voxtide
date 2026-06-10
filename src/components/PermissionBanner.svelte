<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  interface Props { kind: 'mic' | 'audio-capture' | null; ondismiss: () => void; }
  const { kind, ondismiss }: Props = $props();

  const macUrls: Record<'mic' | 'audio-capture', string> = {
    'mic':            'x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone',
    'audio-capture':  'x-apple.systempreferences:com.apple.preference.security?Privacy_Audio',
  };
  const titles: Record<'mic' | 'audio-capture', string> = {
    'mic':           'Microphone access is required',
    'audio-capture': 'Audio capture access is required',
  };
  const bodies: Record<'mic' | 'audio-capture', string> = {
    'mic':           'macOS hasn’t granted Voxtide access to your microphone. Open System Settings → Privacy & Security → Microphone and toggle Voxtide on.',
    'audio-capture': 'macOS hasn’t granted Voxtide permission to capture system audio. Open System Settings → Privacy & Security → System Recording (Audio) and toggle Voxtide on.',
  };
</script>

{#if kind}
  <div data-testid="permission-banner" class="px-4 py-3 flex items-center gap-3"
       style:background="var(--vt-warn-tint)" style:border-bottom="0.5px solid var(--vt-warn-border)">
    <div class="flex-1">
      <div class="text-[12px] font-semibold" style:color="var(--vt-text)">{titles[kind]}</div>
      <div class="text-[11.5px] mt-0.5" style:color="var(--vt-muted)">{bodies[kind]}</div>
    </div>
    <button type="button"
            onclick={() => openUrl(macUrls[kind]).catch((e) => console.error('open settings failed', e))}
            class="px-3 py-1.5 rounded text-[11.5px] cursor-pointer font-semibold"
            style:background="var(--vt-warn)" style:color="var(--vt-bg)" style:border="none">
      Open System Settings
    </button>
    <button type="button" onclick={ondismiss}
            class="text-[11.5px] bg-transparent border-0 cursor-pointer px-2 py-1.5"
            style:color="var(--vt-muted)">Dismiss</button>
  </div>
{/if}
