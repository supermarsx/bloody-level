import { invoke } from "./index";

export interface CsvOut {
  filename: string;
  content: string;
}

export async function exportAnalyteTimeseriesCsv(
  analyteId: string,
  patientId?: string,
): Promise<CsvOut> {
  return invoke<CsvOut>("export_analyte_timeseries_csv", {
    analyteId,
    patientId: patientId ?? null,
  });
}

export async function exportReportRowsCsv(reportId: string): Promise<CsvOut> {
  return invoke<CsvOut>("export_report_rows_csv", { reportId });
}
