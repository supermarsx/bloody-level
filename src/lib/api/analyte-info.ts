import { invoke } from './index';

export interface AnalyteInfo {
  id: string;
  pt_name: string;
  loinc: string | null;
  section: string;
  subsection: string | null;
  panel: string | null;
  method_annotation: string | null;
  expected_units: string[];
  default_ref_json: string | null;
  sex_dependent: boolean;
  age_dependent: boolean;
  cycle_dependent: boolean;
  is_derived: boolean;
  is_qualitative: boolean;
  is_panel_header: boolean;
  description: string | null;
  high_means: string | null;
  low_means: string | null;
  unit_notes: string | null;
  categorical_tiers_json: string | null;
  cycle_phases_json: string | null;
  aliases: string[];
}

export async function get(analyteId: string): Promise<AnalyteInfo> {
  return invoke<AnalyteInfo>('analyte_info', { analyteId });
}

export interface AnalyteOntologyEntry {
  id: string;
  pt_name: string;
  section: string;
  subsection: string | null;
  panel: string | null;
  loinc: string | null;
  method_annotation: string | null;
  expected_units: string[];
  default_ref_json: string | null;
  categorical_tiers_json: string | null;
  cycle_phases_json: string | null;
  sex_dependent: boolean;
  age_dependent: boolean;
  cycle_dependent: boolean;
  is_qualitative: boolean;
  is_derived: boolean;
  is_panel_header: boolean;
  paired_value: boolean;
  has_description: boolean;
  has_high_means: boolean;
  has_low_means: boolean;
  has_unit_notes: boolean;
  alias_count: number;
  result_count: number;
  source: 'seed' | 'user';
}

export async function listOntologyEntries(): Promise<AnalyteOntologyEntry[]> {
  return invoke<AnalyteOntologyEntry[]>('list_ontology_entries');
}

/**
 * Write payload shared by create + update. The frontend hands every field
 * over so nothing is silently inferred — keeps the round-trip predictable.
 */
export interface AnalyteWriteArgs {
  id: string;
  pt_name: string;
  section: string;
  subsection: string | null;
  panel: string | null;
  loinc: string | null;
  method_annotation: string | null;
  expected_units: string[];
  default_ref_json: string | null;
  categorical_tiers_json: string | null;
  cycle_phases_json: string | null;
  sex_dependent: boolean;
  age_dependent: boolean;
  cycle_dependent: boolean;
  is_qualitative: boolean;
  is_derived: boolean;
  is_panel_header: boolean;
  paired_value: boolean;
  description: string | null;
  high_means: string | null;
  low_means: string | null;
  unit_notes: string | null;
  aliases: string[];
}

export async function createAnalyte(args: AnalyteWriteArgs): Promise<void> {
  await invoke('create_analyte', { args });
}
export async function updateAnalyte(args: AnalyteWriteArgs): Promise<void> {
  await invoke('update_analyte', { args });
}
export async function deleteAnalyte(analyteId: string): Promise<void> {
  await invoke('delete_analyte', { analyteId });
}
export async function addAnalyteAlias(analyteId: string, alias: string): Promise<void> {
  await invoke('add_analyte_alias', { args: { analyte_id: analyteId, alias } });
}
export async function removeAnalyteAlias(alias: string): Promise<void> {
  await invoke('remove_analyte_alias', { alias });
}
