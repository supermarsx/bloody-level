// Compare-page preset registry.
//
// Two flavours of preset:
//   - **Bundled** — shipped with the app (Iron panel, Lipids, …). Editable
//     by the user, who gets a "reset to defaults" button if they regret
//     their changes. Their `id` is reserved so we can recognise overrides.
//   - **Dynamic** — patient-aware (Abnormal, Subclinical). Their analyte
//     list is populated by a backend query at runtime; the user can't
//     edit them, but they can still attach filters to them so e.g.
//     "Abnormal in the last 12 months" is one click.
//
// User-created presets are saved encrypted alongside the rest of the app
// preferences in the settings table under `compare.user_presets` (a JSON
// blob). Bundled-preset overrides go under `compare.bundled_overrides`
// keyed by the bundled preset id; deletes from there yields the bundled
// default again.

import * as settings from "$api/settings";
import { setDebounced } from "$api/debounced-settings";
import { t } from "$lib/i18n/index.svelte";

export type HrtFilter = "all" | "pre" | "post";

/** Every per-card filter the Compare page exposes. Mirrors the script
 *  variables on /compare so the round-trip is loss-free. */
export interface ComparePresetFilters {
  dateFromIso: string;
  dateUntilIso: string;
  valueMin: string;
  valueMax: string;
  includeInlinePriors: boolean;
  onlyAbnormal: boolean;
  onlyCritical: boolean;
  onlyNormal: boolean;
  onlyUnflagged: boolean;
  lastNPerAnalyte: number;
  excludeReportIds: string;
  intersectOnly: boolean;
  hrtFilter: HrtFilter;
}

export const EMPTY_FILTERS: ComparePresetFilters = {
  dateFromIso: "",
  dateUntilIso: "",
  valueMin: "",
  valueMax: "",
  includeInlinePriors: false,
  onlyAbnormal: false,
  onlyCritical: false,
  onlyNormal: false,
  onlyUnflagged: false,
  lastNPerAnalyte: 0,
  excludeReportIds: "",
  intersectOnly: false,
  hrtFilter: "all",
};

/** Static preset — fixed analyte list, optional filter overrides. */
export type StaticPreset = {
  kind: "static";
  id: string;
  name: string;
  ids: string[];
  /** When `null` (default), applying this preset doesn't touch filters.
   *  When set, every filter is reset to its empty-default and then this
   *  object's keys are applied on top. */
  filters: Partial<ComparePresetFilters> | null;
  source: "bundled" | "user";
  hint?: string;
};

/** Dynamic preset — analyte list resolved per-patient at runtime. The
 *  set of dynamic sources is closed (we can't ship arbitrary new ones
 *  without backend support), so users can't create them — only attach
 *  filters. */
export type DynamicPreset = {
  kind: "dynamic";
  id: string;
  name: string;
  source: "bundled" | "user";
  /** Identifier the Compare page resolves into the actual analyte ID
   *  list at runtime. */
  dataSource: "abnormal" | "subclinical";
  filters: Partial<ComparePresetFilters> | null;
  hint: string;
};

export type ComparePreset = StaticPreset | DynamicPreset;

// ─── Bundled defaults ─────────────────────────────────────────────────────

const BUNDLED_PRESETS: ComparePreset[] = [
  {
    kind: "static",
    id: "iron",
    name: "Iron panel",
    ids: [
      "ferritina",
      "ferro_serico",
      "transferrina",
      "saturacao_transferrina",
    ],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "lipids",
    name: "Lipids",
    ids: ["colesterol_total", "hdl", "ldl", "trigliceridos"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "thyroid",
    name: "Thyroid",
    ids: ["tsh", "t4_livre", "t3_livre"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "liver",
    name: "Liver",
    ids: ["alt", "ast", "gama_gt", "fosfatase_alcalina"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "kidney",
    name: "Kidney",
    ids: ["creatinina", "ureia", "tfg_estimada"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "hemogram",
    name: "Hemogram",
    ids: ["hemoglobina", "hematocrito", "eritrocitos", "plaquetas"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "hematology",
    name: "Hematology",
    ids: [
      "hemoglobina",
      "hematocrito",
      "eritrocitos",
      "vgm",
      "hgm",
      "cmhg",
      "rdw",
      "leucocitos",
      "neutrofilos",
      "linfocitos",
      "monocitos",
      "eosinofilos",
      "basofilos",
      "plaquetas",
      "reticulocitos",
    ],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "inflammation",
    name: "Inflammation",
    ids: ["proteina_c_reactiva", "velocidade_sedimentacao"],
    filters: null,
    source: "bundled",
  },
  {
    kind: "static",
    id: "hrt_e2_t",
    name: "HRT (E2/T)",
    ids: [
      "estradiol",
      "testosterona_total",
      "testosterona_livre",
      "shbg",
      "prolactina",
    ],
    filters: null,
    source: "bundled",
  },
  {
    kind: "dynamic",
    id: "abnormal",
    name: "Abnormal",
    dataSource: "abnormal",
    filters: null,
    source: "bundled",
    hint: "Every analyte with at least one low / high / critical reading on file.",
  },
  {
    kind: "dynamic",
    id: "subclinical",
    name: "Subclinical",
    dataSource: "subclinical",
    filters: null,
    source: "bundled",
    hint: "Latest reading flagged normal but sitting in the bottom or top decile of its printed reference range.",
  },
];

const KEY_USER_PRESETS = "compare.user_presets";
const KEY_BUNDLED_OVERRIDES = "compare.bundled_overrides";

/** Editable subset of a preset — what the editor mutates. The kind /
 *  dataSource / source are immutable and not exposed. */
export type EditablePreset = {
  name: string;
  ids: string[]; // ignored for dynamic
  filters: Partial<ComparePresetFilters> | null; // null = don't touch filters
};

class ComparePresets {
  /** Bundled preset overrides keyed by bundled id. When present they
   *  REPLACE the bundled fields except `kind`/`dataSource`/`source`. */
  bundledOverrides = $state<Record<string, EditablePreset>>({});
  /** User-created presets — always `kind: static`. */
  userPresets = $state<StaticPreset[]>([]);
  loaded = $state(false);

  async load() {
    if (this.loaded) return;
    try {
      const overrides = await settings.get<Record<string, EditablePreset>>(
        KEY_BUNDLED_OVERRIDES,
      );
      if (overrides && typeof overrides === "object") {
        this.bundledOverrides = overrides;
      }
      const user = await settings.get<StaticPreset[]>(KEY_USER_PRESETS);
      if (Array.isArray(user)) {
        // Ensure every user-loaded entry has the right shape; defensively
        // drop anything malformed so a corrupt save doesn't crash the page.
        this.userPresets = user
          .filter(
            (p) =>
              p &&
              typeof p === "object" &&
              p.kind === "static" &&
              typeof p.id === "string" &&
              typeof p.name === "string",
          )
          .map((p) => ({
            kind: "static",
            id: p.id,
            name: p.name,
            ids: Array.isArray(p.ids)
              ? p.ids.filter((x) => typeof x === "string")
              : [],
            filters:
              p.filters && typeof p.filters === "object" ? p.filters : null,
            source: "user",
            hint: p.hint,
          }));
      }
    } catch {
      /* stay on defaults */
    }
    this.loaded = true;
  }

  /** Merged view: bundled (with overrides applied) followed by user. */
  get presets(): ComparePreset[] {
    const merged: ComparePreset[] = BUNDLED_PRESETS.map((b) => {
      const ov = this.bundledOverrides[b.id];
      if (!ov) {
        return {
          ...b,
          name: t(b.name),
          ...(b.kind === "dynamic" ? { hint: t(b.hint) } : {}),
        };
      }
      if (b.kind === "static") {
        return {
          ...b,
          name: ov.name ?? t(b.name),
          ids: Array.isArray(ov.ids) ? ov.ids : b.ids,
          filters: ov.filters !== undefined ? ov.filters : b.filters,
        };
      }
      // Dynamic — user can rename + override filters; the dataSource and
      // analyte list stay derived at runtime.
      return {
        ...b,
        name: ov.name ?? t(b.name),
        hint: t(b.hint),
        filters: ov.filters !== undefined ? ov.filters : b.filters,
      };
    });
    return [...merged, ...this.userPresets];
  }

  /** Used by the editor to know whether a bundled preset has been
   *  customised so we can show a "reset" affordance. */
  isBundledOverridden(id: string): boolean {
    return id in this.bundledOverrides;
  }

  /** Replace a bundled preset's editable fields. Persists the diff so
   *  reload-defaults restores the original. */
  setBundledOverride(id: string, override: EditablePreset) {
    const next = { ...this.bundledOverrides, [id]: override };
    this.bundledOverrides = next;
    setDebounced(KEY_BUNDLED_OVERRIDES, next);
  }

  /** Reset a single bundled preset to its bundled defaults. */
  clearBundledOverride(id: string) {
    if (!(id in this.bundledOverrides)) return;
    const next = { ...this.bundledOverrides };
    delete next[id];
    this.bundledOverrides = next;
    setDebounced(KEY_BUNDLED_OVERRIDES, next);
  }

  /** Wipe every override — every bundled preset reverts to the bundled
   *  defaults. User-created presets are untouched. */
  resetAllBundled() {
    this.bundledOverrides = {};
    setDebounced(KEY_BUNDLED_OVERRIDES, {});
  }

  addUserPreset(p: {
    name: string;
    ids: string[];
    filters: Partial<ComparePresetFilters> | null;
    hint?: string;
  }): StaticPreset {
    const baseId = `user_${slug(p.name) || "preset"}`;
    let id = baseId;
    let n = 2;
    while (this.userPresets.some((x) => x.id === id)) {
      id = `${baseId}_${n++}`;
    }
    const created: StaticPreset = {
      kind: "static",
      id,
      name: p.name.trim() || t("Untitled"),
      ids: p.ids.filter((x) => typeof x === "string"),
      filters: p.filters,
      source: "user",
      hint: p.hint,
    };
    this.userPresets = [...this.userPresets, created];
    setDebounced(KEY_USER_PRESETS, this.userPresets);
    return created;
  }

  updateUserPreset(id: string, edit: EditablePreset) {
    const next = this.userPresets.map((p) =>
      p.id === id
        ? {
            ...p,
            name: edit.name.trim() || p.name,
            ids: edit.ids.filter((x) => typeof x === "string"),
            filters: edit.filters,
          }
        : p,
    );
    this.userPresets = next;
    setDebounced(KEY_USER_PRESETS, next);
  }

  removeUserPreset(id: string) {
    this.userPresets = this.userPresets.filter((p) => p.id !== id);
    setDebounced(KEY_USER_PRESETS, this.userPresets);
  }
}

function slug(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFKD")
    .replace(/[^\w\s-]/g, "")
    .trim()
    .replace(/\s+/g, "_")
    .slice(0, 32);
}

export const comparePresets = new ComparePresets();
