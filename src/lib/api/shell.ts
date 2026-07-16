import { invoke } from "./index";

export async function openFileExternal(path: string): Promise<void> {
  return invoke<void>("open_file_external", { path });
}

/** Opens an http(s) URL in the OS default browser. Backend rejects any
 *  other scheme so `file://` etc. can't slip through. */
export async function openUrl(url: string): Promise<void> {
  return invoke<void>("open_url", { url });
}
