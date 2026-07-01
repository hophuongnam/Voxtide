<script lang="ts">
  import Icon from '../icons/Icon.svelte';
  import type { ContextPreset } from '../../types';

  interface Props {
    contexts: ContextPreset[];
    activeId: string | null;
    disabled: boolean;
    onpick: (id: string | null) => void;
    onedit: () => void;
  }
  const { contexts, activeId, disabled, onpick, onedit }: Props = $props();
  let open = $state(false);

  // null covers both "no selection" and a dangling id (the active preset was
  // deleted) — both render as "No context".
  const activePreset = $derived(activeId === null ? null : contexts.find((c) => c.id === activeId) ?? null);
  const triggerLabel = $derived(activePreset ? activePreset.name || 'Untitled' : 'No context');

  function toggle() {
    if (disabled) return;
    open = !open;
  }
  function pick(id: string | null) {
    onpick(id);
    open = false;
  }
  function edit() {
    onedit();
    open = false;
  }
</script>

<div class="relative inline-block">
  <button type="button"
    class="inline-flex items-center gap-2 px-[10px] py-[5px] rounded-md text-xs max-w-[200px]"
    style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)" style:color="var(--vt-text)"
    style:opacity={disabled ? '0.4' : '1'} style:cursor={disabled ? 'default' : 'pointer'}
    {disabled} aria-disabled={disabled}
    onclick={toggle}>
    <span class="whitespace-nowrap overflow-hidden text-ellipsis">{triggerLabel}</span>
    <Icon name="chevron" size={11} color="var(--vt-muted)" />
  </button>
  {#if open}
    <div class="absolute right-0 mt-1 z-20 rounded-md min-w-[220px] py-1"
         style:background="var(--vt-surface)" style:border="0.5px solid var(--vt-border)">
      <button type="button"
        class="block w-full text-left px-3 py-1.5 text-xs cursor-pointer"
        style:color="var(--vt-text)"
        onclick={() => pick(null)}>No context</button>
      {#each contexts as preset (preset.id)}
        <button type="button"
          class="block w-full text-left px-3 py-1.5 text-xs cursor-pointer"
          style:color="var(--vt-text)"
          onclick={() => pick(preset.id)}>{preset.name || 'Untitled'}</button>
      {/each}
      <div class="my-1 h-px" style:background="var(--vt-border)"></div>
      <button type="button"
        class="block w-full text-left px-3 py-1.5 text-xs cursor-pointer"
        style:color="var(--vt-muted)"
        onclick={edit}>Edit contexts…</button>
    </div>
  {/if}
</div>
