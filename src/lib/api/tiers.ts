import { invoke } from './index';

export interface TierStatus {
  feature: string;
  compiled: boolean;
  enabled_in_settings: boolean;
  model_present: boolean;
  loaded: boolean;
}

export interface PdfiumStatus {
  available: boolean;
  error: string | null;
}

export async function tesseract(): Promise<TierStatus> { return invoke('tier_status_tesseract'); }
export async function llm(): Promise<TierStatus>       { return invoke('tier_status_llm'); }
export async function olmocr(): Promise<TierStatus>    { return invoke('tier_status_olmocr'); }
export async function pdfium(): Promise<PdfiumStatus>  { return invoke('tier_status_pdfium'); }
