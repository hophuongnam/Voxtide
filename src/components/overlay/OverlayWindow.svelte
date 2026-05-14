<script lang="ts">
  import HoverStrip from './HoverStrip.svelte';

  interface Props {
    lines: string[];
    state: 'active' | 'reconnecting' | 'idle';
    connectionLabel: string;
    hover: boolean;
    onclose: () => void;
    attempt?: number;
    retryInMs?: number;
  }
  const p: Props = $props();

  const opacityFor = (i: number, n: number) =>
    n === 1 ? 1 : 0.35 + (0.65 * i) / (n - 1);
</script>

<div
  class="relative rounded-[20px] overflow-hidden"
  style:width="100%" style:height="100%"
  style:background="var(--vt-overlay-bg)"
  style:backdrop-filter="blur(40px) saturate(160%)"
  style:-webkit-backdrop-filter="blur(40px) saturate(160%)"
  style:border="0.5px solid var(--vt-overlay-border)"
  style:box-shadow="var(--vt-overlay-shadow)"
  style:color="var(--vt-text)"
  style:font-family="'Geist Variable', system-ui, sans-serif">

  <HoverStrip
    state={p.state}
    label={p.state === 'reconnecting' ? 'RECONNECTING'
         : p.state === 'idle' ? 'IDLE'
         : p.connectionLabel}
    visible={p.hover}
    onclose={p.onclose} />

  <div
    class="absolute inset-0 flex flex-col justify-end gap-0.5"
    style:padding="14px 18px 16px">
    {#if p.state === 'active'}
      {#each p.lines as text, i (i)}
        {@const isLast = i === p.lines.length - 1}
        <div
          class={isLast ? 'overflow-hidden' : 'overflow-hidden whitespace-nowrap text-ellipsis'}
          style:font-size={isLast ? '17px' : '14px'}
          style:font-weight={isLast ? 500 : 400}
          style:line-height="1.3"
          style:letter-spacing="-0.15px"
          style:color="var(--vt-text)"
          style:opacity={opacityFor(i, p.lines.length)}
          style:display={isLast ? '-webkit-box' : undefined}
          style:-webkit-line-clamp={isLast ? '2' : undefined}
          style:-webkit-box-orient={isLast ? 'vertical' : undefined}>
          {text}
          {#if isLast}
            <span
              class="inline-block w-[7px] h-[15px] ml-[3px] align-middle"
              style:background="var(--vt-accent)"
              style:animation="vt-blink 0.9s steps(2) infinite"></span>
          {/if}
        </div>
      {/each}
    {:else if p.state === 'reconnecting'}
      <div class="text-xs mb-1" style:color="var(--vt-dim)">
        Connection to Soniox dropped — attempt {p.attempt ?? 1}, retrying in {(p.retryInMs ?? 0) / 1000} s
      </div>
      <div
        class="text-[17px] font-medium"
        style:color="var(--vt-warn)"
        style:letter-spacing="-0.15px">
        Đang kết nối lại…
      </div>
    {:else}
      <div class="text-xs mb-1" style:color="var(--vt-dim)">
        Voxtide overlay · open the main window and press
        <span style:font-family="'Geist Mono Variable', monospace" style:color="var(--vt-muted)">⌃⇧V</span>
        to start
      </div>
      <div
        class="text-[17px] font-medium"
        style:color="var(--vt-muted)"
        style:letter-spacing="-0.15px">
        Waiting for audio
      </div>
    {/if}
  </div>
</div>
