import { invoke } from './index';

export interface SamplePdf {
  path: string;
  name: string;
  size_bytes: number;
}

export async function list(): Promise<SamplePdf[]> {
  return invoke<SamplePdf[]>('sample_pdf_paths');
}
