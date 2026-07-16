// Pick a (low, high) range from an analyte's `cycle_phases_json` based on the
// report's tagged cycle phase. The seed encodes phases as keys like
// `follicular`, `ovulation`, `luteal`, `postmenopause` mapping to `[low, high]`
// arrays. Some seeds use richer keys (e.g. `follicular_-12d`); we accept any
// key that *starts with* the phase name as a match.

export interface PhaseRef {
  low: number | null;
  high: number | null;
  source: string; // the matched key
}

export function phaseRefFor(
  json: string | null | undefined,
  phase: string | null | undefined,
): PhaseRef | null {
  if (!json || !phase) return null;
  let parsed: Record<string, [number | null, number | null]>;
  try {
    parsed = JSON.parse(json);
  } catch {
    return null;
  }
  const norm = phase.toLowerCase();
  // Exact match wins.
  const exact = parsed[norm];
  if (Array.isArray(exact) && exact.length >= 2) {
    return { low: exact[0] ?? null, high: exact[1] ?? null, source: norm };
  }
  // Prefix match (e.g. `follicular_-12d` for `follicular`).
  for (const [k, v] of Object.entries(parsed)) {
    if (k.toLowerCase().startsWith(norm) && Array.isArray(v) && v.length >= 2) {
      return { low: v[0] ?? null, high: v[1] ?? null, source: k };
    }
  }
  return null;
}

export function flagForPhaseRef(value: number, ref: PhaseRef): string | null {
  if (ref.low != null && value < ref.low) return "low";
  if (ref.high != null && value > ref.high) return "high";
  if (ref.low != null || ref.high != null) return "normal";
  return null;
}
