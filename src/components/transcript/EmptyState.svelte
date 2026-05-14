<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  import type { Mode } from '../../types';
  interface Props { mode: Mode; }
  const { mode }: Props = $props();
  const iconName = $derived(mode === 'meeting' ? 'speaker' : 'mic');
  const title = $derived(mode === 'meeting' ? 'Ready to translate a meeting' : 'Ready for a conversation');
  const body = $derived(mode === 'meeting'
    ? 'Voxtide will capture system audio from the selected source and translate the remote speaker into your language.'
    : 'Voxtide will capture your microphone and translate two-way between the chosen language pair.');
</script>

<div class="flex-1 flex flex-col items-center justify-center p-6 gap-3.5">
  <div class="relative w-16 h-16 rounded-2xl flex items-center justify-center"
       style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
    <Icon name={iconName} size={26} color="var(--vt-muted)" stroke={1.4} />
    <div class="absolute -inset-1.5 rounded-[18px]" style:border="0.5px dashed var(--vt-border)"></div>
  </div>
  <div class="text-[15px] font-semibold" style:color="var(--vt-text)">{title}</div>
  <div class="text-[12.5px] max-w-[380px] text-center leading-relaxed"
       style:color="var(--vt-muted)">{body}</div>
</div>
