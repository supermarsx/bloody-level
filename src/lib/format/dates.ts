import {
  format,
  parseISO,
  differenceInDays,
  differenceInYears,
} from "date-fns";
import { appearance } from "$theme/appearance.svelte";
import { t } from "$lib/i18n/index.svelte";

/**
 * Whole-year age between a YYYY-MM-DD date of birth and today. Honours
 * month/day boundary — someone born 1985-08-15 is 39 on 2025-04-30 and 40
 * on 2025-08-15. Returns null on missing or malformed input.
 */
/**
 * Human-readable relative span between two ISO YYYY-MM-DD dates. Returns
 * a compact label like "+14d", "+3w", "+2mo", "+1y 4mo", or "—" for the
 * first item in a sequence. Output convention:
 *   < 1d → "same day"
 *   1-13 days → "+Nd"
 *   14-55 days → "+Nw" (whole weeks)
 *   2-11 months → "+Nmo" (rounded)
 *   ≥ 1 year → "+Ny" or "+Ny Nmo" if the remainder isn't 0
 *
 * Negative values are bracketed so the caller knows the sequence is
 * reverse-chronological — e.g. "-3mo".
 */
export function formatRelativeSpan(
  fromIso: string | null | undefined,
  toIso: string | null | undefined,
): string {
  if (!fromIso || !toIso) return "—";
  let f: Date, toDate: Date;
  try {
    f = parseISO(fromIso);
    toDate = parseISO(toIso);
  } catch {
    return "—";
  }
  const days = differenceInDays(toDate, f);
  if (!Number.isFinite(days)) return "—";
  if (days === 0) return t("same day");
  const sign = days > 0 ? "+" : "−";
  const abs = Math.abs(days);
  if (abs < 14) return `${sign}${abs}${t("d")}`;
  if (abs < 56)
    return `${sign}${Math.round(abs / 7)}${
      appearance.resolvedLocale === "pt-PT" ? t("w") : "w"
    }`;
  const months = Math.round(abs / 30.4375);
  if (months < 12)
    return `${sign}${months}${
      appearance.resolvedLocale === "pt-PT" ? t("mo") : "mo"
    }`;
  const years = Math.floor(months / 12);
  const remMo = months % 12;
  const year = appearance.resolvedLocale === "pt-PT" ? t("y") : "y";
  const month = appearance.resolvedLocale === "pt-PT" ? t("mo") : "mo";
  return remMo === 0
    ? `${sign}${years}${year}`
    : `${sign}${years}${year} ${remMo}${month}`;
}

export function ageFromDob(iso: string | null | undefined): number | null {
  if (!iso) return null;
  try {
    const dob = parseISO(iso);
    const yrs = differenceInYears(new Date(), dob);
    return Number.isFinite(yrs) && yrs >= 0 && yrs < 200 ? yrs : null;
  } catch {
    return null;
  }
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return format(parseISO(iso), "yyyy-MM-dd");
  } catch {
    return iso;
  }
}

export function formatDateLong(iso: string | null | undefined): string {
  if (!iso) return "—";
  try {
    return new Intl.DateTimeFormat(appearance.resolvedLocale, {
      day: "numeric",
      month: "short",
      year: "numeric",
    }).format(parseISO(iso));
  } catch {
    return iso;
  }
}

export function daysBetween(a: string, b: string): number {
  return differenceInDays(parseISO(a), parseISO(b));
}
