<script lang="ts">
  import Icon from '../icons/Icon.svelte';

  interface Props {
    state: 'active' | 'reconnecting' | 'idle';
    label: string;
    visible: boolean;
    onclose: () => void;
  }
  const { state, label, visible, onclose }: Props = $props();

  const dotColor = $derived(state === 'active'
    ? 'var(--vt-rec)'
    : state === 'reconnecting'
      ? 'var(--vt-warn)'
      : 'var(--vt-dim)');
  const dotShadow = $derived(state === 'active' ? '0 0 0 3px var(--vt-rec-glow)' : '');
</script>

<div
  data-strip={visible ? 'visible' : 'hidden'}
  class="absolute top-0 left-0 right-0 h-6 flex items-center gap-2 px-3 z-30"
  style:background="linear-gradient(to bottom, var(--vt-bg-deep), transparent)"
  style:opacity={visible ? 1 : 0}
  style:transform={visible ? 'translateY(0)' : 'translateY(-4px)'}
  style:transition="opacity .12s, transform .12s"
  style:pointer-events={visible ? 'auto' : 'none'}>
  <span
    class="block w-[7px] h-[7px] rounded-full"
    style:background={dotColor}
    style:box-shadow={dotShadow}></span>
  <span
    class="text-[9px] tracking-wider"
    style:color="var(--vt-subtle)"
    style:font-family="'Geist Mono Variable', monospace">{label}</span>
  <div class="flex-1 h-6 cursor-grab" data-tauri-drag-region></div>
  <button
    type="button"
    onclick={onclose}
    title="Close overlay"
    class="w-[22px] h-[22px] rounded bg-transparent border-0 cursor-pointer inline-flex items-center justify-center"
    style:color="var(--vt-muted)"><Icon name="close" size={12} /></button>
</div>
