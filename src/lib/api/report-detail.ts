import { invoke } from "./index";

export interface ReportMeta {
  id: string;
  patient_id: string;
  patient_name: string;
  patient_sex: string;
  collection_date_iso: string;
  emission_date_iso: string | null;
  age_at_collection: number | null;
  lab_entity: string | null;
  requesting_physician: string | null;
  inscription_id: string | null;
  process_id: string | null;
  origin_id: string | null;
  ingest_tier: number;
  parse_version: string;
  doc_confidence: number;
  raw_pdf_path: string;
  source_path: string;
  cycle_phase: string | null;
  nickname: string | null;
  annotations: string | null;
  hrt_start_iso: string | null;
}

export interface CategoricalTier {
  label: string;
  min?: number;
  max?: number;
}

export interface ReportRow {
  id: number;
  analyte_id: string | null;
  analyte_pt_name: string | null;
  analyte_method_annotation: string | null;
  analyte_categorical_tiers_json: string | null;
  analyte_cycle_phases_json: string | null;
  analyte_default_ref_json: string | null;
  analyte_cycle_dependent: boolean;
  raw_analyte_text: string;
  value_numeric: number | null;
  value_qualitative: string | null;
  value_raw_text: string;
  unit: string | null;
  unit_raw: string;
  ref_low: number | null;
  ref_high: number | null;
  ref_grammar: string;
  ref_raw_text: string | null;
  flag: string | null;
  method_annotation: string | null;
  parse_method: string;
  confidence: number;
  inline_prior_pdf: boolean;
  collection_date_iso: string;
}

export interface ReportStats {
  total_rows: number;
  matched_rows: number;
  unmatched_rows: number;
  abnormal_rows: number;
  critical_rows: number;
  inline_prior_rows: number;
  min_confidence: number;
  avg_confidence: number;
}

export interface ReportParseAudit {
  row_index: number;
  diagnostic: string;
  parse_method: string | null;
  confidence: number | null;
  llm_repaired: boolean;
  ocr_tier: number;
}

export interface ReportDetail {
  report: ReportMeta;
  rows: ReportRow[];
  parse_audit: ReportParseAudit[];
  unmatched_analytes: string[];
  stats: ReportStats;
  prev_report_id: string | null;
  next_report_id: string | null;
}

export async function get(reportId: string): Promise<ReportDetail> {
  return invoke<ReportDetail>("report_detail", { reportId });
}
