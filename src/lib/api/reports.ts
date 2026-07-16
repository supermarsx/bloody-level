import { invoke } from "./index";

export interface PatientSummary {
  id: string;
  display_name: string;
  sex: string;
  report_count: number;
  latest_collection_date_iso: string | null;
  nickname?: string | null;
  notes?: string | null;
  dob_iso?: string | null;
  hrt_start_iso?: string | null;
}

export interface ReportSummary {
  id: string;
  patient_id: string;
  patient_name: string;
  collection_date_iso: string;
  doc_confidence: number;
  ingest_tier: number;
  row_count: number;
  nickname: string | null;
  annotations: string | null;
}

export interface AnalyteReading {
  date: string;
  value: number | null;
  qualitative: string | null;
  unit: string | null;
  ref_low: number | null;
  ref_high: number | null;
  flag: string | null;
  patient_id: string;
  patient_name: string;
  patient_sex: string;
  method: string | null;
  source_report_id: string;
  source_report_nickname: string | null;
  inline_prior: boolean;
}

export async function listPatients(): Promise<PatientSummary[]> {
  return invoke<PatientSummary[]>("list_patients");
}

export async function listReports(
  patientId?: string,
): Promise<ReportSummary[]> {
  return invoke<ReportSummary[]>("list_reports", {
    patientId: patientId ?? null,
  });
}

export async function analyteTimeseries(
  analyteId: string,
  patientId?: string,
): Promise<AnalyteReading[]> {
  return invoke<AnalyteReading[]>("analyte_timeseries", {
    analyteId,
    patientId: patientId ?? null,
  });
}

export interface FlaggedAnalytesResult {
  abnormal: string[];
  subclinical: string[];
}

export async function listFlaggedAnalytes(
  patientId: string,
): Promise<FlaggedAnalytesResult> {
  return invoke<FlaggedAnalytesResult>("list_flagged_analytes", { patientId });
}
