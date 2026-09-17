// Debounced settings writer.
//
// Toggles that are user-tweaked through the Settings UI fire off a write
// per change. Without batching, a user dragging through three accent
// swatches in succession produces three round-trips to the encrypted
// store — and three "saved" toasts. This wrapper coalesces all calls
// landing within `DELAY_MS` into a single flush plus one toast.
//
// In-memory state still updates synchronously via the appearance / chart
// stores; only the *persisted save* is debounced.

import * as settings from "./settings";
import { toasts } from "../toasts/store.svelte";

const DELAY_MS = 1500;
const pending = new Map<string, unknown>();
let flushHandle: ReturnType<typeof setTimeout> | null = null;

/**
 * Friendlier label fragment per known key prefix — used by the toast so
 * the user sees what got saved without the `appearance.font_scale`-style
 * dotted-key noise.
 */
const PREFIX_LABEL: Record<string, string> = {
  appearance: "Appearance",
  chart: "Chart preference",
  dashboard: "Dashboard",
};

function summariseKeys(keys: string[]): string {
  if (keys.length === 1) {
    const k = keys[0];
    const prefix = k.split(".")[0];
    const tail = k.split(".").slice(1).join(".").replace(/_/g, " ");
    return PREFIX_LABEL[prefix] ? `${PREFIX_LABEL[prefix]} · ${tail}` : k;
  }
  // Mixed-prefix flush — group counts.
  const groups: Record<string, number> = {};
  for (const k of keys) {
    const prefix = k.split(".")[0];
    groups[prefix] = (groups[prefix] ?? 0) + 1;
  }
  return Object.entries(groups)
    .map(([prefix, n]) => `${PREFIX_LABEL[prefix] ?? prefix} ×${n}`)
    .join(" · ");
}

async function doFlush() {
  flushHandle = null;
  const entries = [...pending.entries()];
  pending.clear();
  if (entries.length === 0) return;
  try {
    for (const [k, v] of entries) {
      await settings.set(k, v);
    }
    toasts.success(
      entries.length === 1
        ? "Setting saved"
        : `${entries.length} settings saved`,
      summariseKeys(entries.map(([k]) => k)),
    );
  } catch (e) {
    toasts.error(e);
  }
}

/**
 * Queue a setting for debounced persistence. Each subsequent call extends
 * the timer by `DELAY_MS` (ie. last-change wins on the timer), so a busy
 * configuration session ends with one final save.
 */
export function setDebounced(key: string, value: unknown): void {
  pending.set(key, value);
  if (flushHandle) clearTimeout(flushHandle);
  flushHandle = setTimeout(doFlush, DELAY_MS);
}

/** Force the buffered batch to write immediately. Useful before navigating. */
export async function flushNow(): Promise<void> {
  if (flushHandle) {
    clearTimeout(flushHandle);
    flushHandle = null;
  }
  await doFlush();
}

// Fire any pending writes when the page is about to unload, so a user
// quitting the app right after flipping a toggle still sees their
// preference persisted on next launch.
if (typeof window !== "undefined") {
  window.addEventListener("beforeunload", () => {
    if (pending.size > 0) {
      // Best-effort synchronous-ish flush. We can't await here, so the
      // worst case is the last write doesn't land — but the OS-level
      // save is so cheap it usually completes before the unload finishes.
      void doFlush();
    }
  });
  // Also flush when the WebView is hidden (tab switch / app minimised on
  // some platforms) — same rationale.
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden" && pending.size > 0) {
      void doFlush();
    }
  });
}
