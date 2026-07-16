import { invoke } from "./index";
import { open } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppErrorPayload } from "./errors";

export interface IngestResult {
  report_id: string;
  patient_id: string;
  patient_name: string;
  collection_date_iso: string;
  rows_parsed: number;
  rows_unmatched: number;
  inline_priors_emitted: number;
  doc_confidence: number;
  already_ingested: boolean;
}

export interface IngestOutcome {
  path: string;
  index: number;
  ok: boolean;
  error: AppErrorPayload | null;
  result: IngestResult | null;
}

export type IngestStage =
  | "started"
  | "hashing"
  | "duplicate"
  | "extracting"
  | "extracted"
  | "parsing_header"
  | "parsing_rows"
  | "writing"
  | "completed"
  | "error";

export interface IngestProgress {
  path: string;
  stage: IngestStage;
  progress: number;
  message: string | null;
  elapsed_ms: number;
  pages: number | null;
  bytes: number | null;
  rows_parsed: number | null;
  rows_unmatched: number | null;
  inline_priors: number | null;
  doc_confidence: number | null;
  already_ingested: boolean | null;
  error: AppErrorPayload | null;
}

export interface BatchProgress {
  total: number;
  completed: number;
  failed: number;
  current_path: string | null;
  elapsed_ms: number;
}

export const STAGE_LABEL: Record<IngestStage, string> = {
  started: "Queued",
  hashing: "Hashing",
  duplicate: "Already ingested",
  extracting: "Extracting PDF",
  extracted: "Extracted",
  parsing_header: "Reading header",
  parsing_rows: "Parsing rows",
  writing: "Writing to database",
  completed: "Done",
  error: "Error",
};

// Ingestion can take many seconds on large PDFs — bump the per-call timeout.
const INGEST_TIMEOUT_MS = 180_000;

export async function ingestPdf(path: string): Promise<IngestResult> {
  return invoke<IngestResult>(
    "ingest_pdf",
    { path },
    { timeoutMs: INGEST_TIMEOUT_MS },
  );
}

export async function ingestPdfs(paths: string[]): Promise<IngestOutcome[]> {
  return invoke<IngestOutcome[]>(
    "ingest_pdfs",
    { paths },
    {
      timeoutMs: Math.max(INGEST_TIMEOUT_MS, paths.length * 30_000),
    },
  );
}

// Re-entry guard for the file picker. The Tauri dialog plugin on Windows
// (WebView2) can leave the modal-state stuck if a second `open()` lands
// while the first hasn't fully torn down — symptom: button does nothing
// on the second click after a cancellation. We coalesce concurrent calls
// to a single in-flight Promise so the UI can't race the plugin.
let pickingInFlight: Promise<string[]> | null = null;

export async function pickPdfs(): Promise<string[]> {
  if (pickingInFlight) return pickingInFlight;
  pickingInFlight = (async () => {
    try {
      const sel = await open({
        multiple: true,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (sel == null) return [];
      return Array.isArray(sel) ? sel : [sel];
    } finally {
      // Yield one microtask before clearing the lock so a synchronous
      // double-click on the trigger button doesn't slip past the guard.
      await Promise.resolve();
      pickingInFlight = null;
    }
  })();
  return pickingInFlight;
}

export async function subscribeProgress(
  onFile: (p: IngestProgress) => void,
  onBatch?: (b: BatchProgress) => void,
): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];
  unlisteners.push(
    await listen<IngestProgress>("ingest:progress", (e) => onFile(e.payload)),
  );
  if (onBatch) {
    unlisteners.push(
      await listen<BatchProgress>("ingest:batch", (e) => onBatch(e.payload)),
    );
  }
  return () => unlisteners.forEach((u) => u());
}
