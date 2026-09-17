<script lang="ts">
  import { onMount, tick } from 'svelte';
  import TimeSeries from '$charts/time-series.svelte';
  import { listPatients, analyteTimeseries, listFlaggedAnalytes,
           type PatientSummary, type AnalyteReading, type FlaggedAnalytesResult } from '$api/reports';
  import { listOntologyEntries, type AnalyteOntologyEntry } from '$api/analyte-info';
  import { defaultRefFor } from '$format/default-ref';
  import { prettyUnit } from '$format/units';
  import { LocalChartPrefs, type ChartPrefsLike, chartPrefs } from '$charts/prefs.svelte';
  import { comparePresets, EMPTY_FILTERS, type ComparePreset } from '$charts/compare-presets.svelte';
  import { toasts } from '../../lib/toasts/store.svelte';

  // The Compare view does multi-analyte overlay for a single patient.
  // Cross-patient comparison of the same analyte is intentionally out of scope:
  // the analyte page already supports a "All patients" picker, and stacking
  // patients on top of each other invites privacy / context bleed.

  let patients = $state<PatientSummary[]>([]);
  let analytes = $state<AnalyteOntologyEntry[]>([]);

  let patientId = $state<string>('');
  let patientSearch = $state('');
  let patientMenuOpen = $state(false);
  let patientActiveIndex = $state(-1);
  let patientPickerElement = $state<HTMLElement | null>(null);
  let patientSearchElement = $state<HTMLInputElement | null>(null);
  // Ordered list of selected analytes — array (not Set) so the user can
  // drag / arrow-button to reorder. The picker derives a Set view from
  // this for O(1) "is selected" checks.
  let pickedOrder = $state<string[]>([]);
  const pickedAnalyteIds = $derived(new Set(pickedOrder));
  let analyteFilter = $state('');

  let readingsByAnalyte = $state<Record<string, AnalyteReading[]>>({});
  let loading = $state(false);

  // ─────────────────────────────────────────────────────────────────────
  //  Compare-wide filters — applied to every card before rendering.
  //  Lives in plain $state objects so toggling propagates instantly.
  // ─────────────────────────────────────────────────────────────────────
  let filtersOpen = $state(false);
  let dateFromIso = $state('');
  let dateUntilIso = $state('');
  let valueMin = $state<string>('');
  let valueMax = $state<string>('');
  let includeInlinePriors = $state(false);
  // Flag filter — when any of these is checked, ONLY readings carrying
  // that flag survive. All four off = no flag filter at all.
  let onlyAbnormal = $state(false);
  let onlyCritical = $state(false);
  let onlyNormal   = $state(false);
  let onlyUnflagged = $state(false);
  // Restrict to the LAST N readings per analyte after the other filters
  // run. 0 = no cap. Useful for taming dense long-history series so the
  // signal-to-noise reads cleanly.
  let lastNPerAnalyte = $state<number>(0);
  // Excluded specific report IDs. Comma- or whitespace-separated.
  let excludeReportIds = $state('');
  // Auto-clip to the date span where every selected analyte has at least
  // one reading. Useful for genuinely-overlapping comparison.
  let intersectOnly = $state(false);
  // HRT-start filter — only meaningful when the active patient has a
  // hrt_start_iso anchor. `all` (default) ignores the anchor entirely;
  // `pre` keeps readings strictly before the anchor; `post` keeps the
  // anchor day onwards.
  let hrtFilter = $state<'all' | 'pre' | 'post'>('all');

  // Per-card prefs registry — lazily populated when a user enables
  // "isolated settings" on a card. Keyed by analyte ID.
  let localPrefsByAnalyte = $state<Record<string, LocalChartPrefs>>({});
  let isolatedAnalyteIds  = $state<Set<string>>(new Set());

  // ─────────────────────────────────────────────────────────────────────
  //  Persistence — every meaningful UI bit of the Compare page is mirrored
  //  to localStorage so a reload / route round-trip puts the user back
  //  exactly where they were. Keyed under one stable namespace; bump the
  //  version suffix when the schema changes incompatibly so old saves
  //  don't crash newer versions.
  // ─────────────────────────────────────────────────────────────────────
  const STORAGE_KEY = 'compare:state:v1';

  /** The fields LocalChartPrefs persists. We dump them to a plain object
   *  on save and re-apply them via setters on restore. Keep in sync with
   *  the LocalChartPrefs class definition. */
  type LocalPrefsSnapshot = {
    showXSlider: boolean; showYSlider: boolean; scrollZoom: boolean;
    referenceSource: 'auto' | 'library' | 'printed';
    showValues: boolean; smooth: boolean; scale: 'linear' | 'log';
    showBands: boolean; useNicknames: boolean; showSymbols: boolean;
    lineWidth: number; symbolSize: number;
    gridDensity: 'off' | 'standard' | 'detailed';
    dateFormat: 'iso' | 'short' | 'monthYear' | 'numeric';
    xAxisPadPct: number; yAxisPadPct: number; yVisiblePadPct: number;
    showTodayLine: boolean; showTrendLine: boolean;
    showMeanLine: boolean; showMinMaxMarkers: boolean;
    seriesType: 'line' | 'area' | 'bar';
    step: 'none' | 'start' | 'middle' | 'end';
    colorByFlag: boolean; connectNulls: boolean;
    bandOpacityPct: number;
    animation: 'off' | 'fast' | 'normal' | 'slow';
    tooltipMode: 'axis' | 'item';
    showTitle: boolean;
    timeWindow: 'all' | '30d' | '90d' | '6m' | '1y' | '2y' | '5y';
  };

  function snapshotLocalPrefs(lp: LocalChartPrefs): LocalPrefsSnapshot {
    return {
      showXSlider: lp.showXSlider, showYSlider: lp.showYSlider, scrollZoom: lp.scrollZoom,
      referenceSource: lp.referenceSource,
      showValues: lp.showValues, smooth: lp.smooth, scale: lp.scale,
      showBands: lp.showBands, useNicknames: lp.useNicknames, showSymbols: lp.showSymbols,
      lineWidth: lp.lineWidth, symbolSize: lp.symbolSize,
      gridDensity: lp.gridDensity, dateFormat: lp.dateFormat,
      xAxisPadPct: lp.xAxisPadPct, yAxisPadPct: lp.yAxisPadPct, yVisiblePadPct: lp.yVisiblePadPct,
      showTodayLine: lp.showTodayLine, showTrendLine: lp.showTrendLine,
      showMeanLine: lp.showMeanLine, showMinMaxMarkers: lp.showMinMaxMarkers,
      seriesType: lp.seriesType, step: lp.step,
      colorByFlag: lp.colorByFlag, connectNulls: lp.connectNulls,
      bandOpacityPct: lp.bandOpacityPct, animation: lp.animation,
      tooltipMode: lp.tooltipMode, showTitle: lp.showTitle,
      timeWindow: lp.timeWindow,
    };
  }

  function applyLocalPrefsSnapshot(lp: LocalChartPrefs, snap: Partial<LocalPrefsSnapshot>) {
    if (snap.showXSlider     !== undefined) lp.setX(snap.showXSlider);
    if (snap.showYSlider     !== undefined) lp.setY(snap.showYSlider);
    if (snap.scrollZoom      !== undefined) lp.setScrollZoom(snap.scrollZoom);
    if (snap.referenceSource !== undefined) lp.setReferenceSource(snap.referenceSource);
    if (snap.showValues      !== undefined) lp.setShowValues(snap.showValues);
    if (snap.smooth          !== undefined) lp.setSmooth(snap.smooth);
    if (snap.scale           !== undefined) lp.setScale(snap.scale);
    if (snap.showBands       !== undefined) lp.setShowBands(snap.showBands);
    if (snap.useNicknames    !== undefined) lp.setUseNicknames(snap.useNicknames);
    if (snap.showSymbols     !== undefined) lp.setShowSymbols(snap.showSymbols);
    if (snap.lineWidth       !== undefined) lp.setLineWidth(snap.lineWidth);
    if (snap.symbolSize      !== undefined) lp.setSymbolSize(snap.symbolSize);
    if (snap.gridDensity     !== undefined) lp.setGridDensity(snap.gridDensity);
    if (snap.dateFormat      !== undefined) lp.setDateFormat(snap.dateFormat);
    if (snap.xAxisPadPct     !== undefined) lp.setXAxisPadPct(snap.xAxisPadPct);
    if (snap.yAxisPadPct     !== undefined) lp.setYAxisPadPct(snap.yAxisPadPct);
    if (snap.yVisiblePadPct  !== undefined) lp.setYVisiblePadPct(snap.yVisiblePadPct);
    if (snap.showTodayLine   !== undefined) lp.setShowTodayLine(snap.showTodayLine);
    if (snap.showTrendLine   !== undefined) lp.setShowTrendLine(snap.showTrendLine);
    if (snap.showMeanLine    !== undefined) lp.setShowMeanLine(snap.showMeanLine);
    if (snap.showMinMaxMarkers !== undefined) lp.setShowMinMaxMarkers(snap.showMinMaxMarkers);
    if (snap.seriesType      !== undefined) lp.setSeriesType(snap.seriesType);
    if (snap.step            !== undefined) lp.setStep(snap.step);
    if (snap.colorByFlag     !== undefined) lp.setColorByFlag(snap.colorByFlag);
    if (snap.connectNulls    !== undefined) lp.setConnectNulls(snap.connectNulls);
    if (snap.bandOpacityPct  !== undefined) lp.setBandOpacityPct(snap.bandOpacityPct);
    if (snap.animation       !== undefined) lp.setAnimation(snap.animation);
    if (snap.tooltipMode     !== undefined) lp.setTooltipMode(snap.tooltipMode);
    if (snap.showTitle       !== undefined) lp.setShowTitle(snap.showTitle);
    if (snap.timeWindow      !== undefined) lp.setTimeWindow(snap.timeWindow);
  }

  // True after the initial load from storage has run, so the persistence
  // effect doesn't write defaults over the stored snapshot before it's
  // had a chance to be applied.
  let stateHydrated = $state(false);

  function toggleIsolatedFor(id: string) {
    if (isolatedAnalyteIds.has(id)) {
      isolatedAnalyteIds.delete(id);
    } else {
      isolatedAnalyteIds.add(id);
      // Snapshot from the current globals so the user starts from where
      // they were rather than the bundled defaults.
      if (!localPrefsByAnalyte[id]) {
        const next = { ...localPrefsByAnalyte };
        next[id] = new LocalChartPrefs();
        localPrefsByAnalyte = next;
      }
    }
    isolatedAnalyteIds = new Set(isolatedAnalyteIds);
  }

  function resyncIsolatedFromGlobal(id: string) {
    const lp = localPrefsByAnalyte[id];
    if (lp) lp.syncFromGlobal();
  }

  function prefsFor(id: string): ChartPrefsLike {
    return isolatedAnalyteIds.has(id) ? localPrefsByAnalyte[id] : chartPrefs;
  }

  // ─────────────────────────────────────────────────────────────────────

  onMount(async () => {
    try {
      [patients, analytes] = await Promise.all([listPatients(), listOntologyEntries()]);
      // Restore persisted state if any. We do this AFTER fetching the
      // patients + ontology so we can validate IDs against current data
      // (a deleted patient or removed analyte just falls out of the
      // restored list rather than producing a confusing empty card).
      const restored = loadPersisted();
      if (restored && patients.some((p) => p.id === restored.patientId)) {
        patientId        = restored.patientId;
        const known      = new Set(analytes.map((a) => a.id));
        pickedOrder      = restored.pickedOrder.filter((id) => known.has(id));
        analyteFilter    = restored.analyteFilter;
        // Filters
        dateFromIso          = restored.filters.dateFromIso;
        dateUntilIso         = restored.filters.dateUntilIso;
        valueMin             = restored.filters.valueMin;
        valueMax             = restored.filters.valueMax;
        includeInlinePriors  = restored.filters.includeInlinePriors;
        onlyAbnormal         = restored.filters.onlyAbnormal;
        onlyCritical         = restored.filters.onlyCritical;
        onlyNormal           = restored.filters.onlyNormal;
        onlyUnflagged        = restored.filters.onlyUnflagged;
        lastNPerAnalyte      = restored.filters.lastNPerAnalyte;
        excludeReportIds     = restored.filters.excludeReportIds;
        intersectOnly        = restored.filters.intersectOnly;
        hrtFilter            = restored.filters.hrtFilter;
        filtersOpen          = restored.filtersOpen;
        // Per-card isolated prefs — only restore the cards still in
        // pickedOrder. The user hasn't opened cards they haven't picked.
        const isolated = new Set<string>();
        const lpMap: Record<string, LocalChartPrefs> = {};
        for (const id of pickedOrder) {
          const snap = restored.localPrefs[id];
          if (!snap) continue;
          const lp = new LocalChartPrefs();
          applyLocalPrefsSnapshot(lp, snap);
          lpMap[id] = lp;
          isolated.add(id);
        }
        localPrefsByAnalyte = lpMap;
        isolatedAnalyteIds  = isolated;
      } else if (patients.length > 0) {
        // Default to the most recent patient when nothing was persisted
        // or the persisted patient no longer exists.
        patientId = [...patients].sort((a, b) =>
          (b.latest_collection_date_iso ?? '').localeCompare(a.latest_collection_date_iso ?? '')
        )[0].id;
      }
    } catch (e) {
      toasts.error(e);
    } finally {
      // Flip the gate AFTER the restore completes so the persistence
      // effect doesn't fire on the initial setter chain above.
      stateHydrated = true;
    }
  });

  type Persisted = {
    patientId: string;
    pickedOrder: string[];
    analyteFilter: string;
    filtersOpen: boolean;
    filters: {
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
      hrtFilter: 'all' | 'pre' | 'post';
    };
    localPrefs: Record<string, LocalPrefsSnapshot>;
  };

  function loadPersisted(): Persisted | null {
    if (typeof window === 'undefined') return null;
    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (!raw) return null;
      const parsed = JSON.parse(raw) as Persisted;
      // Cheap shape check — bail if the basic fields aren't there so a
      // corrupt save doesn't crash the page.
      if (typeof parsed?.patientId !== 'string' || !Array.isArray(parsed?.pickedOrder)) {
        return null;
      }
      return parsed;
    } catch {
      return null;
    }
  }

  function snapshotState(): Persisted {
    const localPrefs: Record<string, LocalPrefsSnapshot> = {};
    for (const id of isolatedAnalyteIds) {
      const lp = localPrefsByAnalyte[id];
      if (lp) localPrefs[id] = snapshotLocalPrefs(lp);
    }
    return {
      patientId,
      pickedOrder,
      analyteFilter,
      filtersOpen,
      filters: {
        dateFromIso, dateUntilIso, valueMin, valueMax,
        includeInlinePriors,
        onlyAbnormal, onlyCritical, onlyNormal, onlyUnflagged,
        lastNPerAnalyte, excludeReportIds, intersectOnly,
        hrtFilter,
      },
      localPrefs,
    };
  }

  // Debounced writer — every UI change schedules a save in 250ms; rapid
  // typing in the search / exclude fields therefore writes once, not
  // once-per-keystroke. localStorage writes are synchronous and cheap,
  // but coalescing keeps Settings and DevTools tidy.
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  function schedulePersist() {
    if (typeof window === 'undefined') return;
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      try {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshotState()));
      } catch { /* quota exceeded etc. — non-critical */ }
    }, 250);
  }

  // One effect that touches every persistable field so any change fires
  // the save scheduler. We also touch the local-prefs snapshots manually
  // because $state objects nested inside a Record don't propagate change
  // events through the parent automatically.
  $effect(() => {
    void patientId; void pickedOrder; void analyteFilter; void filtersOpen;
    void dateFromIso; void dateUntilIso; void valueMin; void valueMax;
    void includeInlinePriors;
    void onlyAbnormal; void onlyCritical; void onlyNormal; void onlyUnflagged;
    void lastNPerAnalyte; void excludeReportIds; void intersectOnly; void hrtFilter;
    void isolatedAnalyteIds;
    // Touch every isolated card's local prefs so toolbar tweaks persist.
    for (const id of isolatedAnalyteIds) {
      const lp = localPrefsByAnalyte[id];
      if (!lp) continue;
      void lp.showValues; void lp.smooth; void lp.scale; void lp.showBands;
      void lp.useNicknames; void lp.showSymbols; void lp.lineWidth; void lp.symbolSize;
      void lp.gridDensity; void lp.dateFormat;
      void lp.xAxisPadPct; void lp.yAxisPadPct; void lp.yVisiblePadPct;
      void lp.showTodayLine; void lp.showTrendLine;
      void lp.showMeanLine; void lp.showMinMaxMarkers;
      void lp.seriesType; void lp.step; void lp.colorByFlag; void lp.connectNulls;
      void lp.bandOpacityPct; void lp.animation;
      void lp.tooltipMode; void lp.showTitle; void lp.timeWindow;
      void lp.showXSlider; void lp.showYSlider; void lp.scrollZoom;
    }
    if (stateHydrated) schedulePersist();
  });

  // Disable the Clear-all button when there's literally nothing to clear,
  // so it's never a confusing no-op. Inlines the filter-active check
  // because `filterActive` is declared further down in the file.
  const hasAnyState = $derived(
    pickedOrder.length > 0
    || filtersOpen
    || isolatedAnalyteIds.size > 0
    || analyteFilter !== ''
    || !!(dateFromIso || dateUntilIso || valueMin !== '' || valueMax !== ''
          || includeInlinePriors
          || onlyAbnormal || onlyCritical || onlyNormal || onlyUnflagged
          || lastNPerAnalyte > 0 || excludeReportIds.trim() || intersectOnly
          || hrtFilter !== 'all')
  );

  async function onClearAll() {
    if (!hasAnyState) return;
    const ok = await confirmAsk(
      'Clear every picked analyte, every filter, and every per-card override?\n\n' +
      'The current patient selection stays.'
    );
    if (!ok) return;
    clearAll();
    toasts.success('Compare reset', 'All picks, filters, and overrides cleared.');
  }

  // Tiny wrapper around the Tauri ask dialog so the rest of the file
  // doesn't have to import + alias it. Returns false on dialog failure
  // so the user is never auto-confirmed when the dialog plugin glitches.
  async function confirmAsk(message: string): Promise<boolean> {
    try {
      const { ask } = await import('@tauri-apps/plugin-dialog');
      return await ask(message, { title: 'Clear Compare state', kind: 'warning' });
    } catch {
      return false;
    }
  }

  /** Reset every Compare-page bit of state and wipe the saved snapshot. */
  function clearAll() {
    pickedOrder = [];
    analyteFilter = '';
    filtersOpen = false;
    dateFromIso = '';
    dateUntilIso = '';
    valueMin = '';
    valueMax = '';
    includeInlinePriors = false;
    onlyAbnormal = false;
    onlyCritical = false;
    onlyNormal = false;
    onlyUnflagged = false;
    lastNPerAnalyte = 0;
    excludeReportIds = '';
    intersectOnly = false;
    hrtFilter = 'all';
    isolatedAnalyteIds = new Set();
    localPrefsByAnalyte = {};
    readingsByAnalyte = {};
    if (typeof window !== 'undefined') {
      try { window.localStorage.removeItem(STORAGE_KEY); } catch { /* */ }
    }
  }

  const activePatient = $derived(patients.find((p) => p.id === patientId) ?? null);

  const filteredPatients = $derived.by(() => {
    const q = patientSearch.trim().toLowerCase();
    if (!q) return patients;
    return patients.filter((p) =>
      p.display_name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q)
    );
  });

  async function openPatientMenu() {
    patientMenuOpen = true;
    patientActiveIndex = -1;
    await tick();
    patientSearchElement?.focus();
  }

  function closePatientMenu() {
    patientMenuOpen = false;
    patientActiveIndex = -1;
  }

  function selectPatient(id: string) {
    patientId = id;
    patientSearch = '';
    closePatientMenu();
  }

  function onPatientPickerWindowClick(event: MouseEvent) {
    if (patientMenuOpen && patientPickerElement && !patientPickerElement.contains(event.target as Node)) {
      closePatientMenu();
    }
  }

  function onPatientSearchKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      closePatientMenu();
      return;
    }
    if (event.key === 'ArrowDown' && filteredPatients.length > 0) {
      event.preventDefault();
      patientActiveIndex = (patientActiveIndex + 1) % filteredPatients.length;
      return;
    }
    if (event.key === 'ArrowUp' && filteredPatients.length > 0) {
      event.preventDefault();
      patientActiveIndex = patientActiveIndex <= 0
        ? filteredPatients.length - 1
        : patientActiveIndex - 1;
      return;
    }
    if (event.key === 'Enter') {
      const patient = filteredPatients[patientActiveIndex >= 0 ? patientActiveIndex : 0];
      if (patient) {
        event.preventDefault();
        selectPatient(patient.id);
      }
    }
  }

  const filteredAnalytes = $derived.by(() => {
    const q = analyteFilter.trim().toLowerCase();
    return analytes
      .filter((a) => !a.is_panel_header)
      .filter((a) => !q || a.pt_name.toLowerCase().includes(q) || a.id.toLowerCase().includes(q))
      .sort((a, b) => {
        // Picked first, then by usage, then alphabetical.
        const pa = pickedAnalyteIds.has(a.id) ? 0 : 1;
        const pb = pickedAnalyteIds.has(b.id) ? 0 : 1;
        if (pa !== pb) return pa - pb;
        if (a.result_count !== b.result_count) return b.result_count - a.result_count;
        return a.pt_name.localeCompare(b.pt_name);
      });
  });

  function toggleAnalyte(id: string) {
    pickedOrder = pickedOrder.includes(id)
      ? pickedOrder.filter((x) => x !== id)
      : [...pickedOrder, id];
  }

  function clearPicked() {
    pickedOrder = [];
    readingsByAnalyte = {};
  }

  /** Move the analyte at `idx` by `delta` positions in the picked-order
   *  array. Gracefully clamps at both ends so the buttons can stay
   *  enabled at the boundaries without out-of-range mutations. */
  function moveAnalyte(idx: number, delta: number) {
    const target = idx + delta;
    if (target < 0 || target >= pickedOrder.length || target === idx) return;
    const next = [...pickedOrder];
    const [moved] = next.splice(idx, 1);
    next.splice(target, 0, moved);
    pickedOrder = next;
  }

  // ── Drag-to-reorder state ──────────────────────────────────────────
  // The dragged card writes its index into `dragSrcIdx`; cards being
  // hovered set `dragOverIdx` so the UI can highlight the drop target.
  // On drop we splice the source out and insert it at the over-index.
  let dragSrcIdx = $state<number | null>(null);
  let dragOverIdx = $state<number | null>(null);

  function onDragStart(idx: number) { dragSrcIdx = idx; }
  function onDragEnter(idx: number) { if (dragSrcIdx != null && idx !== dragSrcIdx) dragOverIdx = idx; }
  function onDragOver(e: DragEvent)  { if (dragSrcIdx != null) e.preventDefault(); }
  function onDrop(idx: number) {
    if (dragSrcIdx == null || dragSrcIdx === idx) {
      dragSrcIdx = null; dragOverIdx = null;
      return;
    }
    const next = [...pickedOrder];
    const [moved] = next.splice(dragSrcIdx, 1);
    // Inserting after a removal at a lower index shifts everything; the
    // target index doesn't need adjustment because splice already
    // accounts for it.
    next.splice(idx, 0, moved);
    pickedOrder = next;
    dragSrcIdx = null; dragOverIdx = null;
  }
  function onDragEnd() { dragSrcIdx = null; dragOverIdx = null; }

  function clearFilters() {
    dateFromIso = '';
    dateUntilIso = '';
    valueMin = '';
    valueMax = '';
    includeInlinePriors = false;
    onlyAbnormal = false;
    onlyCritical = false;
    onlyNormal = false;
    onlyUnflagged = false;
    lastNPerAnalyte = 0;
    excludeReportIds = '';
    intersectOnly = false;
    hrtFilter = 'all';
  }

  // Re-fetch readings whenever the patient or picked analytes change.
  $effect(() => {
    void patientId;
    void pickedOrder;
    if (!patientId || pickedOrder.length === 0) {
      readingsByAnalyte = {};
      return;
    }
    void load();
  });

  async function load() {
    if (!patientId || pickedOrder.length === 0) return;
    loading = true;
    try {
      // Only fetch analytes we don't already have cached for this patient,
      // but issue all the requests we need in parallel.
      const ids = pickedOrder;
      const results = await Promise.all(ids.map((id) => analyteTimeseries(id, patientId)));
      const map: Record<string, AnalyteReading[]> = {};
      ids.forEach((id, i) => { map[id] = results[i]; });
      readingsByAnalyte = map;
    } catch (e) {
      toasts.error(e);
    } finally {
      loading = false;
    }
  }

  // Parse the comma/whitespace-separated exclude list once.
  const excludedReportSet = $derived.by(() => {
    const set = new Set<string>();
    for (const tok of excludeReportIds.split(/[\s,]+/)) {
      const t = tok.trim();
      if (t) set.add(t);
    }
    return set;
  });

  // The set of "active" flag categories the user is filtering to. When
  // all are off, no flag filter applies. ECharts itself doesn't care; we
  // gate at the data layer above so trend / mean / count statistics also
  // honour the filter.
  const flagFilter = $derived.by(() => {
    const wanted = new Set<string>();
    if (onlyNormal)    wanted.add('normal');
    if (onlyAbnormal) { wanted.add('low'); wanted.add('high'); wanted.add('abnormal_qual'); }
    if (onlyCritical) { wanted.add('critical_low'); wanted.add('critical_high'); }
    return { active: wanted.size > 0 || onlyUnflagged, wanted, includeUnflagged: onlyUnflagged };
  });

  /** Apply every compare-wide filter to a single analyte's readings. */
  function applyFilters(rows: AnalyteReading[]): AnalyteReading[] {
    const fromTs  = dateFromIso  ? Date.parse(dateFromIso  + 'T00:00:00') : Number.NEGATIVE_INFINITY;
    const untilTs = dateUntilIso ? Date.parse(dateUntilIso + 'T23:59:59') : Number.POSITIVE_INFINITY;
    const vMin = valueMin === '' ? Number.NEGATIVE_INFINITY : parseFloat(valueMin);
    const vMax = valueMax === '' ? Number.POSITIVE_INFINITY : parseFloat(valueMax);
    // HRT anchor (millis). Used only when the patient has hrt_start_iso
    // set AND the filter is not 'all'; otherwise it short-circuits below.
    const hrtTs = activePatient?.hrt_start_iso
      ? Date.parse(activePatient.hrt_start_iso + 'T00:00:00')
      : null;

    let out = rows.filter((r) => {
      if (r.value == null) return false;
      if (!includeInlinePriors && r.inline_prior) return false;
      const t = Date.parse(r.date);
      if (Number.isFinite(t)) {
        if (t < fromTs)  return false;
        if (t > untilTs) return false;
      }
      const v = r.value as number;
      if (Number.isFinite(vMin) && v < vMin) return false;
      if (Number.isFinite(vMax) && v > vMax) return false;
      if (excludedReportSet.has(r.source_report_id)) return false;
      if (hrtTs != null && hrtFilter !== 'all') {
        const t = Date.parse(r.date);
        if (Number.isFinite(t)) {
          if (hrtFilter === 'pre'  && t >= hrtTs) return false;
          if (hrtFilter === 'post' && t <  hrtTs) return false;
        }
      }
      if (flagFilter.active) {
        const f = r.flag ?? '';
        if (!f && !flagFilter.includeUnflagged) return false;
        if (f && !flagFilter.wanted.has(f)) {
          // Allow unflagged through only if `onlyUnflagged` is set; otherwise
          // a flagged reading must match one of the wanted categories.
          if (!flagFilter.includeUnflagged) return false;
        }
      }
      return true;
    });

    // Last-N cap is applied AFTER filtering so users can pair "abnormal only"
    // with "last 5" and get the 5 most recent abnormals.
    if (lastNPerAnalyte > 0) {
      out = [...out].sort((a, b) => a.date.localeCompare(b.date)).slice(-lastNPerAnalyte);
    }
    return out;
  }

  // Visualisation-friendly view per analyte.
  type Card = {
    id: string;
    name: string;
    section: string;
    unit: string;
    points: { date: string; value: number; label?: string | null; flag?: string | null }[];
    refBands: { low: number | null; high: number | null; tier: 'normal' | 'borderline' | 'critical' }[];
    n: number;
    nBefore: number;
    latest: { value: number; date: string } | null;
    first: { value: number; date: string } | null;
  };

  const cards = $derived.by<Card[]>(() => {
    const sex = (activePatient?.sex ?? '?');
    const out: Card[] = [];
    // Iterate in the user's chosen order so reorder buttons / drags take
    // visual effect immediately.
    for (const id of pickedOrder) {
      const meta = analytes.find((a) => a.id === id);
      const rawRows = readingsByAnalyte[id] ?? [];
      const filtered = applyFilters(rawRows).filter((r) => r.value != null);
      const points = filtered.map((r) => ({
        date: r.date,
        value: r.value as number,
        label: r.source_report_nickname ?? null,
        flag: r.flag ?? null,
      }));
      // Reference band: prefer the analyte's library default for this sex,
      // fall back to the most-recently-printed range if any.
      let refBands: Card['refBands'] = [];
      if (meta?.default_ref_json) {
        const ref = defaultRefFor(meta.default_ref_json, sex);
        if (ref && (ref.low != null || ref.high != null)) {
          refBands = [{ low: ref.low, high: ref.high, tier: 'normal' }];
        }
      }
      if (refBands.length === 0) {
        const r = filtered.find((x) => x.ref_low != null || x.ref_high != null);
        if (r) refBands = [{ low: r.ref_low, high: r.ref_high, tier: 'normal' }];
      }
      const unit = prettyUnit(filtered.find((r) => r.unit)?.unit ?? null);
      const latest = filtered.length ? { value: filtered[filtered.length - 1].value as number, date: filtered[filtered.length - 1].date } : null;
      const first  = filtered.length ? { value: filtered[0].value as number, date: filtered[0].date } : null;
      out.push({
        id,
        name: meta?.pt_name ?? id,
        section: meta?.section ?? '',
        unit,
        points,
        refBands,
        n: filtered.length,
        nBefore: rawRows.filter((r) => r.value != null && (includeInlinePriors || !r.inline_prior)).length,
        latest,
        first,
      });
    }
    return out;
  });

  // Presets come from the shared store, which merges bundled + user-defined
  // overrides. Edited / created via Settings ▸ Charts ▸ Comparison presets.
  const presets = $derived<ComparePreset[]>(comparePresets.presets);

  // Per-patient cache of flagged-analyte buckets. Populated by an effect on
  // patient change so the dynamic presets light up the moment a new patient
  // is selected — no extra click required.
  let flaggedAnalytes = $state<FlaggedAnalytesResult>({ abnormal: [], subclinical: [] });
  let flaggedLoading  = $state(false);

  $effect(() => {
    void patientId;
    if (!patientId) {
      flaggedAnalytes = { abnormal: [], subclinical: [] };
      return;
    }
    void loadFlagged();
  });

  async function loadFlagged() {
    if (!patientId) return;
    flaggedLoading = true;
    try {
      flaggedAnalytes = await listFlaggedAnalytes(patientId);
    } catch (e) {
      // Soft-fail: dynamic presets just stay disabled, the rest of the page
      // is unaffected.
      toasts.error(e);
      flaggedAnalytes = { abnormal: [], subclinical: [] };
    } finally {
      flaggedLoading = false;
    }
  }

  function presetCount(p: ComparePreset): number {
    if (p.kind === 'static') return p.ids.length;
    return (p.dataSource === 'abnormal' ? flaggedAnalytes.abnormal : flaggedAnalytes.subclinical).length;
  }

  function applyPreset(p: ComparePreset) {
    const known = new Set(analytes.map((a) => a.id));
    const ids = p.kind === 'static'
      ? p.ids
      : (p.dataSource === 'abnormal' ? flaggedAnalytes.abnormal : flaggedAnalytes.subclinical);
    // Preset application replaces the order — that's the simplest
    // interpretation of "click a preset". The user can drag/arrow to
    // re-order within the bundle afterwards.
    pickedOrder = ids.filter((id) => known.has(id));
    // If the preset carries filters, reset every filter and apply the
    // overrides. `null` means "don't touch filters" so a stripped-down
    // preset behaves exactly as before.
    if (p.filters) {
      const next = { ...EMPTY_FILTERS, ...p.filters };
      dateFromIso         = next.dateFromIso;
      dateUntilIso        = next.dateUntilIso;
      valueMin            = next.valueMin;
      valueMax            = next.valueMax;
      includeInlinePriors = next.includeInlinePriors;
      onlyAbnormal        = next.onlyAbnormal;
      onlyCritical        = next.onlyCritical;
      onlyNormal          = next.onlyNormal;
      onlyUnflagged       = next.onlyUnflagged;
      lastNPerAnalyte     = next.lastNPerAnalyte;
      excludeReportIds    = next.excludeReportIds;
      intersectOnly       = next.intersectOnly;
      hrtFilter           = next.hrtFilter;
      // Auto-open the filter panel so the user can see what changed,
      // but only if filters were actually overridden (not no-ops).
      if (Object.keys(p.filters).length > 0) filtersOpen = true;
    }
  }

  function delta(card: Card): { abs: number; pct: number } | null {
    if (!card.first || !card.latest || card.first.value === 0) return null;
    const abs = card.latest.value - card.first.value;
    const pct = (abs / card.first.value) * 100;
    return { abs, pct };
  }

  // Shared X-axis bounds — the union of every selected analyte's reading
  // span. Passed into each TimeSeries below so the stacked charts share a
  // single timeline; readings on the same calendar date line up vertically.
  // When `intersectOnly` is true, we instead compute the INTERSECTION —
  // the date span where every analyte has at least one reading — which
  // produces a tighter "genuinely-overlapping" view.
  // `null` falls back to per-chart auto-fit when no points are selected.
  const sharedXRange = $derived.by<{ min: number; max: number } | null>(() => {
    if (cards.length === 0) return null;
    if (intersectOnly) {
      let lo = Number.NEGATIVE_INFINITY;
      let hi = Number.POSITIVE_INFINITY;
      for (const c of cards) {
        if (c.points.length === 0) return null; // any empty card collapses the intersection
        const ts = c.points.map((p) => Date.parse(p.date)).filter(Number.isFinite);
        if (ts.length === 0) return null;
        const cLo = Math.min(...ts);
        const cHi = Math.max(...ts);
        if (cLo > lo) lo = cLo;
        if (cHi < hi) hi = cHi;
      }
      if (!Number.isFinite(lo) || !Number.isFinite(hi) || lo >= hi) return null;
      return { min: lo, max: hi };
    }
    let lo = Number.POSITIVE_INFINITY;
    let hi = Number.NEGATIVE_INFINITY;
    for (const c of cards) {
      for (const p of c.points) {
        const t = Date.parse(p.date);
        if (!Number.isFinite(t)) continue;
        if (t < lo) lo = t;
        if (t > hi) hi = t;
      }
    }
    if (!Number.isFinite(lo) || !Number.isFinite(hi) || lo === hi) return null;
    return { min: lo, max: hi };
  });

  // Aggregate stats — how many readings the filter dropped, total kept,
  // intersection span. Surfaces in the filter bar so the user knows what
  // they've narrowed down to.
  const filterSummary = $derived.by(() => {
    const kept    = cards.reduce((s, c) => s + c.n, 0);
    const before  = cards.reduce((s, c) => s + c.nBefore, 0);
    return { kept, before, dropped: Math.max(0, before - kept) };
  });

  function fmtDate(ts: number): string {
    try { return new Date(ts).toISOString().slice(0, 10); }
    catch { return ''; }
  }

  const filterActive = $derived(
    !!(dateFromIso || dateUntilIso || valueMin !== '' || valueMax !== ''
       || includeInlinePriors
       || onlyAbnormal || onlyCritical || onlyNormal || onlyUnflagged
       || lastNPerAnalyte > 0 || excludeReportIds.trim() || intersectOnly
       || (hrtFilter !== 'all' && activePatient?.hrt_start_iso))
  );
</script>

<svelte:window onclick={onPatientPickerWindowClick} />

<div class="space-y-3">
  <header class="flex items-baseline justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">Compare</h1>
      <p class="text-xs text-fg3">
        Stack multiple analytes for one patient on a shared timeline. Each plot has
        its own scale and reference band. Selections, filters, and per-card overrides
        persist across reloads.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button class="btn text-xs" onclick={() => (filtersOpen = !filtersOpen)}>
        {filtersOpen ? '▾ Hide filters' : '▸ Filters'}
        {#if filterActive}
          <span class="pill-warn ml-1">{filterSummary.dropped} hidden</span>
        {/if}
      </button>
      <!-- Wipes the entire Compare-page state: picks, filters, isolated
           settings, local prefs, and the persisted snapshot. The patient
           selection itself stays so the user doesn't have to reselect. -->
      <button class="btn text-xs text-crit border-crit/40 hover:bg-crit/10"
              onclick={onClearAll}
              disabled={!hasAnyState}
              title="Clear picks, filters, and any per-card overrides. Patient selection stays.">
        Clear all
      </button>
    </div>
  </header>

  <!-- ───── Compare-wide filter panel ───── -->
  {#if filtersOpen}
    <div class="card p-4 space-y-4">
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-4 gap-y-3">
        <!-- Date range -->
        <div>
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">Date range</h3>
          <div class="grid grid-cols-2 gap-2">
            <label class="flex flex-col gap-1 text-[11px] text-fg3">
              <span>From</span>
              <input class="input" type="date" bind:value={dateFromIso} />
            </label>
            <label class="flex flex-col gap-1 text-[11px] text-fg3">
              <span>Until</span>
              <input class="input" type="date" bind:value={dateUntilIso} />
            </label>
          </div>
          <label class="flex items-center gap-2 mt-2 text-xs text-fg2 cursor-pointer">
            <input type="checkbox" bind:checked={intersectOnly} />
            <span>
              Intersection only
              <span class="text-fg3 block text-[10px]">Clip to the date span where every selected analyte has at least one reading.</span>
            </span>
          </label>
        </div>

        <!-- Value bounds -->
        <div>
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">Value bounds</h3>
          <div class="grid grid-cols-2 gap-2">
            <label class="flex flex-col gap-1 text-[11px] text-fg3">
              <span>Min</span>
              <input class="input" type="number" inputmode="decimal" placeholder="—" bind:value={valueMin} />
            </label>
            <label class="flex flex-col gap-1 text-[11px] text-fg3">
              <span>Max</span>
              <input class="input" type="number" inputmode="decimal" placeholder="—" bind:value={valueMax} />
            </label>
          </div>
          <p class="text-[10px] text-fg3 mt-1.5">
            Applies the same numeric cutoff to every selected analyte. Useful for clipping outliers
            on a single-unit panel; less useful when comparing analytes with different magnitudes.
          </p>
        </div>

        <!-- Flag filter -->
        <div>
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">Flag filter</h3>
          <div class="grid grid-cols-2 gap-y-1.5 gap-x-3 text-xs text-fg2">
            <label class="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" bind:checked={onlyNormal} />
              <span>Normal only</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" bind:checked={onlyAbnormal} />
              <span class="text-warn">Abnormal</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" bind:checked={onlyCritical} />
              <span class="text-crit">Critical</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" bind:checked={onlyUnflagged} />
              <span class="text-fg3">Unflagged</span>
            </label>
          </div>
          <p class="text-[10px] text-fg3 mt-1.5">
            All four off = no flag filter. Multiple checks are unioned (e.g. abnormal + critical
            shows everything outside the normal range).
          </p>
        </div>

        <!-- Inline-priors + last-N -->
        <div>
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">Reading source</h3>
          <label class="flex items-center gap-2 text-xs text-fg2 cursor-pointer">
            <input type="checkbox" bind:checked={includeInlinePriors} />
            <span>
              Include inline-prior values
              <span class="text-fg3 block text-[10px]">Older values printed on newer reports' "previous" columns. Off keeps things to first-class readings.</span>
            </span>
          </label>
          <div class="mt-2">
            <label class="flex flex-col gap-1 text-[11px] text-fg3">
              <span>Keep last N per analyte (0 = all)</span>
              <input class="input w-24" type="number" min="0" max="500" step="1" bind:value={lastNPerAnalyte} />
            </label>
          </div>
        </div>

        <!-- Excluded report IDs -->
        <div class="md:col-span-2 lg:col-span-2">
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">Exclude specific reports</h3>
          <input class="input w-full font-mono text-xs"
                 type="text"
                 placeholder="2024-09-07-abc12345, 2023-01-15-def67890"
                 bind:value={excludeReportIds} />
          <p class="text-[10px] text-fg3 mt-1.5">
            Comma- or space-separated report IDs. Every reading sourced from these reports is
            dropped from every chart in the comparison. Useful for redacting a redo / unreliable
            draw without deleting it.
          </p>
        </div>

        <!-- HRT-anchor filter — only meaningful when the active patient has
             a hrt_start_iso set. We render the panel either way so users
             see WHY it's missing for unanchored patients (the muted hint
             stays visible). -->
        <div>
          <h3 class="text-xs font-semibold text-fg2 mb-1.5">HRT anchor</h3>
          {#if activePatient?.hrt_start_iso}
            <div class="seg seg--wrap-compare">
              {#each [
                { id: 'all',  label: 'All readings' },
                { id: 'pre',  label: 'Pre-HRT'      },
                { id: 'post', label: 'Post-HRT'     }
              ] as opt}
                <button type="button"
                        class="seg__opt {hrtFilter === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => (hrtFilter = opt.id as 'all' | 'pre' | 'post')}>
                  {opt.label}
                </button>
              {/each}
            </div>
            <p class="text-[10px] text-fg3 mt-1.5">
              Anchor: <span class="font-mono">{activePatient.hrt_start_iso}</span>.
              Pre keeps readings strictly before this date; Post keeps the anchor day onwards.
            </p>
          {:else}
            <p class="text-[10px] text-fg3 italic">
              No HRT anchor set for this patient. Set <span class="font-mono">hrt_start_iso</span>
              on the patient page to enable pre/post filtering here.
            </p>
          {/if}
        </div>
      </div>

      <div class="flex items-center justify-between text-xs text-fg2 border-t border-line pt-3">
        <div>
          {#if filterActive}
            <span class="font-medium">{filterSummary.kept}</span> of
            <span class="text-fg3">{filterSummary.before}</span> readings kept across
            <span class="font-medium">{cards.length}</span> chart{cards.length === 1 ? '' : 's'}.
            {#if filterSummary.dropped > 0}
              <span class="text-warn">{filterSummary.dropped} hidden</span> by the active filters.
            {/if}
          {:else}
            No filters active.
          {/if}
        </div>
        <button class="btn text-xs" onclick={clearFilters} disabled={!filterActive}>Clear filters</button>
      </div>
    </div>
  {/if}

  <div class="card p-3 grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-3">
    <!-- Left: patient + analyte picker -->
    <div class="space-y-3">
      <label class="flex flex-col gap-1 text-xs text-fg2">
        <span>Patient</span>
        <div class="relative" bind:this={patientPickerElement}>
          <button
            type="button"
            class="select flex w-full items-center justify-between gap-2 text-left cursor-pointer"
            aria-haspopup="listbox"
            aria-expanded={patientMenuOpen}
            aria-controls="compare-patient-options"
            onclick={() => patientMenuOpen ? closePatientMenu() : openPatientMenu()}
          >
            <span class="truncate {activePatient ? '' : 'text-fg3'}">
              {activePatient ? `${activePatient.display_name} (${activePatient.report_count})` : 'Select…'}
            </span>
            <span aria-hidden="true" class="shrink-0 text-fg3">⌄</span>
          </button>

          {#if patientMenuOpen}
            <div class="absolute left-0 right-0 z-30 mt-1 card p-2 shadow-lg bg-bg2">
              <input
                type="search"
                class="input w-full"
                placeholder="Search patients…"
                aria-label="Search patients"
                bind:value={patientSearch}
                bind:this={patientSearchElement}
                onkeydown={onPatientSearchKeydown}
              />
              <div id="compare-patient-options" class="mt-1 max-h-60 overflow-y-auto" role="listbox" aria-label="Patients">
                {#each filteredPatients as p, index (p.id)}
                  <button
                    type="button"
                    role="option"
                    aria-selected={p.id === patientId}
                    class="w-full rounded px-2 py-1.5 text-left text-xs hover:bg-bg3 {index === patientActiveIndex ? 'bg-bg3' : ''}"
                    onmousedown={(event) => { event.preventDefault(); selectPatient(p.id); }}
                  >
                    <span class="block truncate font-medium">{p.display_name}</span>
                    <span class="block text-[10px] text-fg3">{p.report_count} report{p.report_count === 1 ? '' : 's'} · {p.id}</span>
                  </button>
                {/each}
                {#if filteredPatients.length === 0}
                  <div class="px-2 py-3 text-center text-xs text-fg3">No patients match.</div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
      </label>

      <div class="flex flex-col gap-1 text-xs text-fg2">
        <span>Presets {flaggedLoading ? '· loading…' : ''}</span>
        <div class="flex flex-wrap gap-1">
          {#each presets as p}
            {@const count = presetCount(p)}
            {@const dynamicEmpty = p.kind === 'dynamic' && count === 0}
            {@const hasFilters = !!p.filters && Object.keys(p.filters).length > 0}
            <button class="btn text-[11px] px-2 py-1 flex items-center gap-1
                           {p.kind === 'dynamic' && !dynamicEmpty ? 'border-accent/40' : ''}
                           {p.source === 'user' ? 'border-accent/40' : ''}
                           {dynamicEmpty ? 'opacity-50 cursor-not-allowed' : ''}"
                    onclick={() => !dynamicEmpty && applyPreset(p)}
                    disabled={dynamicEmpty}
                    title={(p.kind === 'dynamic'
                      ? `${(p as { hint?: string }).hint ?? ''}${dynamicEmpty ? ' — none for this patient.' : ` (${count} for this patient)`}`
                      : `${count} analyte${count === 1 ? '' : 's'}`) + (hasFilters ? ' · sets filters' : '')}>
              {#if p.kind === 'dynamic'}
                <span class="text-[9px]">●</span>
              {:else if p.source === 'user'}
                <span class="text-[9px] text-accent">★</span>
              {/if}
              <span>{p.name}</span>
              {#if count > 0}
                <span class="text-[9px] text-fg3 tabular-nums">{count}</span>
              {/if}
              {#if hasFilters}
                <span class="text-[9px] text-warn" title="This preset also sets filters">⚐</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <label class="flex flex-col gap-1 text-xs text-fg2">
        <span>Filter analytes</span>
        <input class="input" type="text" placeholder="search…" bind:value={analyteFilter} />
      </label>

      <div class="flex items-center justify-between text-xs text-fg2">
        <span>{pickedAnalyteIds.size} selected</span>
        {#if pickedAnalyteIds.size > 0}
          <button class="text-xs text-accent hover:underline" onclick={clearPicked}>Clear</button>
        {/if}
      </div>
      <div class="border border-line rounded max-h-[60vh] overflow-y-auto">
        {#each filteredAnalytes as a (a.id)}
          {@const picked = pickedAnalyteIds.has(a.id)}
          <button type="button"
                  class="w-full flex items-center justify-between gap-2 px-2 py-1 text-left text-xs hover:bg-bg2 transition-colors {picked ? 'bg-accent/10 text-accent' : ''}"
                  onclick={() => toggleAnalyte(a.id)}>
            <span class="truncate">
              <span class="font-medium">{a.pt_name}</span>
              {#if a.section}<span class="text-fg3 ml-1">· {a.section}</span>{/if}
            </span>
            <span class="text-[10px] text-fg3 tabular-nums">{a.result_count}</span>
          </button>
        {:else}
          <div class="px-2 py-3 text-xs text-fg3 text-center">No analytes match that filter.</div>
        {/each}
      </div>
    </div>

    <!-- Right: stacked sparklines, one per picked analyte. `min-w-0` is
         critical — without it, this grid item defaults to its
         intrinsic content width (driven by the chart canvas) and refuses
         to shrink when the window narrows. With min-w-0 it honours the
         1fr column track's bounds in both directions. -->
    <div class="space-y-3 min-w-0">
      {#if !patientId}
        <div class="card p-6 text-sm text-fg3 text-center">Pick a patient to begin.</div>
      {:else if pickedAnalyteIds.size === 0}
        <div class="card p-6 text-sm text-fg3 text-center">
          Select one or more analytes from the list, or apply a preset.
        </div>
      {:else if loading}
        <div class="card p-6 text-sm text-fg3 text-center">Loading…</div>
      {:else}
        {#each cards as c, idx (c.id)}
          {@const isolated = isolatedAnalyteIds.has(c.id)}
          {@const isDragging = dragSrcIdx === idx}
          {@const isDropTarget = dragOverIdx === idx && dragSrcIdx !== idx}
          <section class="card p-3 transition-colors
                          {isDragging ? 'opacity-40' : ''}
                          {isDropTarget ? 'border-accent' : ''}"
                   role="listitem"
                   aria-grabbed={isDragging}
                   draggable={true}
                   ondragstart={() => onDragStart(idx)}
                   ondragenter={() => onDragEnter(idx)}
                   ondragover={onDragOver}
                   ondragleave={() => { if (dragOverIdx === idx) dragOverIdx = null; }}
                   ondrop={() => onDrop(idx)}
                   ondragend={onDragEnd}>
            <!-- Three-column header: reorder controls on the left, stats
                 in the middle, analyte name pinned to the right. The
                 name's text-right alignment keeps long analyte labels
                 hugging the card's right edge instead of getting lost
                 in the middle of the row. -->
            <header class="flex items-baseline justify-between mb-2 gap-2 flex-wrap">
              <div class="flex items-center gap-1">
                <!-- Drag handle + arrow buttons. The grip cursor signals
                     drag affordance; the arrows offer a keyboard / touch
                     alternative for users who can't drag. -->
                <span class="cursor-grab active:cursor-grabbing text-fg3 hover:text-fg1 select-none px-1"
                      title="Drag to reorder">⋮⋮</span>
                <button type="button"
                        class="reorder-btn"
                        onclick={(e) => { e.stopPropagation(); moveAnalyte(idx, -1); }}
                        disabled={idx === 0}
                        title="Move up">↑</button>
                <button type="button"
                        class="reorder-btn"
                        onclick={(e) => { e.stopPropagation(); moveAnalyte(idx, +1); }}
                        disabled={idx === cards.length - 1}
                        title="Move down">↓</button>
              </div>
              <!-- Right-side cluster — analyte name first, then stats. The
                   `ml-auto` on this wrapper pushes the whole cluster to the
                   right of the row so the reorder controls stay anchored
                   on the left. Name sits just before the unit/count/delta. -->
              <div class="ml-auto flex items-baseline gap-3 flex-wrap justify-end">
                <a href={`/analyte/${c.id}?patient=${patientId}`}
                   class="text-sm font-semibold hover:text-accent text-right truncate max-w-[18rem]"
                   title={c.name}>
                  {c.name}
                </a>
                <div class="text-xs text-fg2 tabular-nums flex items-center gap-3">
                  {#if c.unit}<span class="font-mono text-fg3">{c.unit}</span>{/if}
                  <span>{c.n}{c.n !== c.nBefore ? `/${c.nBefore}` : ''} reading{c.n === 1 ? '' : 's'}</span>
                  {#if c.latest}
                    <span class="font-medium">{c.latest.value}</span>
                  {/if}
                  {#if delta(c)}
                    {@const d = delta(c)!}
                    <span class={d.abs > 0 ? 'text-warn' : d.abs < 0 ? 'text-accent' : 'text-fg3'}>
                      {d.abs > 0 ? '+' : ''}{d.abs.toFixed(2)} ({d.pct > 0 ? '+' : ''}{d.pct.toFixed(0)}%)
                    </span>
                  {/if}
                  <!-- Per-card "isolated settings" gate. Off → reads + writes
                       the global chartPrefs. On → swaps in a LocalChartPrefs
                       so every toolbar tweak on this card stays scoped to it. -->
                  <label class="flex items-center gap-1 cursor-pointer text-[11px] text-fg3 hover:text-fg1"
                         title={isolated
                           ? "Reading from a per-chart preference snapshot — toolbar toggles here won't affect the global default."
                           : "Reading from the global default. Tick to give this chart its own isolated copy."}>
                    <input type="checkbox" checked={isolated}
                           onchange={() => toggleIsolatedFor(c.id)} />
                    <span>{isolated ? '⚙ custom' : 'global'}</span>
                  </label>
                  {#if isolated}
                    <button class="text-[11px] text-accent hover:underline"
                            onclick={() => resyncIsolatedFromGlobal(c.id)}
                            title="Discard this chart's local overrides and snapshot the current global defaults again.">
                      resync
                    </button>
                  {/if}
                </div>
              </div>
            </header>
            {#if c.points.length === 0}
              <div class="text-xs text-fg3 italic px-2 py-6 text-center">
                {filterActive
                  ? `No readings match the active filters for ${c.name}.`
                  : `No readings for ${activePatient?.display_name ?? 'this patient'}.`}
              </div>
            {:else}
              <TimeSeries
                points={c.points}
                refBands={c.refBands}
                unit={c.unit}
                height={180}
                exportName={`compare_${c.id}_${patientId}`}
                xAxisRange={sharedXRange}
                prefs={prefsFor(c.id)}
              />
            {/if}
          </section>
        {/each}

        {#if sharedXRange}
          <div class="text-[11px] text-fg3 text-center">
            Shared timeline: {fmtDate(sharedXRange.min)} → {fmtDate(sharedXRange.max)}
            {#if intersectOnly} (intersection){/if}
          </div>
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  /* Compact reorder buttons used in each compared card's header. Sized so
     two of them fit alongside the drag-grip without overwhelming the
     card heading. */
  .reorder-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.4rem;
    height: 1.4rem;
    border: 1px solid rgb(var(--line));
    background: rgb(var(--bg-2));
    border-radius: 0.3rem;
    color: rgb(var(--fg-2));
    font-size: 0.75rem;
    line-height: 1;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .reorder-btn:hover:not(:disabled) {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--accent) / 0.5);
  }
  .reorder-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  /* Local copy of the segmented-control primitive used by the HRT-anchor
     filter. Mirrors the look used on the analyte / settings pages but
     scoped here so the rule survives Svelte's per-component CSS scope. */
  .seg {
    display: inline-flex;
    flex-wrap: wrap;
    width: 100%;
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
  }
  .seg__opt {
    flex: 1 1 auto;
    padding: 0.35rem 0.6rem;
    font-size: 0.75rem;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 0;
    border-right: 1px solid rgb(var(--line));
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }
  .seg__opt:last-child { border-right: 0; }
  .seg__opt:hover { background: rgb(var(--bg-2)); color: rgb(var(--fg-1)); }
  .seg__opt--on {
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-weight: 600;
  }
</style>
