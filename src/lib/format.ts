export interface Group<T> { label: 'Today' | 'Yesterday' | 'This week' | 'Earlier'; items: T[]; }

export function groupByDate<T>(items: T[], getMs: (x: T) => number, nowMs: number = Date.now()): Group<T>[] {
  const now = new Date(nowMs);
  const startOfDayUtc = (d: Date) => Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
  const today = startOfDayUtc(now);
  const yesterday = today - 24 * 3600 * 1000;
  const weekStart = today - 6 * 24 * 3600 * 1000;

  const labels: Group<T>['label'][] = ['Today', 'Yesterday', 'This week', 'Earlier'];
  const buckets: Record<Group<T>['label'], T[]> = {
    'Today': [], 'Yesterday': [], 'This week': [], 'Earlier': [],
  };
  for (const item of items) {
    const ts = getMs(item);
    if (ts >= today) buckets['Today'].push(item);
    else if (ts >= yesterday) buckets['Yesterday'].push(item);
    else if (ts >= weekStart) buckets['This week'].push(item);
    else buckets['Earlier'].push(item);
  }
  return labels.filter(l => buckets[l].length > 0).map(l => ({ label: l, items: buckets[l] }));
}

export function formatDuration(ms: number): string {
  if (ms <= 0) return '—';
  const secs = Math.floor(ms / 1000);
  const m = Math.floor(secs / 60);
  const h = Math.floor(m / 60);
  if (h > 0) return `${h}h ${String(m % 60).padStart(2, '0')}m`;
  if (m > 0) return `${m}m`;
  return `${secs}s`;
}

export function formatTime(ms: number): string {
  const d = new Date(ms);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export function formatElapsed(ms: number): string {
  const s = Math.floor(ms / 1000);
  const hh = String(Math.floor(s / 3600)).padStart(2, '0');
  const mm = String(Math.floor((s % 3600) / 60)).padStart(2, '0');
  const ss = String(s % 60).padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}
