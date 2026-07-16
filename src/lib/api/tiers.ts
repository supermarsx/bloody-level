import { invoke } from "./index";

export interface TierStatus {
  feature: string;
  compiled: boolean;
  enabled_in_settings: boolean;
  model_present: boolean;
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
  error: string | null;
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

export async function loadOlmocr(modelPath: string): Promise<ModelTierStatus> {
  return invoke("tier_load_olmocr", { modelPath });
}

export async function unloadOlmocr(): Promise<ModelTierStatus> {
  return invoke("tier_unload_olmocr");
}
