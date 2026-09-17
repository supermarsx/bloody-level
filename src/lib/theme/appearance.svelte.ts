// Appearance preferences — accent color, density, font scale, motion.
// Each setting is reflected as a `data-<key>` attribute on the <html>
// element and the CSS in `app.css` keys against those attributes to
// adjust visual variables without re-rendering.
//
// All values are persisted to the encrypted settings table so they
// survive across launches.

import * as settings from "$api/settings";
import { setDebounced } from "$api/debounced-settings";

const KEYS = {
  accent: "appearance.accent",
  density: "appearance.density",
  fontScale: "appearance.font_scale",
  reduceMotion: "appearance.reduce_motion",
  fontFamily: "appearance.font_family",
} as const;

export type AccentName =
  "violet" | "blue" | "emerald" | "rose" | "amber" | "teal";
export type Density = "comfortable" | "compact";
export type FontScale = "tiny" | "sm" | "md" | "lg" | "gigantic";
export type FontFamily = "sans" | "serif" | "mono";

export const ACCENT_PRESETS: {
  id: AccentName;
  label: string;
  swatch: string;
}[] = [
  { id: "violet", label: "Violet", swatch: "#7c3aed" },
  { id: "blue", label: "Blue", swatch: "#2563eb" },
  { id: "emerald", label: "Emerald", swatch: "#059669" },
  { id: "rose", label: "Rose", swatch: "#e11d48" },
  { id: "amber", label: "Amber", swatch: "#d97706" },
  { id: "teal", label: "Teal", swatch: "#0d9488" },
];

class AppearanceStore {
  accent = $state<AccentName>("violet");
  density = $state<Density>("comfortable");
  fontScale = $state<FontScale>("md");
  reduceMotion = $state(false);
  fontFamily = $state<FontFamily>("sans");
  loaded = $state(false);

  /** Hydrate from the settings table. Idempotent. */
  async load() {
    if (this.loaded) return;
    try {
      const a = await settings.get<AccentName>(KEYS.accent);
      const d = await settings.get<Density>(KEYS.density);
      const f = await settings.get<FontScale>(KEYS.fontScale);
      const r = await settings.get<boolean>(KEYS.reduceMotion);
      const ff = await settings.get<FontFamily>(KEYS.fontFamily);
      if (a) this.accent = a;
      if (d) this.density = d;
      if (f) this.fontScale = f;
      if (typeof r === "boolean") this.reduceMotion = r;
      if (ff) this.fontFamily = ff;
    } catch {
      /* stay on defaults */
    }
    this.loaded = true;
    this.apply();
  }

  /** Reflect every setting onto the document root. */
  apply() {
    if (typeof document === "undefined") return;
    const root = document.documentElement;
    root.setAttribute("data-accent", this.accent);
    root.setAttribute("data-density", this.density);
    root.setAttribute("data-font-scale", this.fontScale);
    root.setAttribute("data-font-family", this.fontFamily);
    root.setAttribute(
      "data-reduce-motion",
      this.reduceMotion ? "true" : "false",
    );
    // Fire a synthetic theme-change so chart themes pick up new
    // accent / chart-1 colors immediately.
    root.dispatchEvent(
      new CustomEvent("theme-change", { detail: "appearance" }),
    );
  }

  // Setters update the in-memory state + DOM attributes immediately, then
  // queue the persisted save through the 1.5s debouncer so a flurry of
  // toggle clicks ends with a single round-trip + one toast.
  setAccent(v: AccentName) {
    this.accent = v;
    this.apply();
    setDebounced(KEYS.accent, v);
  }
  setDensity(v: Density) {
    this.density = v;
    this.apply();
    setDebounced(KEYS.density, v);
  }
  setFontScale(v: FontScale) {
    this.fontScale = v;
    this.apply();
    setDebounced(KEYS.fontScale, v);
  }
  setReduceMotion(v: boolean) {
    this.reduceMotion = v;
    this.apply();
    setDebounced(KEYS.reduceMotion, v);
  }
  setFontFamily(v: FontFamily) {
    this.fontFamily = v;
    this.apply();
    setDebounced(KEYS.fontFamily, v);
  }

  reset() {
    void this.setAccent("violet");
    void this.setDensity("comfortable");
    void this.setFontScale("md");
    void this.setReduceMotion(false);
    void this.setFontFamily("sans");
  }
}

export const appearance = new AppearanceStore();
