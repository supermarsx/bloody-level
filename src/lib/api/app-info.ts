import { invoke } from "./index";

export interface Features {
  embedded_llm: boolean;
  embedded_ocr_vision: boolean;
  tesseract_ocr: boolean;
  debug_assertions: boolean;
}

export interface DbStats {
  patient_count: number;
  report_count: number;
  result_count: number;
  inline_prior_count: number;
  analyte_count: number;
  alias_count: number;
  audit_count: number;
  earliest_collection_date_iso: string | null;
  latest_collection_date_iso: string | null;
  journal_mode: string | null;
  page_size: number | null;
  page_count: number | null;
  sqlcipher_version: string | null;
}

export interface AppInfo {
  name: string;
  version: string;
  build_profile: string;
  target_triple: string;
  data_dir: string;
  data_dir_size_bytes: number | null;
  keystore_path: string;
  keystore_size_bytes: number | null;
  db_path: string;
  db_size_bytes: number | null;
  db_sidecar_bytes: number | null;
  pdf_dir: string;
  pdf_count: number;
  pdf_size_bytes: number | null;
  models_dir: string;
  models_count: number;
  models_size_bytes: number | null;
  db_stats: DbStats | null;
  features: Features;
  pdfium_available: boolean;
  pdfium_error: string | null;
}

export async function get(): Promise<AppInfo> {
  return invoke<AppInfo>("app_info");
}

export interface ExportVaultResult {
  destination: string;
  bytes_copied: number;
  files_copied: number;
  manifest_path: string;
}

export async function exportVault(
  destination: string,
): Promise<ExportVaultResult> {
  return invoke<ExportVaultResult>("export_vault", { destination });
}

export interface ImportVaultResult {
  source: string;
  bytes_copied: number;
  files_copied: number;
  backup_dir: string;
}

export async function importVault(source: string): Promise<ImportVaultResult> {
  return invoke<ImportVaultResult>("import_vault", { source });
}

export function formatBytes(b: number | null | undefined): string {
  if (b == null) return "—";
  if (b < 1024) return `${b} B`;
  const kb = b / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  const mb = kb / 1024;
  if (mb < 1024) return `${mb.toFixed(1)} MB`;
  return `${(mb / 1024).toFixed(2)} GB`;
}
