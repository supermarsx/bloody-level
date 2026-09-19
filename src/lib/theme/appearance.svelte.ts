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
  locale: "appearance.locale",
} as const;

export type AccentName =
  "violet" | "blue" | "emerald" | "rose" | "amber" | "teal" | "slate";
export type Density = "comfortable" | "compact";
export type FontScale = "tiny" | "sm" | "md" | "lg" | "gigantic";
export type FontFamily = "sans" | "serif" | "mono";
export type AppLocale = "en-US" | "pt-PT";
export type LocalePreference = "auto" | AppLocale;

const SUPPORTED_LOCALES: AppLocale[] = ["en-US", "pt-PT"];

function browserLocaleCandidates(): string[] {
  if (typeof navigator === "undefined") return [];
  return [...(navigator.languages ?? []), navigator.language].filter(
    (value): value is string => typeof value === "string" && value.length > 0,
  );
}

/** Resolve a browser/OS locale to one of the locales the app ships. */
export function detectLocale(): AppLocale {
  const candidates = browserLocaleCandidates().map((value) =>
    value.toLowerCase(),
  );
  if (candidates.some((value) => value === "pt" || value.startsWith("pt-"))) {
    return "pt-PT";
  }
  return "en-US";
}

export function resolveLocale(preference: LocalePreference): AppLocale {
  return preference === "auto" ? detectLocale() : preference;
}

export function isLocalePreference(value: unknown): value is LocalePreference {
  return value === "auto" || SUPPORTED_LOCALES.includes(value as AppLocale);
}

export const LOCALE_OPTIONS: {
  id: LocalePreference;
  label: string;
  description: string;
}[] = [
  { id: "auto", label: "Automatic", description: "Use your device language" },
  {
    id: "en-US",
    label: "English (US)",
    description: "English regional formatting",
  },
  {
    id: "pt-PT",
    label: "Português (Portugal)",
    description: "Portuguese regional formatting",
  },
];

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
  { id: "slate", label: "Slate", swatch: "#64748b" },
];

class AppearanceStore {
  accent = $state<AccentName>("violet");
  density = $state<Density>("comfortable");
  fontScale = $state<FontScale>("md");
  reduceMotion = $state(false);
  fontFamily = $state<FontFamily>("sans");
  locale = $state<LocalePreference>("auto");
  resolvedLocale = $derived(resolveLocale(this.locale));
  loaded = $state(false);

  /** Hydrate from the settings table. Idempotent. */
  async load() {
    if (this.loaded) return;
    let loadedFromVault = false;
    try {
      const a = await settings.get<AccentName>(KEYS.accent);
      const d = await settings.get<Density>(KEYS.density);
      const f = await settings.get<FontScale>(KEYS.fontScale);
      const r = await settings.get<boolean>(KEYS.reduceMotion);
      const ff = await settings.get<FontFamily>(KEYS.fontFamily);
      const l = await settings.get<LocalePreference>(KEYS.locale);
      if (a) this.accent = a;
      if (d) this.density = d;
      if (f) this.fontScale = f;
      if (typeof r === "boolean") this.reduceMotion = r;
      if (ff) this.fontFamily = ff;
      if (isLocalePreference(l)) this.locale = l;
      loadedFromVault = true;
    } catch {
      /* Keep browser-detected defaults and retry once the vault is unlocked. */
    }
    this.loaded = loadedFromVault;
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
    root.setAttribute("data-locale", this.resolvedLocale);
    root.setAttribute("lang", this.resolvedLocale);
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
  setLocale(v: LocalePreference) {
    this.locale = v;
    this.apply();
    setDebounced(KEYS.locale, v);
  }

  reset() {
    void this.setAccent("violet");
    void this.setDensity("comfortable");
    void this.setFontScale("md");
    void this.setReduceMotion(false);
    void this.setFontFamily("sans");
    void this.setLocale("auto");
  }
}

export const appearance = new AppearanceStore();
