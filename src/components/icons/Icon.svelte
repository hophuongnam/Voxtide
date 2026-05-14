<script lang="ts">
  import { PATHS, type IconName } from './paths';

  interface Props {
    name: IconName;
    size?: number;
    stroke?: number;
    color?: string;
    'aria-label'?: string;
  }
  const { name, size = 14, stroke = 1.6, color = 'currentColor', ...rest }: Props = $props();
</script>

<svg
  width={size} height={size} viewBox="0 0 24 24" fill="none"
  stroke={color} stroke-width={stroke} stroke-linecap="round" stroke-linejoin="round"
  role="img" aria-label={rest['aria-label'] ?? name}>
  <path d={PATHS[name]} />
</svg>
