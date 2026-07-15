export function formatNumber(v: number | null | undefined, digits = 2): string {
  if (v == null || Number.isNaN(v)) return '—';
  const abs = Math.abs(v);
  if (abs >= 100) return v.toFixed(0);
  if (abs >= 10) return v.toFixed(1);
  return v.toFixed(digits);
}

export function formatPercent(v: number | null | undefined, digits = 1): string {
  if (v == null || Number.isNaN(v)) return '—';
  return `${v.toFixed(digits)}%`;
}

export function formatDelta(curr: number, prev: number): { abs: number; pct: number; dir: 'up' | 'down' | 'flat' } {
  const abs = curr - prev;
  const pct = prev === 0 ? 0 : (abs / prev) * 100;
  const dir = abs > 0 ? 'up' : abs < 0 ? 'down' : 'flat';
  return { abs, pct, dir };
}
