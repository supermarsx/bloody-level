// Helpers for categorical-tier reference ranges (Vit D, Ferritina, …).

export interface CategoricalTier {
  label: string;
  min?: number;
  max?: number;
}

export function parseTiers(json: string | null | undefined): CategoricalTier[] {
  if (!json) return [];
  try {
    const v = JSON.parse(json);
    return Array.isArray(v) ? (v as CategoricalTier[]) : [];
  } catch {
    return [];
  }
}

export function matchTier(value: number, tiers: CategoricalTier[]): CategoricalTier | null {
  for (const t of tiers) {
    const minOk = t.min === undefined || value >= t.min;
    const maxOk = t.max === undefined || value < t.max;
    if (minOk && maxOk) return t;
  }
  return null;
}

/// Map a tier label to a flag used by FlagPill. Conservative: anything
/// containing "deficiência", "insuficiência", "ferropénia" → low; "elevado",
/// "sobrecarga", "toxicidade" → high; "normal", "suficiência", "limite normal"
/// → normal; everything else → null (no pill).
export function tierToFlag(label: string): string | null {
  const k = label
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase();
  if (k.includes('toxicidade') || k.includes('sobrecarga') || k.includes('muito elevado')) {
    return 'critical_high';
  }
  if (k.includes('elevado')) return 'high';
  if (k.includes('deficiencia') || k.includes('ferropenia absoluta')) {
    return 'low';
  }
  if (k.includes('insuficiencia') || k.includes('ferropenia funcional')) {
    return 'low';
  }
  if (k.includes('limite')) return 'normal';
  if (k.includes('normal') || k.includes('suficiencia') || k.includes('baixo ou moderado')) {
    return 'normal';
  }
  return null;
}

export function formatTierRange(tier: CategoricalTier): string {
  if (tier.min !== undefined && tier.max !== undefined) {
    return `${tier.min}–${tier.max}`;
  }
  if (tier.max !== undefined) return `< ${tier.max}`;
  if (tier.min !== undefined) return `≥ ${tier.min}`;
  return '—';
}
