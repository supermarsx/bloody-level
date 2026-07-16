import { invoke } from "./index";

export interface SettingEntry {
  key: string;
  value_json: string;
}

export async function get<T = unknown>(key: string): Promise<T | null> {
  const raw = await invoke<string | null>("settings_get", { key });
  if (raw == null) return null;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return null;
  }
}

export async function set(key: string, value: unknown): Promise<void> {
  await invoke("settings_set", { key, valueJson: JSON.stringify(value) });
}

export async function getAll(): Promise<Record<string, unknown>> {
  const entries = await invoke<SettingEntry[]>("settings_get_all");
  const out: Record<string, unknown> = {};
  for (const e of entries) {
    try {
      out[e.key] = JSON.parse(e.value_json);
    } catch {
      out[e.key] = e.value_json;
    }
  }
  return out;
}
