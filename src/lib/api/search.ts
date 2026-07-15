import { invoke } from './index';

export interface SearchHit {
  kind: 'patient' | 'analyte' | 'report';
  id: string;
  label: string;
  sub: string | null;
  href: string;
}

export interface SearchResults {
  patients: SearchHit[];
  analytes: SearchHit[];
  reports: SearchHit[];
}

export async function globalSearch(query: string): Promise<SearchResults> {
  return invoke<SearchResults>('global_search', { query });
}
