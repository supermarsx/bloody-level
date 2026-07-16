// Compute the time elapsed between an HRT (or any other reference) start
// date and a report's collection date, expressed as an at-a-glance
// "milestone" string. Day count for early days, weeks 1–8, months after,
// years past 12 months. Negative values are flagged as `pre` so users
// labelling baseline labs see "Day −7 (baseline)".

import { differenceInDays, parseISO } from "date-fns";

export interface HrtMilestone {
  /** Whole-day distance from start (negative if collection precedes start). */
  days: number;
  /** Whole-month approximation (`days / 30.4375`, rounded). */
  months: number;
  /** Whole-year approximation (`days / 365.25`, with one decimal kept). */
  years: number;
  /** Whether the collection happened before the start date. */
  isPre: boolean;
  /** Compact UI label, e.g. "Day 14", "Month 6", "Year 2", "Day −7 (baseline)". */
  label: string;
  /** Long-form sub-label for tooltips, e.g. "182 days · 6 mo · 0.5 y since HRT start". */
  long: string;
}

export function hrtMilestoneFor(
  collectionDateIso: string | null | undefined,
  hrtStartIso: string | null | undefined,
): HrtMilestone | null {
  if (!collectionDateIso || !hrtStartIso) return null;
  let start: Date, collect: Date;
  try {
    start = parseISO(hrtStartIso);
    collect = parseISO(collectionDateIso);
  } catch {
    return null;
  }
  const days = differenceInDays(collect, start);
  if (!Number.isFinite(days)) return null;
  const months = Math.round(days / 30.4375);
  // One-decimal year resolution — enough to show 0.5y / 1.5y / 2y on
  // the timeline without false precision. Negative years preserved for
  // pre-start (baseline) draws.
  const years = Math.round((days / 365.25) * 10) / 10;
  const isPre = days < 0;
  const absDays = Math.abs(days);
  const absMonths = Math.abs(months);
  const absYears = Math.abs(years);

  let label: string;
  if (isPre) {
    label = `Day −${absDays} (baseline)`;
  } else if (days <= 7) {
    label = `Day ${days}`;
  } else if (days <= 56) {
    const weeks = Math.round(days / 7);
    label = `Week ${weeks}`;
  } else if (months < 12) {
    label = `Month ${months}`;
  } else {
    const wholeYears = Math.floor(months / 12);
    const remainder = months % 12;
    label =
      remainder === 0 ? `Year ${wholeYears}` : `${wholeYears}y ${remainder}m`;
  }

  // Long label always shows all three units so the tooltip / sub-label
  // tells the same story regardless of which scale the user is reading.
  const yearText = absYears > 0 ? ` · ${absYears} y` : "";
  const long = isPre
    ? `${absDays} day${absDays === 1 ? "" : "s"} before HRT start (${absMonths} mo${yearText})`
    : `${days} day${days === 1 ? "" : "s"} · ${absMonths} mo${yearText} since HRT start`;
  return { days, months, years, isPre, label, long };
}
