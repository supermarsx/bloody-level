import { invoke } from "./index";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface TierStatus {
  feature: string;
  compiled: boolean;
  enabled_in_settings: boolean;
  model_present: boolean;
  estimated_size_bytes: number;
  disk_size_bytes: number | null;
  loading: boolean;
  loaded: boolean;
  loaded_model_path: string | null;
  last_error: string | null;
}

export interface ModelTierStatus extends TierStatus {
  configured_model_path: string | null;
  configured_model_present: boolean;
}

export interface PdfiumStatus {
  available: boolean;
  estimated_size_bytes: number;
  disk_size_bytes: number | null;
  error: string | null;
}

export interface DownloadProgress {
  resource: string;
  file: string;
  file_index: number;
  file_count: number;
  downloaded_bytes: number;
  total_bytes: number | null;
  progress: number | null;
  done: boolean;
  cancelled: boolean;
}

export async function subscribeDownloadProgress(
  onProgress: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("tier:download-progress", (event) => {
    onProgress(event.payload);
  });
}

export async function cancelDownload(): Promise<void> {
  return invoke("tier_cancel_download");
}

export async function deleteModel(
  tier: "llm" | "olmocr" | "tesseract",
): Promise<void> {
  return invoke("tier_delete_model", { tier });
}

export async function tesseract(): Promise<TierStatus> {
  return invoke("tier_status_tesseract");
}
export async function llm(): Promise<ModelTierStatus> {
  return invoke("tier_status_llm");
}
export async function olmocr(): Promise<ModelTierStatus> {
  return invoke("tier_status_olmocr");
}
export async function pdfium(): Promise<PdfiumStatus> {
  return invoke("tier_status_pdfium");
}

export async function loadLlm(modelPath: string): Promise<ModelTierStatus> {
  return invoke("tier_load_llm", { modelPath });
}

export async function unloadLlm(): Promise<ModelTierStatus> {
  return invoke("tier_unload_llm");
}

export async function downloadLlm(): Promise<ModelTierStatus> {
  return invoke("tier_download_llm", undefined, { timeoutMs: 0 });
}

export async function loadOlmocr(modelPath: string): Promise<ModelTierStatus> {
  return invoke("tier_load_olmocr", { modelPath });
}

export async function unloadOlmocr(): Promise<ModelTierStatus> {
  return invoke("tier_unload_olmocr");
}

export async function downloadOlmocr(): Promise<ModelTierStatus> {
  return invoke("tier_download_olmocr", undefined, { timeoutMs: 0 });
}

export async function downloadTesseractLanguage(
  language: string,
): Promise<TierStatus> {
  return invoke(
    "tier_download_tesseract_language",
    { language },
    { timeoutMs: 0 },
  );
}
