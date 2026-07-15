import { invoke } from './index';

export interface AuditEntry {
  id: number;
  ts: number;                       // unix seconds
  action: string;                   // 'create' | 'update' | 'delete' | 'merge' | 'ingest' | 'reload' | …
  entity_type: string;              // 'patient' | 'report' | 'result' | 'analyte' | 'alias' | 'ontology' | 'system' | 'audit'
  entity_id: string | null;
  summary: string;
  details_json: string | null;
}

export interface AuditListResult {
  entries: AuditEntry[];
  total: number;
  distinct_actions: string[];
  distinct_entity_types: string[];
}

export interface AuditListFilters {
  action?: string;
  entity_type?: string;
  entity_id?: string;
  text?: string;
  since_ts?: number;
  until_ts?: number;
  limit?: number;
  offset?: number;
}

export async function listAuditEntries(filters: AuditListFilters = {}): Promise<AuditListResult> {
  return invoke<AuditListResult>('list_audit_entries', { ...filters });
}

export interface AuditClearResult {
  deleted: number;
}

export async function clearAuditLog(): Promise<AuditClearResult> {
  return invoke<AuditClearResult>('clear_audit_log');
}
