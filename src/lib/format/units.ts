// Unit display normalization (parser-side normalization is in Rust).

const NICE: Record<string, string> = {
  'g/dl': 'g/dL',
  'mg/dl': 'mg/dL',
  'mmol/l': 'mmol/L',
  'u/l': 'U/L',
  'ui/ml': 'UI/mL',
  'ng/dl': 'ng/dL',
  'ng/ml': 'ng/mL',
  'ng/l': 'ng/L',
  'pg/ml': 'pg/mL',
  'µg/dl': 'µg/dL',
  'mui/l': 'mUI/L',
  'mui/ml': 'mUI/mL',
  'nmol/l': 'nmol/L',
  'mmol/mol': 'mmol/mol',
  'x 10^3/µl': '×10³/µL',
  'x 10^6/µl': '×10⁶/µL',
  'ml/min/1,73 m2': 'mL/min/1.73 m²'
};

export function prettyUnit(unit: string | null | undefined): string {
  if (!unit) return '';
  const k = unit.trim().toLowerCase();
  return NICE[k] ?? unit;
}
