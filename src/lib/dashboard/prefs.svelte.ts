import * as settings from "$api/settings";
import { setDebounced } from "$api/debounced-settings";

export type DashboardSection =
  "kpis" | "spotlight" | "reports" | "activity" | "patients";
export type DashboardLimit = "spotlight" | "reports" | "activity" | "patients";

export const DASHBOARD_SECTIONS: ReadonlyArray<{
  id: DashboardSection;
  label: string;
  hint: string;
}> = [
  {
    id: "kpis",
    label: "Summary cards",
    hint: "Library-wide patient, report, abnormal, and critical totals.",
  },
  {
    id: "spotlight",
    label: "Recent abnormal flags",
    hint: "Patients with abnormal or critical results from the last 12 months.",
  },
  {
    id: "reports",
    label: "Recent reports",
    hint: "The newest imported reports and their extraction confidence.",
  },
  {
    id: "activity",
    label: "Recent activity",
    hint: "The latest audit-log events from this vault.",
  },
  {
    id: "patients",
    label: "Patients",
    hint: "Patient cards ordered by most recent report activity.",
  },
];

export type DashboardConfig = {
  order: DashboardSection[];
  visible: Record<DashboardSection, boolean>;
  spotlightLimit: number;
  reportLimit: number;
  activityLimit: number;
  patientLimit: number;
};

const KEY = "dashboard.config";
const SECTION_IDS = DASHBOARD_SECTIONS.map((section) => section.id);
const DEFAULT_CONFIG: DashboardConfig = {
  order: [...SECTION_IDS],
  visible: {
    kpis: true,
    spotlight: true,
    reports: true,
    activity: true,
    patients: true,
  },
  spotlightLimit: 6,
  reportLimit: 8,
  activityLimit: 6,
  patientLimit: 0,
};

function clamp(
  value: unknown,
  min: number,
  max: number,
  fallback: number,
): number {
  const n =
    typeof value === "number" && Number.isFinite(value)
      ? Math.round(value)
      : fallback;
  return Math.min(max, Math.max(min, n));
}

function normalize(raw: unknown): DashboardConfig {
  if (!raw || typeof raw !== "object") return cloneDefaults();
  const input = raw as Partial<DashboardConfig>;
  const suppliedOrder = Array.isArray(input.order) ? input.order : [];
  const order: DashboardSection[] = [];
  for (const id of suppliedOrder) {
    if (
      SECTION_IDS.includes(id as DashboardSection) &&
      !order.includes(id as DashboardSection)
    ) {
      order.push(id as DashboardSection);
    }
  }
  for (const id of SECTION_IDS) if (!order.includes(id)) order.push(id);

  const visible = { ...DEFAULT_CONFIG.visible };
  if (input.visible && typeof input.visible === "object") {
    for (const id of SECTION_IDS) {
      const value = (
        input.visible as Partial<Record<DashboardSection, unknown>>
      )[id];
      if (typeof value === "boolean") visible[id] = value;
    }
  }

  return {
    order,
    visible,
    spotlightLimit: clamp(
      input.spotlightLimit,
      1,
      50,
      DEFAULT_CONFIG.spotlightLimit,
    ),
    reportLimit: clamp(input.reportLimit, 1, 50, DEFAULT_CONFIG.reportLimit),
    activityLimit: clamp(
      input.activityLimit,
      1,
      50,
      DEFAULT_CONFIG.activityLimit,
    ),
    patientLimit: clamp(
      input.patientLimit,
      0,
      100,
      DEFAULT_CONFIG.patientLimit,
    ),
  };
}

function cloneDefaults(): DashboardConfig {
  return {
    order: [...DEFAULT_CONFIG.order],
    visible: { ...DEFAULT_CONFIG.visible },
    spotlightLimit: DEFAULT_CONFIG.spotlightLimit,
    reportLimit: DEFAULT_CONFIG.reportLimit,
    activityLimit: DEFAULT_CONFIG.activityLimit,
    patientLimit: DEFAULT_CONFIG.patientLimit,
  };
}

class DashboardPrefs {
  order = $state<DashboardSection[]>([...DEFAULT_CONFIG.order]);
  visible = $state<Record<DashboardSection, boolean>>({
    ...DEFAULT_CONFIG.visible,
  });
  spotlightLimit = $state(DEFAULT_CONFIG.spotlightLimit);
  reportLimit = $state(DEFAULT_CONFIG.reportLimit);
  activityLimit = $state(DEFAULT_CONFIG.activityLimit);
  patientLimit = $state(DEFAULT_CONFIG.patientLimit);
  loaded = $state(false);
  private loadPromise: Promise<void> | null = null;

  async load(): Promise<void> {
    if (this.loaded) return;
    if (this.loadPromise) return this.loadPromise;
    this.loadPromise = settings
      .get<unknown>(KEY)
      .then((raw) => {
        const config = normalize(raw);
        this.apply(config);
        this.loaded = true;
      })
      .catch(() => {
        this.loaded = true;
      })
      .finally(() => {
        this.loadPromise = null;
      });
    return this.loadPromise;
  }

  private apply(config: DashboardConfig): void {
    this.order = [...config.order];
    this.visible = { ...config.visible };
    this.spotlightLimit = config.spotlightLimit;
    this.reportLimit = config.reportLimit;
    this.activityLimit = config.activityLimit;
    this.patientLimit = config.patientLimit;
  }

  snapshot(): DashboardConfig {
    return {
      order: [...this.order],
      visible: { ...this.visible },
      spotlightLimit: this.spotlightLimit,
      reportLimit: this.reportLimit,
      activityLimit: this.activityLimit,
      patientLimit: this.patientLimit,
    };
  }

  private persist(): void {
    setDebounced(KEY, this.snapshot());
  }

  toggle(section: DashboardSection): void {
    this.visible = { ...this.visible, [section]: !this.visible[section] };
    this.persist();
  }

  move(section: DashboardSection, direction: -1 | 1): void {
    const index = this.order.indexOf(section);
    const target = index + direction;
    if (index < 0 || target < 0 || target >= this.order.length) return;
    const next = [...this.order];
    [next[index], next[target]] = [next[target], next[index]];
    this.order = next;
    this.persist();
  }

  setLimit(limit: DashboardLimit, value: number): void {
    if (limit === "spotlight")
      this.spotlightLimit = clamp(value, 1, 50, DEFAULT_CONFIG.spotlightLimit);
    if (limit === "reports")
      this.reportLimit = clamp(value, 1, 50, DEFAULT_CONFIG.reportLimit);
    if (limit === "activity")
      this.activityLimit = clamp(value, 1, 50, DEFAULT_CONFIG.activityLimit);
    if (limit === "patients")
      this.patientLimit = clamp(value, 0, 100, DEFAULT_CONFIG.patientLimit);
    this.persist();
  }

  reset(): void {
    this.apply(cloneDefaults());
    this.persist();
  }
}

export const dashboardPrefs = new DashboardPrefs();
