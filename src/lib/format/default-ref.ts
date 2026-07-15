// Helpers for sex-stratified / universal "default reference" ranges from the
// ontology. Used as a fallback in the Ref column when the row has no printed
// range and no categorical-tier match.

export interface DefaultRef {
  low: number | null;
  high: number | null;
  source: 'm' | 'f' | 'all';
}

export function defaultRefFor(json: string | null | undefined, sex: string | null | undefined): DefaultRef | null {
  if (!json) return null;
  let parsed: Record<string, [number | null, number | null]>;
  try {
    parsed = JSON.parse(json);
  } catch {
    return null;
  }

  const trySex = (key: 'm' | 'f' | 'all'): DefaultRef | null => {
    const v = parsed[key];
    if (!Array.isArray(v) || v.length < 2) return null;
    return { low: v[0] ?? null, high: v[1] ?? null, source: key };
  };

  if (sex === 'm' || sex === 'f') {
    const own = trySex(sex as 'm' | 'f');
    if (own) return own;
  }
  return trySex('all');
}

export function formatDefaultRef(ref: DefaultRef): string {
  if (ref.low != null && ref.high != null) return `${ref.low}–${ref.high}`;
  if (ref.high != null) return `< ${ref.high}`;
  if (ref.low != null) return `≥ ${ref.low}`;
  return '—';
}

export function flagForDefaultRef(value: number, ref: DefaultRef): string | null {
  if (ref.low != null && value < ref.low) return 'low';
  if (ref.high != null && value > ref.high) return 'high';
  if (ref.low != null || ref.high != null) return 'normal';
  return null;
}
