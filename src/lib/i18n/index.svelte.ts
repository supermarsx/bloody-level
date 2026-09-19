import { appearance } from "$theme/appearance.svelte";
import { sharedPtPT } from "./shared";

export type TranslationValue = string | number | boolean | null | undefined;
export type TranslationParams = Record<string, TranslationValue>;

/**
 * Translate a user-facing English source string. English is deliberately the
 * source language so missing entries remain safe and readable while the
 * Portuguese catalogue is being expanded.
 */
export function t(source: string, params: TranslationParams = {}): string {
  const translated =
    appearance.resolvedLocale === "pt-PT"
      ? (sharedPtPT[source] ?? source)
      : source;
  return translated.replace(/\{([a-zA-Z0-9_]+)\}/g, (whole, name: string) => {
    const value = params[name];
    return value === null || value === undefined ? whole : String(value);
  });
}

export function countText(
  count: number,
  singular: string,
  plural: string,
): string {
  return t(count === 1 ? singular : plural, { count });
}

export function translateErrorTitle(source: string): string {
  return t(source);
}
