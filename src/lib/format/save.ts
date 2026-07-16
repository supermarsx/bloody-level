// Save a string to a user-chosen path via tauri_plugin_dialog + plugin-fs.
// Used by the CSV/JSON export flows.

import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

export interface SaveOpts {
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}

export async function saveTextFile(
  content: string,
  opts: SaveOpts = {},
): Promise<string | null> {
  const path = await save({
    defaultPath: opts.defaultPath,
    filters: opts.filters ?? [{ name: "CSV", extensions: ["csv"] }],
  });
  if (!path) return null; // user cancelled
  await writeTextFile(path, content);
  return path;
}
