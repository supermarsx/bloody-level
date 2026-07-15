// Chart-rendering preferences persisted to the encrypted settings store.
// Each key lives under `chart.<name>` so future chart-related toggles can
// share the namespace cleanly. The store loads once at app boot (via the
// `load()` call from the Settings page or any chart instance) and emits
// reactive updates to consumers using Svelte 5 runes.
//
// Design: every toggle the chart toolbar exposes is BACKED by a global
// default here. Flipping a toggle in the toolbar writes through to the
// global default (debounced). This keeps the model simple — there's exactly
// one source of truth for each preference, and the user can see and
// reset everything from Settings ▸ Charts.

import * as settings from '$api/settings';
import { setDebounced } from '$api/debounced-settings';

const KEY_X_SLIDER          = 'chart.show_x_slider';
const KEY_Y_SLIDER          = 'chart.show_y_slider';
const KEY_SCROLL_ZOOM       = 'chart.scroll_zoom';
const KEY_REF_SOURCE        = 'chart.reference_source';

// Per-chart toggles, hoisted to global defaults.
const KEY_SHOW_VALUES       = 'chart.show_values';
const KEY_SMOOTH            = 'chart.smooth';
const KEY_SCALE             = 'chart.scale';
const KEY_SHOW_BANDS        = 'chart.show_bands';
const KEY_USE_NICKNAMES     = 'chart.use_nicknames';
const KEY_SHOW_SYMBOLS      = 'chart.show_symbols';

// Visual style knobs.
const KEY_LINE_WIDTH        = 'chart.line_width';
const KEY_SYMBOL_SIZE       = 'chart.symbol_size';
const KEY_GRID_DENSITY      = 'chart.grid_density';
const KEY_DATE_FORMAT       = 'chart.date_format';

// Padding / zoom defaults — fractions, stored as percentages so the
// settings UI can render plain integers.
const KEY_X_AXIS_PAD_PCT    = 'chart.x_axis_pad_pct';
const KEY_Y_AXIS_PAD_PCT    = 'chart.y_axis_pad_pct';
const KEY_Y_VIS_PAD_PCT     = 'chart.y_visible_pad_pct';

// Reference markers.
const KEY_SHOW_TODAY_LINE   = 'chart.show_today_line';
const KEY_SHOW_TREND_LINE   = 'chart.show_trend_line';
const KEY_SHOW_MEAN_LINE    = 'chart.show_mean_line';
const KEY_SHOW_MIN_MAX      = 'chart.show_min_max_markers';

// Series rendering style.
const KEY_SERIES_TYPE       = 'chart.series_type';
const KEY_STEP              = 'chart.step';
const KEY_COLOR_BY_FLAG     = 'chart.color_by_flag';
const KEY_CONNECT_NULLS     = 'chart.connect_nulls';
const KEY_BAND_OPACITY      = 'chart.band_opacity_pct';
const KEY_ANIMATION         = 'chart.animation';

// Tooltip + interaction.
const KEY_TOOLTIP_MODE      = 'chart.tooltip_mode';
const KEY_SHOW_TITLE        = 'chart.show_title';
// How the X-axis handles overlapping labels:
//   - `auto`   : hide overlapping labels; tooltip shows the date on hover
//   - `rotate` : tilt labels 35° so more fit, plus auto-hide for what
//                still collides
//   - `hide`   : never render axis labels at all (cleanest visual; date
//                still appears in the hover tooltip)
const KEY_X_LABEL_MODE      = 'chart.x_label_mode';

// Default time window — shrinks the visible X range to the most recent N
// months at chart open. Zoom-out via slider/wheel still reveals full history.
const KEY_TIME_WINDOW       = 'chart.time_window';

/**
 * Where flag derivation and chart bands draw their reference range from:
 *
 *   - `auto` (default): library wins when the ontology has any usable
 *     reference (sex-keyed, tier, cycle phase, or universal default);
 *     falls back to the printed range stored on the row otherwise.
 *   - `library`: always derive from the analyte ontology, even when the
 *     PDF printed a different range. Ignores the parser-stored flag.
 *   - `printed`: always trust the lab's printed range, even when the
 *     ontology has a sex/age/cycle-aware default that disagrees.
 */
export type ReferenceSource = 'auto' | 'library' | 'printed';
export type Scale           = 'linear' | 'log';
export type GridDensity     = 'off' | 'standard' | 'detailed';
export type DateFormat      = 'iso' | 'short' | 'monthYear' | 'numeric';
export type SeriesType      = 'line' | 'area' | 'bar';
export type StepKind        = 'none' | 'start' | 'middle' | 'end';
export type AnimationSpeed  = 'off' | 'fast' | 'normal' | 'slow';
export type TooltipMode     = 'axis' | 'item';
export type XLabelMode      = 'auto' | 'rotate' | 'hide';
/** Default time-window shortcut applied at chart open. `all` means no
 *  filtering — render every point. The numeric variants window down to
 *  the last N days, but the user can still zoom out to see history. */
export type TimeWindow      = 'all' | '30d' | '90d' | '6m' | '1y' | '2y' | '5y';

// Defaults — chosen so the chart still reads as a clean line at first paint.
const DEFAULTS = {
  showXSlider:          false,
  showYSlider:          false,
  scrollZoom:           false,
  referenceSource:      'auto'   as ReferenceSource,
  showValues:           false,
  smooth:               false,
  scale:                'linear' as Scale,
  showBands:            true,
  useNicknames:         false,
  showSymbols:          true,
  lineWidth:            2.4,           // px
  symbolSize:           7,             // px diameter
  gridDensity:          'standard' as GridDensity,
  dateFormat:           'iso'      as DateFormat,
  xAxisPadPct:          5,             // %
  yAxisPadPct:          40,            // %
  yVisiblePadPct:       3,             // %
  showTodayLine:        false,
  showTrendLine:        false,
  showMeanLine:         false,
  showMinMaxMarkers:    false,
  seriesType:           'line'   as SeriesType,
  step:                 'none'   as StepKind,
  colorByFlag:          false,
  connectNulls:         true,
  bandOpacityPct:       100,            // 100 = use the bundled token alpha as-is
  animation:            'normal' as AnimationSpeed,
  tooltipMode:          'axis'   as TooltipMode,
  showTitle:            true,
  xLabelMode:           'auto'   as XLabelMode,
  timeWindow:           'all'    as TimeWindow,
};

class ChartPrefs {
  // ── existing toggles ───────────────────────────────────────────────
  showXSlider     = $state(DEFAULTS.showXSlider);
  showYSlider     = $state(DEFAULTS.showYSlider);
  scrollZoom      = $state(DEFAULTS.scrollZoom);
  referenceSource = $state<ReferenceSource>(DEFAULTS.referenceSource);

  // ── per-chart toggles, persisted ───────────────────────────────────
  showValues   = $state(DEFAULTS.showValues);
  smooth       = $state(DEFAULTS.smooth);
  scale        = $state<Scale>(DEFAULTS.scale);
  showBands    = $state(DEFAULTS.showBands);
  useNicknames = $state(DEFAULTS.useNicknames);
  showSymbols  = $state(DEFAULTS.showSymbols);

  // ── visual style ───────────────────────────────────────────────────
  lineWidth   = $state(DEFAULTS.lineWidth);
  symbolSize  = $state(DEFAULTS.symbolSize);
  gridDensity = $state<GridDensity>(DEFAULTS.gridDensity);
  dateFormat  = $state<DateFormat>(DEFAULTS.dateFormat);

  // ── padding / zoom ─────────────────────────────────────────────────
  xAxisPadPct    = $state(DEFAULTS.xAxisPadPct);
  yAxisPadPct    = $state(DEFAULTS.yAxisPadPct);
  yVisiblePadPct = $state(DEFAULTS.yVisiblePadPct);

  // ── reference markers ──────────────────────────────────────────────
  showTodayLine     = $state(DEFAULTS.showTodayLine);
  showTrendLine     = $state(DEFAULTS.showTrendLine);
  showMeanLine      = $state(DEFAULTS.showMeanLine);
  showMinMaxMarkers = $state(DEFAULTS.showMinMaxMarkers);

  // ── series rendering ───────────────────────────────────────────────
  seriesType     = $state<SeriesType>(DEFAULTS.seriesType);
  step           = $state<StepKind>(DEFAULTS.step);
  colorByFlag    = $state(DEFAULTS.colorByFlag);
  connectNulls   = $state(DEFAULTS.connectNulls);
  bandOpacityPct = $state(DEFAULTS.bandOpacityPct);
  animation      = $state<AnimationSpeed>(DEFAULTS.animation);

  // ── tooltip + chrome ───────────────────────────────────────────────
  tooltipMode = $state<TooltipMode>(DEFAULTS.tooltipMode);
  showTitle   = $state(DEFAULTS.showTitle);
  xLabelMode  = $state<XLabelMode>(DEFAULTS.xLabelMode);

  // ── default time window ────────────────────────────────────────────
  timeWindow = $state<TimeWindow>(DEFAULTS.timeWindow);

  loaded = $state(false);

  /** Hydrate from the settings table. Idempotent. */
  async load() {
    if (this.loaded) return;
    try {
      // Load each preference; tolerate missing or wrongly-typed values by
      // simply leaving the in-memory default intact.
      const reads = await Promise.all([
        settings.get<boolean>(KEY_X_SLIDER),
        settings.get<boolean>(KEY_Y_SLIDER),
        settings.get<boolean>(KEY_SCROLL_ZOOM),
        settings.get<ReferenceSource>(KEY_REF_SOURCE),
        settings.get<boolean>(KEY_SHOW_VALUES),
        settings.get<boolean>(KEY_SMOOTH),
        settings.get<Scale>(KEY_SCALE),
        settings.get<boolean>(KEY_SHOW_BANDS),
        settings.get<boolean>(KEY_USE_NICKNAMES),
        settings.get<boolean>(KEY_SHOW_SYMBOLS),
        settings.get<number>(KEY_LINE_WIDTH),
        settings.get<number>(KEY_SYMBOL_SIZE),
        settings.get<GridDensity>(KEY_GRID_DENSITY),
        settings.get<DateFormat>(KEY_DATE_FORMAT),
        settings.get<number>(KEY_X_AXIS_PAD_PCT),
        settings.get<number>(KEY_Y_AXIS_PAD_PCT),
        settings.get<number>(KEY_Y_VIS_PAD_PCT),
        settings.get<boolean>(KEY_SHOW_TODAY_LINE),
        settings.get<boolean>(KEY_SHOW_TREND_LINE),
        settings.get<boolean>(KEY_SHOW_MEAN_LINE),
        settings.get<boolean>(KEY_SHOW_MIN_MAX),
        settings.get<SeriesType>(KEY_SERIES_TYPE),
        settings.get<StepKind>(KEY_STEP),
        settings.get<boolean>(KEY_COLOR_BY_FLAG),
        settings.get<boolean>(KEY_CONNECT_NULLS),
        settings.get<number>(KEY_BAND_OPACITY),
        settings.get<AnimationSpeed>(KEY_ANIMATION),
        settings.get<TooltipMode>(KEY_TOOLTIP_MODE),
        settings.get<boolean>(KEY_SHOW_TITLE),
        settings.get<XLabelMode>(KEY_X_LABEL_MODE),
        settings.get<TimeWindow>(KEY_TIME_WINDOW),
      ]);
      const [x, y, z, r, sv, sm, sc, sb, un, ssym, lw, ssz, gd, df, xpp, ypp, yvpp, stl, str,
             sml, smm, st, stp, cbf, cn, bo, an, tm, sti, xlm, tw] = reads;
      if (typeof x === 'boolean')  this.showXSlider     = x;
      if (typeof y === 'boolean')  this.showYSlider     = y;
      if (typeof z === 'boolean')  this.scrollZoom      = z;
      if (r === 'auto' || r === 'library' || r === 'printed') this.referenceSource = r;
      if (typeof sv === 'boolean') this.showValues   = sv;
      if (typeof sm === 'boolean') this.smooth       = sm;
      if (sc === 'linear' || sc === 'log') this.scale = sc;
      if (typeof sb === 'boolean') this.showBands    = sb;
      if (typeof un === 'boolean') this.useNicknames = un;
      if (typeof ssym === 'boolean') this.showSymbols = ssym;
      if (typeof lw === 'number'  && lw > 0)  this.lineWidth  = lw;
      if (typeof ssz === 'number' && ssz > 0) this.symbolSize = ssz;
      if (gd === 'off' || gd === 'standard' || gd === 'detailed') this.gridDensity = gd;
      if (df === 'iso' || df === 'short' || df === 'monthYear' || df === 'numeric') this.dateFormat = df;
      if (typeof xpp  === 'number' && xpp  >= 0) this.xAxisPadPct    = xpp;
      if (typeof ypp  === 'number' && ypp  >= 0) this.yAxisPadPct    = ypp;
      if (typeof yvpp === 'number' && yvpp >= 0) this.yVisiblePadPct = yvpp;
      if (typeof stl === 'boolean') this.showTodayLine = stl;
      if (typeof str === 'boolean') this.showTrendLine = str;
      if (typeof sml === 'boolean') this.showMeanLine  = sml;
      if (typeof smm === 'boolean') this.showMinMaxMarkers = smm;
      if (st === 'line' || st === 'area' || st === 'bar') this.seriesType = st;
      if (stp === 'none' || stp === 'start' || stp === 'middle' || stp === 'end') this.step = stp;
      if (typeof cbf === 'boolean') this.colorByFlag  = cbf;
      if (typeof cn  === 'boolean') this.connectNulls = cn;
      if (typeof bo === 'number' && bo >= 0 && bo <= 200) this.bandOpacityPct = bo;
      if (an === 'off' || an === 'fast' || an === 'normal' || an === 'slow') this.animation = an;
      if (tm === 'axis' || tm === 'item') this.tooltipMode = tm;
      if (typeof sti === 'boolean') this.showTitle = sti;
      if (xlm === 'auto' || xlm === 'rotate' || xlm === 'hide') this.xLabelMode = xlm;
      if (tw === 'all' || tw === '30d' || tw === '90d' || tw === '6m' || tw === '1y' || tw === '2y' || tw === '5y') this.timeWindow = tw;
    } catch { /* stay on defaults */ }
    this.loaded = true;
  }

  // In-memory state changes are reflected immediately so charts re-render
  // without latency; the persisted save funnels through the 1.5s debouncer
  // so a quick string of toggles still produces just one toast.
  setX(v: boolean)               { this.showXSlider     = v; setDebounced(KEY_X_SLIDER, v); }
  setY(v: boolean)               { this.showYSlider     = v; setDebounced(KEY_Y_SLIDER, v); }
  setScrollZoom(v: boolean)      { this.scrollZoom      = v; setDebounced(KEY_SCROLL_ZOOM, v); }
  setReferenceSource(v: ReferenceSource) { this.referenceSource = v; setDebounced(KEY_REF_SOURCE, v); }

  setShowValues(v: boolean)      { this.showValues   = v; setDebounced(KEY_SHOW_VALUES, v); }
  setSmooth(v: boolean)          { this.smooth       = v; setDebounced(KEY_SMOOTH, v); }
  setScale(v: Scale)             { this.scale        = v; setDebounced(KEY_SCALE, v); }
  setShowBands(v: boolean)       { this.showBands    = v; setDebounced(KEY_SHOW_BANDS, v); }
  setUseNicknames(v: boolean)    { this.useNicknames = v; setDebounced(KEY_USE_NICKNAMES, v); }
  setShowSymbols(v: boolean)     { this.showSymbols  = v; setDebounced(KEY_SHOW_SYMBOLS, v); }

  setLineWidth(v: number)        { this.lineWidth   = v; setDebounced(KEY_LINE_WIDTH, v); }
  setSymbolSize(v: number)       { this.symbolSize  = v; setDebounced(KEY_SYMBOL_SIZE, v); }
  setGridDensity(v: GridDensity) { this.gridDensity = v; setDebounced(KEY_GRID_DENSITY, v); }
  setDateFormat(v: DateFormat)   { this.dateFormat  = v; setDebounced(KEY_DATE_FORMAT, v); }

  setXAxisPadPct(v: number)      { this.xAxisPadPct    = v; setDebounced(KEY_X_AXIS_PAD_PCT, v); }
  setYAxisPadPct(v: number)      { this.yAxisPadPct    = v; setDebounced(KEY_Y_AXIS_PAD_PCT, v); }
  setYVisiblePadPct(v: number)   { this.yVisiblePadPct = v; setDebounced(KEY_Y_VIS_PAD_PCT, v); }

  setShowTodayLine(v: boolean)      { this.showTodayLine     = v; setDebounced(KEY_SHOW_TODAY_LINE, v); }
  setShowTrendLine(v: boolean)      { this.showTrendLine     = v; setDebounced(KEY_SHOW_TREND_LINE, v); }
  setShowMeanLine(v: boolean)       { this.showMeanLine      = v; setDebounced(KEY_SHOW_MEAN_LINE, v); }
  setShowMinMaxMarkers(v: boolean)  { this.showMinMaxMarkers = v; setDebounced(KEY_SHOW_MIN_MAX, v); }

  setSeriesType(v: SeriesType)      { this.seriesType     = v; setDebounced(KEY_SERIES_TYPE, v); }
  setStep(v: StepKind)              { this.step           = v; setDebounced(KEY_STEP, v); }
  setColorByFlag(v: boolean)        { this.colorByFlag    = v; setDebounced(KEY_COLOR_BY_FLAG, v); }
  setConnectNulls(v: boolean)       { this.connectNulls   = v; setDebounced(KEY_CONNECT_NULLS, v); }
  setBandOpacityPct(v: number)      { this.bandOpacityPct = v; setDebounced(KEY_BAND_OPACITY, v); }
  setAnimation(v: AnimationSpeed)   { this.animation      = v; setDebounced(KEY_ANIMATION, v); }
  setTooltipMode(v: TooltipMode)    { this.tooltipMode    = v; setDebounced(KEY_TOOLTIP_MODE, v); }
  setShowTitle(v: boolean)          { this.showTitle      = v; setDebounced(KEY_SHOW_TITLE, v); }
  setXLabelMode(v: XLabelMode)      { this.xLabelMode     = v; setDebounced(KEY_X_LABEL_MODE, v); }
  setTimeWindow(v: TimeWindow)      { this.timeWindow     = v; setDebounced(KEY_TIME_WINDOW, v); }

  /** Reset every chart preference to the bundled defaults. Persisted via
   *  the debouncer so the user gets a single toast. */
  resetAll() {
    this.setX(DEFAULTS.showXSlider);
    this.setY(DEFAULTS.showYSlider);
    this.setScrollZoom(DEFAULTS.scrollZoom);
    this.setReferenceSource(DEFAULTS.referenceSource);
    this.setShowValues(DEFAULTS.showValues);
    this.setSmooth(DEFAULTS.smooth);
    this.setScale(DEFAULTS.scale);
    this.setShowBands(DEFAULTS.showBands);
    this.setUseNicknames(DEFAULTS.useNicknames);
    this.setShowSymbols(DEFAULTS.showSymbols);
    this.setLineWidth(DEFAULTS.lineWidth);
    this.setSymbolSize(DEFAULTS.symbolSize);
    this.setGridDensity(DEFAULTS.gridDensity);
    this.setDateFormat(DEFAULTS.dateFormat);
    this.setXAxisPadPct(DEFAULTS.xAxisPadPct);
    this.setYAxisPadPct(DEFAULTS.yAxisPadPct);
    this.setYVisiblePadPct(DEFAULTS.yVisiblePadPct);
    this.setShowTodayLine(DEFAULTS.showTodayLine);
    this.setShowTrendLine(DEFAULTS.showTrendLine);
    this.setShowMeanLine(DEFAULTS.showMeanLine);
    this.setShowMinMaxMarkers(DEFAULTS.showMinMaxMarkers);
    this.setSeriesType(DEFAULTS.seriesType);
    this.setStep(DEFAULTS.step);
    this.setColorByFlag(DEFAULTS.colorByFlag);
    this.setConnectNulls(DEFAULTS.connectNulls);
    this.setBandOpacityPct(DEFAULTS.bandOpacityPct);
    this.setAnimation(DEFAULTS.animation);
    this.setTooltipMode(DEFAULTS.tooltipMode);
    this.setShowTitle(DEFAULTS.showTitle);
    this.setXLabelMode(DEFAULTS.xLabelMode);
    this.setTimeWindow(DEFAULTS.timeWindow);
  }
}

export const chartPrefs = new ChartPrefs();
export const CHART_DEFAULTS = DEFAULTS;

/**
 * Drop-in stand-in for `chartPrefs` that keeps its values in memory only.
 * Used by the Compare page when a card has "custom settings" enabled — it
 * gets its own LocalChartPrefs instance so its toolbar mutations and
 * configuration inputs don't leak into the global store. The shape is
 * structurally identical to `ChartPrefs` so `TimeSeries.svelte` can take
 * either via the same `prefs` prop.
 *
 * Constructed by snapshotting the global state at the moment of "go local",
 * so the user transitions seamlessly from the inherited defaults to their
 * own copy.
 */
export class LocalChartPrefs {
  showXSlider     = $state(chartPrefs.showXSlider);
  showYSlider     = $state(chartPrefs.showYSlider);
  scrollZoom      = $state(chartPrefs.scrollZoom);
  referenceSource = $state<ReferenceSource>(chartPrefs.referenceSource);

  showValues   = $state(chartPrefs.showValues);
  smooth       = $state(chartPrefs.smooth);
  scale        = $state<Scale>(chartPrefs.scale);
  showBands    = $state(chartPrefs.showBands);
  useNicknames = $state(chartPrefs.useNicknames);
  showSymbols  = $state(chartPrefs.showSymbols);

  lineWidth   = $state(chartPrefs.lineWidth);
  symbolSize  = $state(chartPrefs.symbolSize);
  gridDensity = $state<GridDensity>(chartPrefs.gridDensity);
  dateFormat  = $state<DateFormat>(chartPrefs.dateFormat);

  xAxisPadPct    = $state(chartPrefs.xAxisPadPct);
  yAxisPadPct    = $state(chartPrefs.yAxisPadPct);
  yVisiblePadPct = $state(chartPrefs.yVisiblePadPct);

  showTodayLine     = $state(chartPrefs.showTodayLine);
  showTrendLine     = $state(chartPrefs.showTrendLine);
  showMeanLine      = $state(chartPrefs.showMeanLine);
  showMinMaxMarkers = $state(chartPrefs.showMinMaxMarkers);

  seriesType     = $state<SeriesType>(chartPrefs.seriesType);
  step           = $state<StepKind>(chartPrefs.step);
  colorByFlag    = $state(chartPrefs.colorByFlag);
  connectNulls   = $state(chartPrefs.connectNulls);
  bandOpacityPct = $state(chartPrefs.bandOpacityPct);
  animation      = $state<AnimationSpeed>(chartPrefs.animation);

  tooltipMode = $state<TooltipMode>(chartPrefs.tooltipMode);
  showTitle   = $state(chartPrefs.showTitle);
  xLabelMode  = $state<XLabelMode>(chartPrefs.xLabelMode);

  timeWindow = $state<TimeWindow>(chartPrefs.timeWindow);

  loaded = $state(true); // always already-hydrated

  setX(v: boolean)               { this.showXSlider     = v; }
  setY(v: boolean)               { this.showYSlider     = v; }
  setScrollZoom(v: boolean)      { this.scrollZoom      = v; }
  setReferenceSource(v: ReferenceSource) { this.referenceSource = v; }

  setShowValues(v: boolean)      { this.showValues   = v; }
  setSmooth(v: boolean)          { this.smooth       = v; }
  setScale(v: Scale)             { this.scale        = v; }
  setShowBands(v: boolean)       { this.showBands    = v; }
  setUseNicknames(v: boolean)    { this.useNicknames = v; }
  setShowSymbols(v: boolean)     { this.showSymbols  = v; }

  setLineWidth(v: number)        { this.lineWidth   = v; }
  setSymbolSize(v: number)       { this.symbolSize  = v; }
  setGridDensity(v: GridDensity) { this.gridDensity = v; }
  setDateFormat(v: DateFormat)   { this.dateFormat  = v; }

  setXAxisPadPct(v: number)      { this.xAxisPadPct    = v; }
  setYAxisPadPct(v: number)      { this.yAxisPadPct    = v; }
  setYVisiblePadPct(v: number)   { this.yVisiblePadPct = v; }

  setShowTodayLine(v: boolean)      { this.showTodayLine     = v; }
  setShowTrendLine(v: boolean)      { this.showTrendLine     = v; }
  setShowMeanLine(v: boolean)       { this.showMeanLine      = v; }
  setShowMinMaxMarkers(v: boolean)  { this.showMinMaxMarkers = v; }

  setSeriesType(v: SeriesType)      { this.seriesType     = v; }
  setStep(v: StepKind)              { this.step           = v; }
  setColorByFlag(v: boolean)        { this.colorByFlag    = v; }
  setConnectNulls(v: boolean)       { this.connectNulls   = v; }
  setBandOpacityPct(v: number)      { this.bandOpacityPct = v; }
  setAnimation(v: AnimationSpeed)   { this.animation      = v; }
  setTooltipMode(v: TooltipMode)    { this.tooltipMode    = v; }
  setShowTitle(v: boolean)          { this.showTitle      = v; }
  setXLabelMode(v: XLabelMode)      { this.xLabelMode     = v; }
  setTimeWindow(v: TimeWindow)      { this.timeWindow     = v; }

  /** Re-snapshot every field from the current global, discarding any
   *  divergence the user introduced on this isolated instance. */
  syncFromGlobal() {
    this.showXSlider = chartPrefs.showXSlider;
    this.showYSlider = chartPrefs.showYSlider;
    this.scrollZoom = chartPrefs.scrollZoom;
    this.referenceSource = chartPrefs.referenceSource;
    this.showValues = chartPrefs.showValues;
    this.smooth = chartPrefs.smooth;
    this.scale = chartPrefs.scale;
    this.showBands = chartPrefs.showBands;
    this.useNicknames = chartPrefs.useNicknames;
    this.showSymbols = chartPrefs.showSymbols;
    this.lineWidth = chartPrefs.lineWidth;
    this.symbolSize = chartPrefs.symbolSize;
    this.gridDensity = chartPrefs.gridDensity;
    this.dateFormat = chartPrefs.dateFormat;
    this.xAxisPadPct = chartPrefs.xAxisPadPct;
    this.yAxisPadPct = chartPrefs.yAxisPadPct;
    this.yVisiblePadPct = chartPrefs.yVisiblePadPct;
    this.showTodayLine = chartPrefs.showTodayLine;
    this.showTrendLine = chartPrefs.showTrendLine;
    this.showMeanLine = chartPrefs.showMeanLine;
    this.showMinMaxMarkers = chartPrefs.showMinMaxMarkers;
    this.seriesType = chartPrefs.seriesType;
    this.step = chartPrefs.step;
    this.colorByFlag = chartPrefs.colorByFlag;
    this.connectNulls = chartPrefs.connectNulls;
    this.bandOpacityPct = chartPrefs.bandOpacityPct;
    this.animation = chartPrefs.animation;
    this.tooltipMode = chartPrefs.tooltipMode;
    this.showTitle = chartPrefs.showTitle;
    this.xLabelMode = chartPrefs.xLabelMode;
    this.timeWindow = chartPrefs.timeWindow;
  }

  async load() { /* no-op — local-only */ }
}

/** Structural shape that any consumer of chartPrefs must satisfy. */
export type ChartPrefsLike = ChartPrefs | LocalChartPrefs;
