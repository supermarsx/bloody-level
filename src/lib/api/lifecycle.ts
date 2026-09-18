import { invoke, isTauri } from "./index";

/** Reload the current webview without restarting the native process. */
export function restartFrontend(): void {
  if (typeof window !== "undefined") window.location.reload();
}

/** Restart the native Tauri process, or reload when running in a browser. */
export async function restartApp(): Promise<void> {
  if (!isTauri()) {
    restartFrontend();
    return;
  }
  // The native command intentionally never resolves: Tauri replaces the
  // process immediately after requesting the restart.
  await invoke<void>("restart_app", undefined, { silentAuth: true });
}
