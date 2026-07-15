import { invoke } from './index';

export interface ReparseResult {
  report_id: string;
  patient_name: string;
  collection_date_iso: string;
  rows_before: number;
  rows_after: number;
  rows_unmatched: number;
  inline_priors_emitted: number;
  doc_confidence: number;
  parse_version: string;
}

export interface ReparseBatchResult {
  total: number;
  succeeded: number;
  failed: Array<[string, string]>;
  total_rows_after: number;
}

export async function reparseReport(reportId: string): Promise<ReparseResult> {
  return invoke<ReparseResult>('reparse_report', { reportId });
}

export async function reparseAll(): Promise<ReparseBatchResult> {
  return invoke<ReparseBatchResult>('reparse_all_reports', undefined, { timeoutMs: 600_000 });
}
