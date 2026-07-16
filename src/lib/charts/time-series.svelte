<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as echarts from 'echarts';
  import {
    ensureThemesRegistered,
    reRegisterThemes,
    activeThemeName,
    refBandColors
  } from '$theme/echarts-themes';
  import { toasts } from '../toasts/store.svelte';
  import { chartPrefs, type ChartPrefsLike } from './prefs.svelte';

  type Point = {
    date: string;
    value: number;
    /** Optional friendly label (the source report's nickname). When the
     *  "labels" toggle is on, this string replaces the date on the X axis
     *  and tooltip header. */
    label?: string | null;
    /** Optional flag — used by the "color by flag" toggle to tint each
     *  point individually. Falls through to the series colour when absent. */
    flag?: 'low' | 'high' | 'normal' | 'critical_low' | 'critical_high' | 'abnormal_qual' | string | null;
  };
  type RefBand = { low: number | null; high: number | null; tier: 'normal' | 'borderline' | 'critical' };

  let {
    points = [] as Point[],
    refBands = [] as RefBand[],
    unit = '' as string,
    title = '' as string,
    height = 320,
    /** Filename stem for saveAsImage / clipboard. */
    exportName = 'chart' as string,
    /** Optional X-axis lock. When provided, the chart ignores its own data
     *  span on the X axis and pins to these millis-since-epoch bounds. The
     *  Compare page uses this so every stacked chart shares a single
     *  timeline — readings from different analytes line up vertically by
     *  date. `null` means "use the chart's own data span" (default). */
    xAxisRange = null as { min: number; max: number } | null,
    /** Source of preference values + setters. Defaults to the global
     *  `chartPrefs` so unmodified callers keep the global-default behaviour;
     *  the Compare page passes a per-card `LocalChartPrefs` instance when
     *  the user clicks "use isolated settings" so toolbar mutations and
     *  configuration inputs stay scoped to that one chart. */
    prefs = chartPrefs as ChartPrefsLike,
  } = $props<{
    points: Point[];
    refBands?: RefBand[];
    unit?: string;
    title?: string;
    height?: number;
    exportName?: string;
    xAxisRange?: { min: number; max: number } | null;
    prefs?: ChartPrefsLike;
  }>();

  // ── User-controllable display options ────────────────────────────────
  // Each toggle reads through to the global `chartPrefs` so flipping it in
  // the toolbar persists as the new default. Reset everything from
  // Settings ▸ Charts ▸ Reset chart defaults.
  const showValues       = $derived(prefs.showValues);
  const smooth           = $derived(prefs.smooth);
  const scale            = $derived(prefs.scale);
  const showBands        = $derived(prefs.showBands);
  const useNicknames     = $derived(prefs.useNicknames);
  const showSymbols      = $derived(prefs.showSymbols);
  const lineWidth        = $derived(prefs.lineWidth);
  const symbolSize       = $derived(prefs.symbolSize);
  const gridDensity      = $derived(prefs.gridDensity);
  const dateFormat       = $derived(prefs.dateFormat);
  const showTodayLine    = $derived(prefs.showTodayLine);
  const showTrendLine    = $derived(prefs.showTrendLine);
  const showMeanLine     = $derived(prefs.showMeanLine);
  const showMinMaxMarkers= $derived(prefs.showMinMaxMarkers);
  const seriesType       = $derived(prefs.seriesType);
  const stepKind         = $derived(prefs.step);
  const colorByFlag      = $derived(prefs.colorByFlag);
  const connectNulls     = $derived(prefs.connectNulls);
  const bandOpacityPct   = $derived(prefs.bandOpacityPct);
  const animation        = $derived(prefs.animation);
  const tooltipMode      = $derived(prefs.tooltipMode);
  const showTitle        = $derived(prefs.showTitle);
  const xLabelMode       = $derived(prefs.xLabelMode);
  const timeWindow       = $derived(prefs.timeWindow);
  // Tooltip suffix indicating where toolbar mutations are saved.
  const persistHint      = $derived(
    prefs === chartPrefs
      ? 'Saved as the global default.'
      : 'Local to this chart only.'
  );

  let host = $state<HTMLDivElement | null>(null);
  let chart: echarts.ECharts | null = null;

  /**
   * Two-stage X-axis sizing strategy:
   *
   *   - The *axis bounds* are padded generously (X_AXIS_PAD = 40%) so the
   *     user has plenty of room to zoom OUT past the data and see context
   *     beyond the first/last point.
   *   - The *default visible window* (X_DEFAULT_VISIBLE_PAD = 7%) is
   *     much tighter so the chart opens looking populated — data fills
   *     the canvas with just a hair of margin on each side.
   *
   * `xZoomDefault` translates the visible-pad into start/end percentages
   * relative to the axis bounds. With axis ±40% and visible ±7%:
   *   start = (40 - 7) / (100 + 80)% × 100 = 33 / 180 ≈ 18.3%
   *   end   = (140 + 7) / 180             ≈ 81.7%
   * (using the algebra `(pad − vis) / (1 + 2·pad)` and its mirror).
   */
  // X axis padding is configurable via Settings ▸ Charts ▸ Default zoom &
  // padding. 5% is the bundled default (tight margins around the data).
  // Visible window always opens at the maximum extent — the user wanted
  // the timeline to start snug, with zoom-out limited to the configured pad.
  const X_AXIS_PAD = $derived(prefs.xAxisPadPct / 100);
  const xZoomDefault = { start: 0, end: 100 };

  // Y axis keeps the two-stage strategy: bounds extend `yAxisPadPct` past
  // the data so the user can zoom OUT vertically (e.g. to fit a reference
  // band that extends well beyond the data); the default visible window
  // opens snug against the data with a tiny `yVisiblePadPct` buffer.
  const Y_AXIS_PAD            = $derived(prefs.yAxisPadPct    / 100);
  const Y_DEFAULT_VISIBLE_PAD = $derived(prefs.yVisiblePadPct / 100);
  const yZoomDefault = $derived.by(() => {
    const total = 1 + 2 * Y_AXIS_PAD;
    if (total <= 0) return { start: 0, end: 100 };
    const start = ((Y_AXIS_PAD - Y_DEFAULT_VISIBLE_PAD) / total) * 100;
    const end   = ((Y_AXIS_PAD + 1 + Y_DEFAULT_VISIBLE_PAD) / total) * 100;
    return { start: Math.max(0, start), end: Math.min(100, end) };
  });

  /**
   * Compute a padded `min`/`max` for the time axis: shift the bound by
   * `padFraction` of the data span beyond the actual extreme. Returns
   * ms-since-epoch so ECharts' time axis can consume it directly.
   * Returns `undefined` when `points` has fewer than 2 entries.
   */
  function timePad(side: 'min' | 'max', pts: Point[], padFraction: number): number | undefined {
    if (pts.length < 2) return undefined;
    const ms = pts.map((p) => Date.parse(p.date));
    const lo = Math.min(...ms);
    const hi = Math.max(...ms);
    const span = hi - lo;
    if (!Number.isFinite(span) || span <= 0) return undefined;
    return side === 'min'
      ? lo - span * padFraction
      : hi + span * padFraction;
  }

  // Resolve the active theme's tokens once per build. Returns rgb(...) strings
  // ECharts can consume directly. Chart is rebuilt on `theme-change`, so
  // the label badge re-tints automatically when the user flips light/dark.
  function themeVar(name: string, fallback: string): string {
    if (typeof document === 'undefined') return fallback;
    const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return raw ? `rgb(${raw})` : fallback;
  }

  // Build a Date.parse-keyed lookup from point timestamps to their friendly
  // labels (the source report's nickname). The X-axis label formatter
  // receives a millis timestamp, so we precompute this once per render.
  function buildNicknameIndex(pts: Point[]): Map<number, string> {
    const m = new Map<number, string>();
    for (const p of pts) {
      if (!p.label) continue;
      const t = Date.parse(p.date);
      if (Number.isFinite(t)) m.set(t, p.label);
    }
    return m;
  }

  // ECharts label formatter: produces a short label that fits the axis.
  // Falls back to the configured date format when the point's report has
  // no nickname.
  function formatNicknameAxisLabel(value: number): string {
    const nick = nicknameIndex.get(value);
    if (nick) return nick;
    return formatAxisDate(value, dateFormat);
  }
  // Mutable so buildOption picks up the freshest version on each render.
  let nicknameIndex: Map<number, string> = new Map();

  /** Format a numeric value for axis labels / tooltips with adaptive decimal
   *  precision. Without this, ECharts' default formatter happily prints
   *  `0.20000000000000004` because of float imprecision in our pad math.
   *  We pick precision based on magnitude: 0 decimals for >=1000, 1 for
   *  >=100, 2 for >=10, 3 for >=1, 4 for sub-unit values. Trailing zeros
   *  are stripped so integers still render as integers. */
  function formatAxisValue(v: number): string {
    if (!Number.isFinite(v)) return '';
    if (Number.isInteger(v) && Math.abs(v) < 1e9) return v.toString();
    const abs = Math.abs(v);
    const decimals =
      abs >= 1000 ? 0 :
      abs >= 100  ? 1 :
      abs >= 10   ? 2 :
      abs >= 1    ? 3 : 4;
    // toFixed produces fixed-width strings with trailing zeros; round-trip
    // through parseFloat to drop them so 12.300 -> "12.3".
    return parseFloat(v.toFixed(decimals)).toString();
  }

  /** Format a millis timestamp for the X axis according to the user's
   *  configured chart date-format default. Falls back to ISO when the
   *  timestamp is malformed so we never render `Invalid Date`. */
  function formatAxisDate(value: number, fmt: 'iso' | 'short' | 'monthYear' | 'numeric'): string {
    const d = new Date(value);
    if (Number.isNaN(d.getTime())) return '';
    const yyyy = d.getUTCFullYear();
    const mm   = String(d.getUTCMonth() + 1).padStart(2, '0');
    const dd   = String(d.getUTCDate()).padStart(2, '0');
    const yy   = String(yyyy).slice(2);
    const monNames = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    const mon  = monNames[d.getUTCMonth()];
    switch (fmt) {
      case 'short':     return `${mon} ${dd}`;
      case 'monthYear': return `${mon} '${yy}`;
      case 'numeric':   return `${mm}/${dd}/${yy}`;
      case 'iso':
      default:          return `${yyyy}-${mm}-${dd}`;
    }
  }

  /** Linear regression over the visible data points. Returns `null` for
   *  fewer than two points or when the X span is zero. The trend line is
   *  rendered as a second series anchored at the first / last points so it
   *  honours the chart's date axis without needing extra interpolation. */
  function trendlineEndpoints(pts: Point[]): { from: [string, number]; to: [string, number] } | null {
    const valid = pts
      .map((p) => ({ x: Date.parse(p.date), y: p.value }))
      .filter((p) => Number.isFinite(p.x) && Number.isFinite(p.y));
    if (valid.length < 2) return null;
    const n = valid.length;
    const meanX = valid.reduce((s, p) => s + p.x, 0) / n;
    const meanY = valid.reduce((s, p) => s + p.y, 0) / n;
    let num = 0, den = 0;
    for (const p of valid) {
      num += (p.x - meanX) * (p.y - meanY);
      den += (p.x - meanX) ** 2;
    }
    if (den === 0) return null;
    const slope = num / den;
    const intercept = meanY - slope * meanX;
    const xMin = valid[0].x;
    const xMax = valid[valid.length - 1].x;
    return {
      from: [new Date(xMin).toISOString(), slope * xMin + intercept],
      to:   [new Date(xMax).toISOString(), slope * xMax + intercept],
    };
  }

  /** Map a parser flag string to a tinted RGB so points can be coloured by
   *  flag when `colorByFlag` is on. Falls through to the series colour for
   *  null / unknown flags so a partly-flagged series still renders. */
  function flagColor(flag: Point['flag'] | undefined): string | undefined {
    switch (flag) {
      case 'low':
      case 'high':           return themeVar('--warn', 'rgb(245 158 11)');
      case 'critical_low':
      case 'critical_high':  return themeVar('--crit', 'rgb(239 68 68)');
      case 'normal':         return themeVar('--ok',   'rgb(34 197 94)');
      case 'abnormal_qual':  return themeVar('--warn', 'rgb(245 158 11)');
      default:               return undefined;
    }
  }

  /** Translate the user-chosen `timeWindow` preset into millis-since-epoch
   *  cutoffs. `all` returns null so the chart uses its data span. */
  function timeWindowCutoff(window: typeof timeWindow): number | null {
    if (window === 'all') return null;
    const days =
      window === '30d' ? 30 :
      window === '90d' ? 90 :
      window === '6m'  ? 183 :
      window === '1y'  ? 365 :
      window === '2y'  ? 730 :
      window === '5y'  ? 1825 : 0;
    if (days <= 0) return null;
    return Date.now() - days * 86_400_000;
  }

  function buildOption(): echarts.EChartsOption {
    const sortedAll = [...points].sort((a: Point, b: Point) => a.date.localeCompare(b.date));
    // Apply the default time-window filter. We keep the unfiltered set
    // around so trend / mean computations honour the visible subset.
    const cutoff = timeWindowCutoff(timeWindow);
    const sorted = cutoff != null
      ? sortedAll.filter((p) => Date.parse(p.date) >= cutoff)
      : sortedAll;
    nicknameIndex = buildNicknameIndex(sorted);
    const labelBg     = themeVar('--bg-2',  'rgb(20 23 28)');
    const labelFg     = themeVar('--fg-1',  'rgb(243 244 246)');
    const labelBorder = themeVar('--accent', 'rgb(167 139 250)');

    // Y-axis bounds. We pin them tight to the data span (with a small 8%
    // pad on each side) so the chart doesn't zoom out to include the full
    // reference band when bands extend far beyond the actual readings.
    // Without this, Estradiol with f:[11,400] would force the Y axis up
    // to 400 even when all values sit between 40 and 250 — markArea
    // extents would otherwise drag the visible range with them.
    // Setting `min`/`max` explicitly anchors the axis to the data; the
    // markArea still renders, but it gets clipped to whatever portion is
    // visible, which is what users actually want for trend reading.
    // Y axis uses the same two-stage strategy as X:
    //   - Axis bounds extend ±Y_AXIS_PAD past the data so the user can
    //     zoom OUT past the data range and see context (or fit a
    //     reference band that extends well beyond).
    //   - Default visible window is much tighter (Y_DEFAULT_VISIBLE_PAD)
    //     so the chart opens snug against the data.
    //
    // The default start/end are computed once below for the X axis and
    // reused for Y here.
    const yVals = sorted.map((p) => p.value).filter((v) => Number.isFinite(v));
    let yMin: number | undefined;
    let yMax: number | undefined;
    if (yVals.length >= 1) {
      const lo = Math.min(...yVals);
      const hi = Math.max(...yVals);
      const span = Math.max(hi - lo, Math.abs(hi) * 0.05, 1);
      yMin = lo - span * Y_AXIS_PAD;
      yMax = hi + span * Y_AXIS_PAD;
    }

    // Reference-band fills, optionally re-tinted by the user's chosen
     // band-opacity. The bundled token alpha (~12%) is the 100% baseline; the
     // pref scales it from 0 (transparent) up to 200% for an extra-bold band.
    const opacityScale = bandOpacityPct / 100;
    const tintBand = (rgba: string): string => {
      // Token alpha is bundled as `rgba(r, g, b, a)`; multiply `a` by the
      // user's scale and clamp to [0, 1]. Falls back to the original string
      // if the regex doesn't match (defensive — token format is stable).
      const m = rgba.match(/^rgba\((-?\d+),\s*(-?\d+),\s*(-?\d+),\s*([\d.]+)\)$/);
      if (!m) return rgba;
      const a = Math.max(0, Math.min(1, parseFloat(m[4]) * opacityScale));
      return `rgba(${m[1]},${m[2]},${m[3]},${a})`;
    };
    const markAreaData = showBands
      ? refBands.map((b: RefBand) => {
          const fill =
            b.tier === 'critical'
              ? refBandColors.critical()
              : b.tier === 'borderline'
                ? refBandColors.borderline()
                : refBandColors.normal();
          return [
            { yAxis: b.low ?? Number.NEGATIVE_INFINITY, itemStyle: { color: tintBand(fill) } },
            { yAxis: b.high ?? Number.POSITIVE_INFINITY }
          ];
        })
      : [];

    // Log axis chokes on zero / negative values; clamp the type to linear
    // when the data can't be log-scaled, otherwise the chart renders blank.
    const canLog = sorted.every((p) => p.value > 0);
    const yType = scale === 'log' && canLog ? 'log' : 'value';

    // Optional reference lines on the primary series. Today (xAxis) +
    // mean-of-visible (yAxis). Typed as `any[]` because ECharts' MarkLine
    // data type is a tagged union we'd otherwise have to import.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const markLineEntries: any[] = [];
    if (showTodayLine) {
      markLineEntries.push({ xAxis: new Date().toISOString(), name: 'today' });
    }
    if (showMeanLine && yVals.length > 0) {
      const mean = yVals.reduce((s, v) => s + v, 0) / yVals.length;
      markLineEntries.push({
        yAxis: mean,
        name: 'mean',
        lineStyle: { color: themeVar('--fg-3', 'rgb(107 114 128)') }
      });
    }
    const markLineConfig = markLineEntries.length
      ? {
          silent: true,
          symbol: ['none', 'none'] as [string, string],
          lineStyle: { type: 'dashed' as const, color: themeVar('--accent', 'rgb(167 139 250)'), width: 1 },
          label: { show: true, formatter: '{b}', fontSize: 10, color: themeVar('--fg-2', 'rgb(156 163 175)') },
          data: markLineEntries,
        }
      : undefined;

    return {
      title: title && showTitle ? { text: title, left: 8, top: 4 } : undefined,
      tooltip: {
        trigger: tooltipMode === 'item' ? 'item' as const : 'axis' as const,
        axisPointer: tooltipMode === 'axis' ? { type: 'cross' as const } : undefined,
        // Custom formatter so every reading's tooltip header carries the
        // source report's nickname (when set), plus the ISO date — not
        // just the date alone. This is independent of the X-axis labels
        // toggle, so even when the axis shows raw dates the user still
        // sees "Annual checkup · 2024-09-07" on hover.
        formatter: (raw: unknown) => {
          // ECharts passes either a single param object or an array
          // depending on how `trigger` is configured.
          const params = (Array.isArray(raw) ? raw : [raw]) as Array<{
            value: unknown;
            color?: string;
            seriesName?: string;
          }>;
          if (params.length === 0) return '';
          const first = params[0];
          const arr = Array.isArray(first.value) ? first.value : [first.value, first.value];
          const ts = typeof arr[0] === 'number' || typeof arr[0] === 'string'
            ? Date.parse(String(arr[0]))
            : NaN;
          const numVal = arr[1];
          const iso = Number.isFinite(ts)
            ? new Date(ts as number).toISOString().slice(0, 10)
            : '';
          const nick = Number.isFinite(ts) ? nicknameIndex.get(ts as number) : undefined;
          // Two-line header — friendly nickname on top (if any), date below.
          const header = nick
            ? `<div style="font-weight:600;margin-bottom:2px;">${nick}</div>` +
              `<div style="font-size:0.65rem;opacity:0.7;">${iso}</div>`
            : `<div style="font-weight:600;">${iso}</div>`;
          const dot = first.color
            ? `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:${first.color};margin-right:6px;"></span>`
            : '';
          const valueStr = typeof numVal === 'number'
            ? `${formatAxisValue(numVal)}${unit ? ` ${unit}` : ''}`
            : '';
          return `${header}<div style="margin-top:4px;">${dot}${valueStr}</div>`;
        }
      },
      // Built-in toolbox: data zoom, restore, save image. We provide our own
      // copy-to-clipboard + scale/labels toggles outside the chart so they
      // sit in the page chrome and follow the app theme.
      toolbox: {
        show: true,
        right: 8,
        top: 4,
        itemSize: 14,
        itemGap: 8,
        feature: {
          dataZoom: { yAxisIndex: 'none', title: { zoom: 'Zoom', back: 'Reset zoom' } },
          restore:  { title: 'Restore' },
          saveAsImage: {
            title: 'Save PNG',
            name: exportName,
            pixelRatio: 2,
            backgroundColor: 'transparent'
          }
        }
      },
      // dataZoom modes:
      //   - Drag-to-pan ("moveOnMouseMove") is always available so the
      //     user can drag the line left/right.
      //   - Wheel zoom is OFF by default — accidental scroll-to-zoom is a
      //     common UX trap when the chart sits in the middle of a long
      //     page. Toggle it on via the chart toolbar (or globally in
      //     Settings ▸ Charts ▸ Scroll-zoom).
      //   - The slider stretchers are still opt-in.
      //
      // Default `start`/`end` values come from `xZoomDefault` below. The
      // axis itself spans ±40% past the data, but the *visible* window
      // starts tighter (~7% margin each side of the data) so the chart
      // doesn't open with a sea of empty timeline. Wheel-out / button-out
      // / slider-drag-out reveal the extra padding the user asked for.
      dataZoom: ([
        { type: 'inside', xAxisIndex: 0,
          zoomOnMouseWheel: prefs.scrollZoom,
          moveOnMouseMove: true,
          start: xZoomDefault.start, end: xZoomDefault.end },
        { type: 'inside', yAxisIndex: 0,
          zoomOnMouseWheel: prefs.scrollZoom ? 'shift' : false,
          moveOnMouseMove: false,
          start: yZoomDefault.start, end: yZoomDefault.end },
        prefs.showXSlider
          ? { type: 'slider', xAxisIndex: 0, height: 18, bottom: 6,
              start: xZoomDefault.start, end: xZoomDefault.end }
          : null,
        prefs.showYSlider
          ? { type: 'slider', yAxisIndex: 0, width: 14, right: 4, top: 44, bottom: 44,
              start: yZoomDefault.start, end: yZoomDefault.end }
          : null
      ].filter(Boolean) as echarts.EChartsOption['dataZoom']),
      grid: {
        // Reserve right-margin for the Y slider only when it's visible.
        left: 56,
        right: prefs.showYSlider ? 28 : 12,
        top: 44,
        bottom: prefs.showXSlider ? 44 : 16
      },
      xAxis: {
        type: 'time',
        axisLabel: {
          // X-axis label modes:
          //   - 'auto'  : let ECharts auto-skip overlapping labels
          //   - 'rotate': tilt 35° so more fit, plus skip on collision
          //   - 'hide'  : never render labels (date still appears in
          //               the hover tooltip via the axis pointer)
          show: xLabelMode !== 'hide',
          rotate: xLabelMode === 'rotate' ? 35 : 0,
          hideOverlap: xLabelMode !== 'hide',
          formatter: useNicknames
            ? formatNicknameAxisLabel
            : (value: number) => formatAxisDate(value, dateFormat)
        },
        splitLine: { show: gridDensity !== 'off' },
        minorTick: { show: gridDensity === 'detailed' },
        minorSplitLine: { show: gridDensity === 'detailed' },
        // X-axis bounds. Two cases:
        //   - When `xAxisRange` is provided (Compare view), pin to the
        //     caller's range so every stacked chart shares one timeline.
        //     We still apply the configured `X_AXIS_PAD` so the locked
        //     range gets a touch of breathing room on each side.
        //   - Otherwise, single-stage tight margin around this chart's
        //     own data.
        min: xAxisRange
          ? xAxisRange.min - (xAxisRange.max - xAxisRange.min) * X_AXIS_PAD
          : timePad('min', sorted, X_AXIS_PAD),
        max: xAxisRange
          ? xAxisRange.max + (xAxisRange.max - xAxisRange.min) * X_AXIS_PAD
          : timePad('max', sorted, X_AXIS_PAD)
      },
      // Cast to keep TS happy: yType is the union 'log' | 'value', but
      // ECharts' YAxisOption is a discriminated union — split per literal.
      // We know the runtime shape is correct, so we widen via `as any` and
      // rely on the surrounding `yType === 'value'` checks for safety.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      yAxis: ({
        type: yType,
        scale: yType === 'value',
        // Trim noisy float-precision artefacts from axis labels — see
        // formatAxisValue() above. ECharts hands us a number directly.
        axisLabel: { formatter: (v: number) => formatAxisValue(v) },
        splitLine: { show: gridDensity !== 'off' },
        minorTick: { show: gridDensity === 'detailed' },
        minorSplitLine: { show: gridDensity === 'detailed' },
        // Pinned to the data span (computed above). markArea renders
        // inside this range but does NOT expand it — so reference bands
        // appear as horizontal stripes overlapping the data without
        // forcing a zoomed-out view. Log axis falls back to ECharts'
        // automatic positive-only fit.
        min: yType === 'value' ? yMin : undefined,
        max: yType === 'value' ? yMax : undefined,
        name: unit,
        nameLocation: 'end',
        nameTextStyle: { padding: [0, 0, 0, 8] }
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      } as any),
      series: [
        {
          // Series type: 'line' (default) | 'area' (line + filled below) |
          // 'bar' (vertical bars at each timestamp). Bar charts ignore
          // smoothing/step (ECharts handles the geometry separately).
          type: seriesType === 'bar' ? 'bar' : 'line',
          smooth: seriesType === 'bar' ? false : smooth,
          // Step interpolation translates to ECharts' `step` field on line
          // series only. `none` falls through to a plain Bezier or straight
          // segment depending on the smooth toggle.
          ...(seriesType !== 'bar' && stepKind !== 'none'
            ? { step: stepKind === 'start' ? 'start' as const : stepKind === 'middle' ? 'middle' as const : 'end' as const }
            : {}),
          showSymbol: seriesType === 'bar' ? false : showSymbols,
          symbolSize,
          // `connectNulls` honoured here so the user can choose between a
          // gappy line (default-ish for missing data) and a solid line
          // bridging gaps. ECharts ignores it for bar charts.
          connectNulls: connectNulls,
          // Each datum carries the optional flag-tinted itemStyle so
          // colorByFlag works at point granularity. The series-level colour
          // still wins when the flag is unknown / disabled.
          data: sorted.map((p) => {
            const tint = colorByFlag ? flagColor(p.flag) : undefined;
            return tint
              ? { value: [p.date, p.value], itemStyle: { color: tint } }
              : [p.date, p.value];
          }),
          // Area fill for the 'area' variant. ECharts renders the band
          // beneath the line using the series colour at low alpha.
          areaStyle: seriesType === 'area' ? { opacity: 0.18 } : undefined,
          markArea: markAreaData.length ? { silent: true, data: markAreaData } : undefined,
          markLine: markLineConfig,
          // Optional reference lines. We collect everything the user
          // enabled into a single markLine block — ECharts handles the
          // mix of vertical (xAxis) and horizontal (yAxis) anchors.
          // Optional min/max markPoints — bubble labels at the highest and
          // lowest reading. Useful for spotting extremes on busy charts.
          markPoint: showMinMaxMarkers && yVals.length > 0
            ? {
                symbol: 'pin',
                symbolSize: 28,
                label: { fontSize: 10, color: '#fff' },
                data: [
                  { type: 'max', name: 'Max', itemStyle: { color: themeVar('--crit', 'rgb(220 38 38)') } },
                  { type: 'min', name: 'Min', itemStyle: { color: themeVar('--ok',   'rgb(22 163 74)') } },
                ],
              }
            : undefined,
          lineStyle: { width: lineWidth },
          label: showValues
            ? {
                show: true,
                position: 'top',
                distance: 8,
                // Contrasting pill so the value reads cleanly on top of the
                // line / reference band — without it the number gets lost
                // in either the data line or the green band.
                backgroundColor: labelBg,
                color: labelFg,
                borderColor: labelBorder,
                borderWidth: 1,
                borderRadius: 4,
                padding: [2, 5],
                fontSize: 10,
                fontWeight: 600,
                // ECharts types `value` as a wide union; pull the second
                // tuple member at runtime and tolerate other shapes.
                formatter: (p: { value: unknown }) => {
                  const v = Array.isArray(p.value) ? p.value[1] : p.value;
                  return typeof v === 'number' && Number.isFinite(v) ? formatAxisValue(v) : '';
                }
              }
            : { show: false }
        },
        // Optional second series: linear-regression trend line. Rendered
        // dashed and slightly thinner than the primary so it reads as a
        // guide rather than competing with the data. Hidden from the
        // legend / tooltip (silent).
        ...((() => {
          if (!showTrendLine) return [];
          const tl = trendlineEndpoints(sorted);
          if (!tl) return [];
          return [{
            type: 'line' as const,
            silent: true,
            showSymbol: false,
            data: [tl.from, tl.to],
            lineStyle: {
              type: 'dashed' as const,
              width: Math.max(1, lineWidth * 0.7),
              color: themeVar('--fg-3', 'rgb(107 114 128)'),
              opacity: 0.85,
            },
            z: 1,
          }];
        })())
      ],
      animation: animation !== 'off',
      animationDuration:
        animation === 'fast'   ? 100 :
        animation === 'slow'   ? 600 :
        animation === 'normal' ? 250 : 0,
    };
  }

  function init() {
    if (!host) return;
    ensureThemesRegistered();
    chart = echarts.init(host, activeThemeName(), { renderer: 'canvas' });
    chart.setOption(buildOption());
  }

  function rebuild() {
    if (!chart || !host) return;
    chart.dispose();
    reRegisterThemes();
    chart = echarts.init(host, activeThemeName(), { renderer: 'canvas' });
    chart.setOption(buildOption());
  }

  // ResizeObserver tracks the host element directly, so the chart redraws
  // whenever its container changes width — opening/closing the filter
  // panel, toggling the navbar's icon-only mode, isolating a card, etc.
  // The plain `window resize` listener alone misses every layout-driven
  // size change because the OS window's bounds haven't shifted.
  let hostResizeObserver: ResizeObserver | null = null;
  let resizeRaf = 0;
  function onResize() {
    if (resizeRaf) cancelAnimationFrame(resizeRaf);
    resizeRaf = requestAnimationFrame(() => {
      chart?.resize();
      resizeRaf = 0;
    });
  }

  // ── Toolbar handlers (zoom, copy, export, scale, labels, smooth) ────
  function zoomIn() {
    if (!chart) return;
    // Tighten the visible window by 25% from each end. ECharts dispatches
    // a `dataZoom` action which is also picked up by the inside-zoom for
    // wheel parity.
    const opt = chart.getOption() as { dataZoom?: { start?: number; end?: number }[] };
    const s = opt.dataZoom?.[0]?.start ?? 0;
    const e = opt.dataZoom?.[0]?.end ?? 100;
    const span = e - s;
    const nextSpan = Math.max(span * 0.6, 4);
    const center = (s + e) / 2;
    chart.dispatchAction({
      type: 'dataZoom',
      start: Math.max(0, center - nextSpan / 2),
      end: Math.min(100, center + nextSpan / 2)
    });
  }
  function zoomOut() {
    if (!chart) return;
    const opt = chart.getOption() as { dataZoom?: { start?: number; end?: number }[] };
    const s = opt.dataZoom?.[0]?.start ?? 0;
    const e = opt.dataZoom?.[0]?.end ?? 100;
    const span = e - s;
    const nextSpan = Math.min(span * 1.66, 100);
    const center = (s + e) / 2;
    chart.dispatchAction({
      type: 'dataZoom',
      start: Math.max(0, center - nextSpan / 2),
      end: Math.min(100, center + nextSpan / 2)
    });
  }
  function resetZoom() {
    if (!chart) return;
    // Reset = back to the *default tight view*, not the maxed-out 0..100%
    // bounds. Users still have +/- and slider drag to push beyond it.
    chart.dispatchAction({
      type: 'dataZoom',
      start: xZoomDefault.start,
      end:   xZoomDefault.end
    });
  }

  function dataUrl(): string | null {
    if (!chart) return null;
    return chart.getDataURL({
      type: 'png',
      pixelRatio: 2,
      backgroundColor: getComputedStyle(document.documentElement)
        .getPropertyValue('--bg-1')
        .trim()
        ? `rgb(${getComputedStyle(document.documentElement).getPropertyValue('--bg-1').trim()})`
        : '#0b0d10'
    });
  }

  async function copyPng() {
    const url = dataUrl();
    if (!url) return;
    try {
      const blob = await (await fetch(url)).blob();
      // ClipboardItem is supported in Chromium-based WebViews (which Tauri
      // bundles on Windows + macOS), Edge WebView2, and modern Safari.
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const ClipboardItemCtor = (window as any).ClipboardItem;
      if (!ClipboardItemCtor || !navigator.clipboard?.write) {
        throw new Error('Clipboard image API not available — use Save PNG instead.');
      }
      await navigator.clipboard.write([new ClipboardItemCtor({ 'image/png': blob })]);
      toasts.success('Chart copied', 'PNG is on your clipboard.');
    } catch (e) {
      toasts.error(e);
    }
  }

  function downloadPng() {
    const url = dataUrl();
    if (!url) return;
    const a = document.createElement('a');
    a.href = url;
    a.download = `${exportName}.png`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }

  function exportCsvFromChart() {
    if (!chart) return;
    const sorted = [...points].sort((a: Point, b: Point) => a.date.localeCompare(b.date));
    const lines = ['date,value', ...sorted.map((p) => `${p.date},${p.value}`)];
    const blob = new Blob([lines.join('\n')], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${exportName}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  onMount(() => {
    init();
    window.addEventListener('resize', onResize);
    document.documentElement.addEventListener('theme-change', rebuild);
    // Watch the host's box for any width/height change — covers layout-
    // driven resizes (panel toggles, sidebar collapses) that don't fire
    // a window resize event.
    if (host && typeof ResizeObserver !== 'undefined') {
      hostResizeObserver = new ResizeObserver(() => onResize());
      hostResizeObserver.observe(host);
    }
  });

  onDestroy(() => {
    if (resizeRaf) cancelAnimationFrame(resizeRaf);
    hostResizeObserver?.disconnect();
    hostResizeObserver = null;
    chart?.dispose();
    chart = null;
    if (typeof window !== 'undefined') {
      window.removeEventListener('resize', onResize);
      document.documentElement.removeEventListener('theme-change', rebuild);
    }
  });

  $effect(() => {
    // re-render whenever the inputs OR display toggles change
    if (chart) chart.setOption(buildOption(), true);
    // touch every reactive dependency so this effect re-runs on changes:
    void showValues; void smooth; void scale; void showBands; void showSymbols;
    void useNicknames; void lineWidth; void symbolSize; void gridDensity;
    void dateFormat; void showTodayLine; void showTrendLine;
    void showMeanLine; void showMinMaxMarkers;
    void seriesType; void stepKind; void colorByFlag; void connectNulls;
    void bandOpacityPct; void animation; void tooltipMode; void showTitle;
    void xLabelMode; void timeWindow;
    void X_AXIS_PAD; void Y_AXIS_PAD; void Y_DEFAULT_VISIBLE_PAD; void yZoomDefault;
    void points; void refBands; void xAxisRange;
    void prefs.showXSlider; void prefs.showYSlider; void prefs.scrollZoom;
  });
</script>

<div class="ts-shell">
  <div class="ts-toolbar">
    <button type="button" class="ts-btn" onclick={zoomIn}     title="Zoom in"  aria-label="Zoom in">＋</button>
    <button type="button" class="ts-btn" onclick={zoomOut}    title="Zoom out" aria-label="Zoom out">−</button>
    <button type="button" class="ts-btn" onclick={resetZoom}  title="Reset zoom" aria-label="Reset zoom">↺</button>

    <span class="ts-sep" aria-hidden="true"></span>

    <!-- Each toolbar toggle writes to the global chart-prefs store, so the
         choice sticks across pages and reloads. Reset everything from
         Settings ▸ Charts ▸ Reset chart defaults. -->
    <!-- Toolbar mutations write to whichever `prefs` instance was passed
         in — global by default, or a `LocalChartPrefs` snapshot when the
         caller flipped this chart into "isolated settings" mode. The
         tooltips read "global / chart-local" depending on the binding. -->
    <button type="button"
            class="ts-btn {showValues ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowValues(!showValues)}
            title="Toggle value labels on each point. {persistHint}">
      123
    </button>
    <button type="button"
            class="ts-btn {smooth ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setSmooth(!smooth)}
            title="Toggle smoothed line. {persistHint}">
      ∿
    </button>
    <button type="button"
            class="ts-btn {showSymbols ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowSymbols(!showSymbols)}
            title="Toggle the dot at each reading. {persistHint}">
      •
    </button>
    <button type="button"
            class="ts-btn {showBands ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowBands(!showBands)}
            title="Toggle reference band(s). {persistHint}">
      ⇆
    </button>
    <button type="button"
            class="ts-btn {scale === 'log' ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setScale(scale === 'linear' ? 'log' : 'linear')}
            title="Toggle linear / logarithmic Y-axis (log requires all values > 0). {persistHint}">
      log
    </button>
    <button type="button"
            class="ts-btn {showTrendLine ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowTrendLine(!showTrendLine)}
            title="Overlay a linear-regression trend line. {persistHint}">
      ⤴
    </button>
    <button type="button"
            class="ts-btn {showTodayLine ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowTodayLine(!showTodayLine)}
            title="Show a vertical reference line at today's date. {persistHint}">
      📍
    </button>
    <button type="button"
            class="ts-btn {showMeanLine ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowMeanLine(!showMeanLine)}
            title="Show a horizontal line at the mean of visible readings. {persistHint}">
      μ
    </button>
    <button type="button"
            class="ts-btn {showMinMaxMarkers ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setShowMinMaxMarkers(!showMinMaxMarkers)}
            title="Pin the highest and lowest readings. {persistHint}">
      ⇕
    </button>
    <button type="button"
            class="ts-btn {colorByFlag ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setColorByFlag(!colorByFlag)}
            title="Tint each point by its flag (low / normal / high / critical). {persistHint}">
      🎯
    </button>
    <button type="button"
            class="ts-btn {useNicknames ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setUseNicknames(!useNicknames)}
            title="Show report nicknames on the X axis (when set on the source report). Falls back to the date for unlabelled reports. {persistHint}">
      🏷
    </button>
    <button type="button"
            class="ts-btn {prefs.scrollZoom ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setScrollZoom(!prefs.scrollZoom)}
            title="Toggle mouse-wheel zoom inside the chart (off by default — wheel scrolls the page). {persistHint}">
      🖱
    </button>
    <!-- Cycle X-axis label mode: auto → rotate → hide → auto. The icon
         glyph reflects the current state so the chart's chrome doubles
         as a status indicator. -->
    <button type="button"
            class="ts-btn {xLabelMode !== 'auto' ? 'ts-btn--on' : ''}"
            onclick={() => prefs.setXLabelMode(
              xLabelMode === 'auto'   ? 'rotate' :
              xLabelMode === 'rotate' ? 'hide'   : 'auto'
            )}
            title={
              xLabelMode === 'auto'
                ? 'X-axis labels: auto-hide overlaps. Click to tilt (rotate).'
                : xLabelMode === 'rotate'
                  ? 'X-axis labels: rotated 35°. Click to hide entirely (date still on hover).'
                  : 'X-axis labels: hidden — hover the line for the exact date. Click to restore auto.'
            }>
      {xLabelMode === 'auto' ? '⇲' : xLabelMode === 'rotate' ? '⤢' : '∅'}
    </button>

    <span class="ts-sep" aria-hidden="true"></span>

    <button type="button" class="ts-btn" onclick={copyPng}    title="Copy chart to clipboard as PNG">⧉ Copy</button>
    <button type="button" class="ts-btn" onclick={downloadPng} title="Save chart as PNG">⤓ PNG</button>
    <button type="button" class="ts-btn" onclick={exportCsvFromChart} title="Export plotted points as CSV">⤓ CSV</button>
  </div>

  <div class="ts-host" style="height: {height}px;" bind:this={host}></div>
</div>

<style>
  .ts-shell {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    width: 100%;
    /* Without min-width: 0 a flex / grid item defaults to min-width: auto
       which expands to its content's intrinsic size — that's what was
       blocking the chart from SHRINKING when the window narrowed (the
       canvas already had a wider intrinsic size, and the host
       inherited it). With this, the host honours the parent's bounds
       in both directions. */
    min-width: 0;
  }
  .ts-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.7rem;
    /* Same min-width: 0 reasoning — let the toolbar shrink + wrap rather
       than pinning the parent open at toolbar-content width. */
    min-width: 0;
  }
  .ts-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.18rem 0.5rem;
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    cursor: pointer;
    line-height: 1;
    font-family: inherit;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
    min-width: 1.6rem;
    justify-content: center;
  }
  .ts-btn:hover {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--accent) / 0.6);
  }
  .ts-btn--on {
    background: rgb(var(--accent) / 0.15);
    color: rgb(var(--accent));
    border-color: rgb(var(--accent) / 0.45);
  }
  .ts-sep {
    width: 1px;
    height: 16px;
    background: rgb(var(--line));
    margin: 0 0.25rem;
  }
  .ts-host {
    width: 100%;
    /* The ECharts canvas inside has explicit pixel dimensions; without
       overflow:hidden the host briefly inherits the canvas's old (wider)
       size during resize and refuses to shrink below it. Clipping the
       overflow lets the host honour the parent's narrower bound, then
       the ResizeObserver fires `chart.resize()` to shrink the canvas
       to match. */
    overflow: hidden;
    min-width: 0;
  }
</style>
