import { invoke } from "./index";

export async function deleteReport(reportId: string): Promise<void> {
  await invoke("delete_report", { reportId });
}

export interface BulkDeleteResult {
  deleted: number;
  failed: string[];
}

export async function bulkDeleteReports(
  reportIds: string[],
): Promise<BulkDeleteResult> {
  return invoke<BulkDeleteResult>("bulk_delete_reports", { reportIds });
}

export interface MergePatientsArgs {
  source_id: string;
  target_id: string;
}

export interface MergePatientsResult {
  reports_moved: number;
  source_deleted: boolean;
}

export async function mergePatients(
  args: MergePatientsArgs,
): Promise<MergePatientsResult> {
  return invoke<MergePatientsResult>("merge_patients", { args });
}

export interface ReloadOntologyResult {
  analytes_installed: number;
}

export async function reloadOntology(): Promise<ReloadOntologyResult> {
  return invoke<ReloadOntologyResult>("reload_ontology");
}

export async function deletePatient(patientId: string): Promise<void> {
  await invoke("delete_patient", { patientId });
}

export async function deleteResult(resultId: number): Promise<void> {
  await invoke("delete_result", { resultId });
}

export interface UpdatePatientArgs {
  id: string;
  display_name: string;
  sex: "m" | "f" | "x" | "?";
  dob_iso: string | null;
  nickname?: string | null;
  notes?: string | null;
}

export async function updatePatient(args: UpdatePatientArgs): Promise<void> {
  await invoke("update_patient", { args });
}

export interface UpdateReportArgs {
  id: string;
  collection_date_iso: string;
  emission_date_iso: string | null;
  lab_entity: string | null;
  requesting_physician: string | null;
  cycle_phase?: string | null;
}

export async function updateReport(args: UpdateReportArgs): Promise<void> {
  await invoke("update_report", { args });
}

export type CyclePhase =
  "follicular" | "ovulation" | "luteal" | "postmenopause" | null;

export async function setReportCyclePhase(
  id: string,
  cyclePhase: CyclePhase,
): Promise<void> {
  await invoke("set_report_cycle_phase", {
    args: { id, cycle_phase: cyclePhase },
  });
}

export async function setReportNickname(
  id: string,
  nickname: string | null,
): Promise<void> {
  await invoke("set_report_nickname", { args: { id, nickname } });
}

export interface BackfillSexResult {
  patients_scanned: number;
  patients_updated: number;
  patients_still_unknown: number;
}

export async function backfillPatientSex(): Promise<BackfillSexResult> {
  return invoke<BackfillSexResult>("backfill_patient_sex");
}

export interface BackfillDobResult {
  patients_scanned: number;
  patients_updated: number;
  patients_still_unknown: number;
  exact_updates: number;
  approximate_updates: number;
}

export async function backfillPatientDob(): Promise<BackfillDobResult> {
  return invoke<BackfillDobResult>("backfill_patient_dob");
}

export type PatientSex = "m" | "f" | "x" | "?";

export interface CreatePatientArgs {
  display_name: string;
  sex: PatientSex;
  dob_iso: string | null;
}

export interface CreatePatientResult {
  id: string;
  created: boolean;
}

export async function createPatient(
  args: CreatePatientArgs,
): Promise<CreatePatientResult> {
  return invoke<CreatePatientResult>("create_patient", { args });
}

export interface PatientOverview {
  id: string;
  display_name: string;
  nickname: string | null;
  notes: string | null;
  sex: string;
  dob_iso: string | null;
  report_count: number;
  latest_collection_date_iso: string | null;
  earliest_collection_date_iso: string | null;
  abnormal_row_count: number;
  critical_row_count: number;
  distinct_analyte_count: number;
  recent_report_count: number;
  recent_abnormal_row_count: number;
  recent_critical_row_count: number;
}

export async function patientOverviews(): Promise<PatientOverview[]> {
  return invoke<PatientOverview[]>("patient_overviews");
}

export async function setPatientNickname(
  id: string,
  nickname: string | null,
): Promise<void> {
  await invoke("set_patient_nickname", { args: { id, nickname } });
}

export async function setPatientNotes(
  id: string,
  notes: string | null,
): Promise<void> {
  await invoke("set_patient_notes", { args: { id, notes } });
}

export async function setPatientHrtStart(
  id: string,
  hrtStartIso: string | null,
): Promise<void> {
  await invoke("set_patient_hrt_start", {
    args: { id, hrt_start_iso: hrtStartIso },
  });
}

export async function setReportAnnotations(
  id: string,
  annotations: string | null,
): Promise<void> {
  await invoke("set_report_annotations", { args: { id, annotations } });
}

export interface UpdateResultArgs {
  id: number;
  analyte_id: string | null;
  value_numeric: number | null;
  value_qualitative: string | null;
  unit: string | null;
  ref_low: number | null;
  ref_high: number | null;
  flag: string | null;
}

export async function updateResult(args: UpdateResultArgs): Promise<void> {
  await invoke("update_result", { args });
}

export interface LinkAnalyteArgs {
  raw_text: string;
  analyte_id: string;
}

export interface LinkAnalyteResult {
  rows_relinked: number;
}

export async function linkUnmatchedAnalyte(
  args: LinkAnalyteArgs,
): Promise<LinkAnalyteResult> {
  return invoke<LinkAnalyteResult>("link_unmatched_analyte", { args });
}

export interface AnalyteOption {
  id: string;
  pt_name: string;
  section: string;
  subsection: string | null;
  panel: string | null;
}

export async function listAnalytes(): Promise<AnalyteOption[]> {
  return invoke<AnalyteOption[]>("list_analytes");
}
