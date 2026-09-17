import * as echarts from "echarts";

function cssVar(name: string): string {
  if (typeof document === "undefined") return "";
  return getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
}

function rgb(name: string, alpha = 1): string {
  const triplet = cssVar(name);
  if (!triplet) return "#000";
  return alpha === 1 ? `rgb(${triplet})` : `rgba(${triplet} / ${alpha})`;
}

/**
 * Read the actual currently-applied theme from the document. The `mode`
 * argument passed by callers is only used for naming the registered theme
 * ('bloody-level-light' / 'bloody-level-dark'); the actual styling always reflects
 * whatever data-theme is live, because that's the only set of CSS vars
 * we can read.
 */
function activeMode(): "light" | "dark" {
  if (typeof document === "undefined") return "dark";
  return document.documentElement.getAttribute("data-theme") === "light"
    ? "light"
    : "dark";
}

function buildTheme(_registeredAs: "light" | "dark") {
  // `_registeredAs` is symbolic — both `bloody-level-light` and `bloody-level-dark` get
  // the same body, reflecting the current document mode. We re-register on
  // every theme-change event so this stays correct.
  const mode = activeMode();
  const palette = [
    cssVar("--chart-1"),
    cssVar("--chart-2"),
    cssVar("--chart-3"),
    cssVar("--chart-4"),
    cssVar("--chart-5"),
    cssVar("--chart-6"),
    cssVar("--chart-7"),
    cssVar("--chart-8"),
  ].filter(Boolean);

  const grid = cssVar("--chart-grid");
  const axis = cssVar("--chart-axis");
  const fg1 = rgb("--fg-1");
  const fg2 = rgb("--fg-2");
  // `--bg-2` is too close to `--bg-1` in light mode (246 vs 255), making
  // tooltips fade into the page. We explicitly compose a higher-contrast
  // tooltip surface that works in both themes:
  //   - In light: near-white with a strong shadow + border so it lifts
  //     off the page.
  //   - In dark: keep the existing dark grey card.
  const bg1 = rgb("--bg-1");
  const tooltipBg = mode === "light" ? bg1 : rgb("--bg-2");
  const line = rgb("--line");

  return {
    color: palette,
    backgroundColor: "transparent",
    textStyle: { color: fg1, fontFamily: "Inter, system-ui, sans-serif" },
    title: {
      textStyle: { color: fg1, fontWeight: 600 },
      subtextStyle: { color: fg2 },
    },
    legend: {
      textStyle: { color: fg1 },
      icon: "circle",
    },
    tooltip: {
      backgroundColor: tooltipBg,
      borderColor: line,
      borderWidth: 1,
      textStyle: { color: fg1, fontSize: 12 },
      extraCssText:
        mode === "light"
          ? "box-shadow: 0 6px 20px rgba(0,0,0,0.14); border-radius: 8px;"
          : "box-shadow: 0 4px 16px rgba(0,0,0,0.45); border-radius: 8px;",
    },
    grid: {
      borderColor: line,
      left: 48,
      right: 24,
      top: 32,
      bottom: 40,
      containLabel: true,
    },
    xAxis: {
      axisLine: { lineStyle: { color: axis } },
      axisTick: { lineStyle: { color: axis } },
      // Axis labels use fg-1 (full-contrast) instead of fg-2 (medium grey).
      // In light mode the medium grey on near-white was reading as almost
      // invisible — fg-1 keeps tick text legible at small sizes.
      axisLabel: { color: fg1, fontWeight: 500 },
      splitLine: { lineStyle: { color: grid } },
    },
    yAxis: {
      axisLine: { lineStyle: { color: axis } },
      axisTick: { lineStyle: { color: axis } },
      axisLabel: { color: fg1, fontWeight: 500 },
      splitLine: { lineStyle: { color: grid } },
    },
    line: {
      smooth: false,
      symbol: "circle",
      symbolSize: 6,
      lineStyle: { width: 2 },
    },
    valueAxis: {
      splitLine: { lineStyle: { color: grid } },
    },
    timeline: { lineStyle: { color: line } },
  };
}

let registered = false;

export function ensureThemesRegistered() {
  if (registered) return;
  echarts.registerTheme("bloody-level-light", buildTheme("light"));
  echarts.registerTheme("bloody-level-dark", buildTheme("dark"));
  registered = true;
}

export function reRegisterThemes() {
  // CSS vars may have changed (theme switch); re-register so future inits pick up new colors.
  echarts.registerTheme("bloody-level-light", buildTheme("light"));
  echarts.registerTheme("bloody-level-dark", buildTheme("dark"));
}

export function activeThemeName(): string {
  if (typeof document === "undefined") return "bloody-level-dark";
  const t = document.documentElement.getAttribute("data-theme");
  return t === "light" ? "bloody-level-light" : "bloody-level-dark";
}

export const refBandColors = {
  normal: () => rgb("--ok", 0.1),
  borderline: () => rgb("--warn", 0.1),
  critical: () => rgb("--crit", 0.12),
};
