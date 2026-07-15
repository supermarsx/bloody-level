<script lang="ts">
  // Compact "applicable reference range" card for an analyte. Computes the
  // range the way every flag derivation does — sex-keyed default_ref ▸
  // categorical tiers ▸ generic default — and renders it as a short
  // pill row so the user sees at a glance what range governs their
  // current readings.

  import type { AnalyteInfo } from '$api/analyte-info';
  import { defaultRefFor, formatDefaultRef } from '$format/default-ref';
  import { parseTiers, formatTierRange } from '$format/tiers';

  let {
    info,
    /** 'm' | 'f' | 'x' | '?' — the patient's *global* sex (not per-PDF). */
    patientSex = '?',
    /** Optional unit string for the value column header. */
    unit = '',
    /** When `true` the card collapses to a single small tile that fits
     *  inside a row of equally-sized stat cards. Hides tier / phase
     *  chips and the descriptive footnote. */
    compact = false
  } = $props<{
    info: AnalyteInfo | null;
    patientSex?: string;
    unit?: string;
    compact?: boolean;
  }>();

  // The "primary" range — what flag derivation will use for a single value
  // when no categorical tier matches. We surface it as the headline pill
  // so the user immediately sees the threshold being applied.
  const primary = $derived.by(() => {
    if (!info) return null;
    const ref = defaultRefFor(info.default_ref_json, patientSex);
    if (!ref) return null;
    return {
      low: ref.low,
      high: ref.high,
      source: ref.source,
      sourceLabel:
        ref.source === 'm' ? 'male'
        : ref.source === 'f' ? 'female'
        : 'all'
    };
  });

  const tiers = $derived(info?.categorical_tiers_json ? parseTiers(info.categorical_tiers_json) : []);

  const cyclePhases = $derived.by(() => {
    if (!info?.cycle_phases_json) return [];
    try {
      const parsed = JSON.parse(info.cycle_phases_json) as Record<string, [number | null, number | null]>;
      return Object.entries(parsed).map(([phase, range]) => ({
        phase,
        low: range[0] ?? null,
        high: range[1] ?? null
      }));
    } catch { return []; }
  });

  function fmtRange(low: number | null, high: number | null): string {
    if (low != null && high != null) return `${low}–${high}`;
    if (high != null)               return `< ${high}`;
    if (low != null)                return `≥ ${low}`;
    return '—';
  }

  // Show the card only when we actually have something to display.
  const hasContent = $derived(
    !!primary || tiers.length > 0 || cyclePhases.length > 0
  );
</script>

{#if info && hasContent && compact}
  <!-- Compact mode: drops in next to the snapshot stat tiles, matches
       their height/padding. Shows only the headline range — the full
       card with tier/phase chip rows lives elsewhere. -->
  <div class="card p-3 ref-card--compact">
    <div class="text-xs text-fg2 flex items-center justify-between gap-1">
      <span>Reference</span>
      {#if primary && primary.source !== 'all'}
        <span class="ref-card__source ref-card__source--{primary.source}">
          {primary.sourceLabel}
        </span>
      {/if}
    </div>
    {#if primary}
      <div class="text-2xl font-semibold tabular-nums">{fmtRange(primary.low, primary.high)}</div>
      <div class="text-[10px] text-fg3">
        {#if unit}{unit}{:else}&nbsp;{/if}
        {#if tiers.length > 0 && !primary} · {tiers.length} tiers{/if}
      </div>
    {:else if tiers.length > 0}
      <div class="text-sm font-semibold">{tiers.length} tier ranges</div>
      <div class="text-[10px] text-fg3">{tiers.map((t) => t.label).slice(0, 3).join(' · ')}</div>
    {:else if cyclePhases.length > 0}
      <div class="text-sm font-semibold">{cyclePhases.length} cycle phases</div>
      <div class="text-[10px] text-fg3">{cyclePhases.map((c) => c.phase).slice(0, 3).join(' · ')}</div>
    {/if}
  </div>

{:else if info && hasContent}
  <section class="ref-card">
    <header class="ref-card__head">
      <span class="ref-card__pill" aria-hidden="true">REF</span>
      <span class="ref-card__title">Applicable reference</span>
      {#if unit}
        <span class="ref-card__unit" title="Reported unit">{unit}</span>
      {/if}
    </header>

    <!-- Headline range — the one flag derivation will apply for a numeric
         reading on this analyte / patient. -->
    {#if primary}
      <div class="ref-card__primary">
        <span class="ref-card__range">{fmtRange(primary.low, primary.high)}</span>
        <span class="ref-card__source ref-card__source--{primary.source}">
          {primary.sourceLabel}
        </span>
        {#if unit}
          <span class="ref-card__unit-inline">{unit}</span>
        {/if}
      </div>
    {/if}

    <!-- Categorical tiers (Vit D, Ferritina, IgE, …) -->
    {#if tiers.length > 0}
      <div class="ref-card__group">
        <div class="ref-card__group-label">Tiers</div>
        <div class="ref-card__chips">
          {#each tiers as t}
            <span class="ref-chip" title={t.label}>
              <span class="ref-chip__name">{t.label}</span>
              <span class="ref-chip__range">{formatTierRange(t)}</span>
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Cycle-phase ranges (Estradiol, Progesterona, FSH/LH female) -->
    {#if cyclePhases.length > 0}
      <div class="ref-card__group">
        <div class="ref-card__group-label">Cycle phases</div>
        <div class="ref-card__chips">
          {#each cyclePhases as cp}
            <span class="ref-chip" title="{cp.phase}: {fmtRange(cp.low, cp.high)}">
              <span class="ref-chip__name">{cp.phase}</span>
              <span class="ref-chip__range">{fmtRange(cp.low, cp.high)}</span>
            </span>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Footnote: which combination of sex / age / cycle drove the headline. -->
    {#if info.sex_dependent || info.cycle_dependent || info.age_dependent}
      <p class="ref-card__note">
        {#if info.sex_dependent}This range varies by sex.{/if}
        {#if info.cycle_dependent} This range varies across the menstrual cycle.{/if}
        {#if info.age_dependent}   This range varies by age.{/if}
      </p>
    {/if}
  </section>
{/if}

<style>
  .ref-card--compact {
    /* Override the stat-tile padding only enough to keep the source pill
       comfortable next to the label. */
    display: flex;
    flex-direction: column;
    gap: 0.05rem;
    min-width: 0;
  }
  .ref-card {
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-left: 3px solid rgb(var(--accent));
    border-radius: 0.55rem;
    padding: 0.6rem 0.85rem 0.7rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }
  .ref-card__head {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }
  .ref-card__pill {
    display: inline-flex;
    padding: 0.05rem 0.4rem;
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-radius: 0.3rem;
  }
  .ref-card__title {
    font-size: 0.7rem;
    font-weight: 600;
    color: rgb(var(--fg-1));
  }
  .ref-card__unit {
    margin-left: auto;
    font-size: 0.7rem;
    color: rgb(var(--fg-3));
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .ref-card__primary {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .ref-card__range {
    font-size: 1.2rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: rgb(var(--fg-1));
  }
  .ref-card__source {
    display: inline-flex;
    padding: 0.05rem 0.4rem;
    border-radius: 9999px;
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    border: 1px solid;
  }
  .ref-card__source--m { color: rgb(var(--accent)); background: rgb(var(--accent) / 0.10); border-color: rgb(var(--accent) / 0.4); }
  .ref-card__source--f { color: rgb(var(--warn));   background: rgb(var(--warn)   / 0.10); border-color: rgb(var(--warn)   / 0.4); }
  .ref-card__source--all {
    color: rgb(var(--fg-2));
    background: rgb(var(--bg-3));
    border-color: rgb(var(--line));
  }
  .ref-card__unit-inline {
    color: rgb(var(--fg-3));
    font-size: 0.8rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }
  .ref-card__group { display: flex; flex-direction: column; gap: 0.25rem; }
  .ref-card__group-label {
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--fg-3));
  }
  .ref-card__chips {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }
  .ref-chip {
    display: inline-flex;
    align-items: baseline;
    gap: 0.35rem;
    padding: 0.15rem 0.5rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    font-size: 0.7rem;
  }
  .ref-chip__name { color: rgb(var(--fg-1)); font-weight: 500; }
  .ref-chip__range { color: rgb(var(--fg-3)); font-variant-numeric: tabular-nums; }
  .ref-card__note {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    font-style: italic;
  }
</style>
