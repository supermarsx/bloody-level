// Centralized window-title controller.
//
// Tauri 2 doesn't auto-mirror `document.title` into the OS window title, so
// every meaningful navigation/state change funnels through `setTitle()` here.
// Format:  "<page-specific>  ·  bloody-level [<state>]"
//
// `state` surfaces transient signals: locked, ingest in flight, etc.

import { isTauri } from "$api/index";
import { t } from "$lib/i18n/index.svelte";
import { untrack } from "svelte";

const APP = "bloody-level";

class TitleStore {
  /** Page-specific subject — e.g. 'Hemoglobina · MARIANA' */
  subject = $state<string | null>(null);
  /** Transient status segments — `set('ingest', '3 of 12')`, `clear('ingest')`. */
  segments = $state<Record<string, string>>({});
  /** Lock state — prefixes the title with a plain-text status marker. */
  locked = $state(false);

  setSubject(s: string | null) {
    this.subject = s;
    this.flush();
  }
  setLocked(b: boolean) {
    this.locked = b;
    this.flush();
  }
  set(key: string, value: string) {
    // Title updates may be triggered by a $effect. Keep the segment mutation
    // untracked so that effect only depends on its own input state (for
    // example, ingest batch progress) and cannot subscribe to this write.
    untrack(() => {
      this.segments[key] = value;
    });
    this.flush();
  }
  clear(key: string) {
    untrack(() => {
      delete this.segments[key];
    });
    this.flush();
  }

  /** Compose the final title string and push to OS + document. */
  private compose(): string {
    const parts: string[] = [];
    if (this.subject && this.subject.trim()) parts.push(this.subject.trim());
    parts.push(APP);

    const stateBits: string[] = [];
    if (this.locked) stateBits.push("locked");
    for (const [k, v] of Object.entries(this.segments)) {
      stateBits.push(v ? `${k}: ${v}` : k);
    }

    let title = parts.join("  ·  ");
    if (stateBits.length > 0) {
      title += `  [${stateBits.join(" · ")}]`;
    }
    if (this.locked) title = `${t("Locked")} · ${title}`;
    return title;
  }

  private flushHandle: ReturnType<typeof setTimeout> | null = null;
  flush() {
    // Debounce so a burst of state changes (e.g., subject + segment together)
    // produces a single setTitle call.
    if (this.flushHandle) clearTimeout(this.flushHandle);
    this.flushHandle = setTimeout(() => this.applyNow(), 16);
  }

  private async applyNow() {
    const title = this.compose();
    if (typeof document !== "undefined") {
      document.title = title;
    }
    if (!isTauri()) return;
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().setTitle(title);
    } catch {
      // Non-critical — `setTitle` permission may be missing in legacy builds.
      // We still have document.title as a fallback for the WebView's own UI.
    }
  }
}

export const windowTitle = new TitleStore();

/**
 * Convenience for route components: set a page subject, auto-clear on
 * navigation away. Call from a `$effect` or `onMount`.
 */
export function setPageTitle(subject: string | null) {
  windowTitle.setSubject(subject);
}
