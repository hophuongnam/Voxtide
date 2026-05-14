<script lang="ts">
  interface Props { size?: number; color: string; bars?: 3 | 5; }
  const { size = 12, color, bars = 5 }: Props = $props();
  const heights = $derived(bars === 3 ? [0.55, 1.0, 0.55] : [0.4, 0.7, 1.0, 0.7, 0.4]);
  const barW = $derived(Math.max(1, Math.round(size / (heights.length * 2.5))));
  const gap = $derived(Math.max(1, Math.round(size / (heights.length * 4))));
  const totalW = $derived(heights.length * barW + (heights.length - 1) * gap);
  const startX = $derived((size - totalW) / 2);
</script>

<svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
  {#each heights as h, i}
    {@const bh = h * size * 0.7}
    <rect
      x={startX + i * (barW + gap)}
      y={(size - bh) / 2}
      width={barW} height={bh}
      rx={Math.max(0.5, barW * 0.3)}
      fill={color} />
  {/each}
</svg>
