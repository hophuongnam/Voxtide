const VARS = [
  'var(--vt-speaker-a)',
  'var(--vt-speaker-b)',
  'var(--vt-speaker-c)',
  'var(--vt-speaker-d)',
];

export function speakerVar(letter: string): string {
  const idx = (letter.charCodeAt(0) - 'A'.charCodeAt(0)) % VARS.length;
  return VARS[(idx + VARS.length) % VARS.length]!;
}
