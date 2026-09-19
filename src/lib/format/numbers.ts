import { appearance } from "$theme/appearance.svelte";

export function formatNumber(v: number | null | undefined, digits = 2): string {
  if (v == null || Number.isNaN(v)) return "—";
  const abs = Math.abs(v);
  const minimumFractionDigits = abs >= 100 ? 0 : abs >= 10 ? 1 : digits;
  return new Intl.NumberFormat(appearance.resolvedLocale, {
    minimumFractionDigits,
    maximumFractionDigits: minimumFractionDigits,
  }).format(v);
}

export function formatPercent(
  v: number | null | undefined,
  digits = 1,
): string {
  if (v == null || Number.isNaN(v)) return "—";
  return `${new Intl.NumberFormat(appearance.resolvedLocale, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(v)}%`;
}

export function formatDelta(
  curr: number,
  prev: number,
): { abs: number; pct: number; dir: "up" | "down" | "flat" } {
  const abs = curr - prev;
  const pct = prev === 0 ? 0 : (abs / prev) * 100;
  const dir = abs > 0 ? "up" : abs < 0 ? "down" : "flat";
  return { abs, pct, dir };
}
