<script lang="ts">
  // HRT-anchor card that shows on a patient's page. Set/edit/clear the
  // start date; once set, surfaces a live "X days · Y months since HRT
  // start" counter, and renders a small timeline of the patient's reports
  // grouped by milestone bucket (Baseline / Week 1-8 / Month / Year).

  import { differenceInDays, parseISO } from 'date-fns';
  import * as admin from '$api/records-admin';
  import { toasts } from '../toasts/store.svelte';
  import { formatDate } from '$format/dates';
  import { hrtMilestoneFor } from '$format/hrt-milestone';
  import type { ReportSummary } from '$api/reports';

  let {
    patientId,
    hrtStartIso,
    reports = [] as ReportSummary[],
    onChanged,
    /** When `true`, hides the inline date editor and clear button. The
     *  consumer (patient detail page) drives the value through the
     *  metadata form so this section stays purely a display surface. */
    readOnly = false
  } = $props<{
    patientId: string;
    hrtStartIso: string | null;
    reports?: ReportSummary[];
    onChanged?: (next: string | null) => void;
    readOnly?: boolean;
  }>();

  let editing = $state(false);
  let draft = $state('');
  let busy = $state(false);

  function startEdit() {
    if (readOnly) return;
    draft = hrtStartIso ?? '';
    editing = true;
  }
  async function save() {
    busy = true;
    try {
      const next = draft.trim() || null;
      await admin.setPatientHrtStart(patientId, next);
      toasts.success('HRT start updated', next ?? 'cleared');
      editing = false;
      onChanged?.(next);
    } catch (e) { toasts.error(e); }
    finally { busy = false; }
  }
  async function clear() {
    busy = true;
    try {
      await admin.setPatientHrtStart(patientId, null);
      toasts.success('HRT start cleared');
      editing = false;
      onChanged?.(null);
    } catch (e) { toasts.error(e); }
    finally { busy = false; }
  }

  // Live "today − start" counter for the header. We surface days, months,
  // and years so users can read the duration at whichever scale they need
  // (early HRT readers care about days, long-term users about years).
  const sinceStart = $derived.by(() => {
    if (!hrtStartIso) return null;
    try {
      const days = differenceInDays(new Date(), parseISO(hrtStartIso));
      const months = Math.round(days / 30.4375);
      const years = Math.round((days / 365.25) * 10) / 10;
      return { days, months, years };
    } catch { return null; }
  });

  // Bucket reports by milestone label so the timeline shows them grouped.
  const milestoneBuckets = $derived.by(() => {
    if (!hrtStartIso) return [];
    const bucketed = reports
      .map((r: ReportSummary) => ({ r, m: hrtMilestoneFor(r.collection_date_iso, hrtStartIso) }))
      .filter((x: { m: ReturnType<typeof hrtMilestoneFor> }) => x.m !== null)
      .sort((a: { m: { days: number } }, b: { m: { days: number } }) => a.m.days - b.m.days);
    return bucketed;
  });
</script>

<section class="hrt-card">
  <header class="hrt-card__header">
    <span class="hrt-card__title">
      <span class="hrt-card__pill" aria-hidden="true">HRT</span>
      Hormone-replacement timeline
    </span>
    {#if !readOnly && hrtStartIso && !editing}
      <button class="text-xs text-accent hover:underline" onclick={startEdit}>Edit</button>
    {:else if !readOnly && !hrtStartIso && !editing}
      <button class="text-xs text-accent hover:underline" onclick={startEdit}>Set start date</button>
    {:else if readOnly}
      <span class="text-[11px] text-fg3 italic">Edit in patient form</span>
    {/if}
  </header>

  {#if editing}
    <div class="hrt-card__edit">
      <label class="edit-field">
        <span class="edit-field__label">HRT start date</span>
        <input type="date" class="input max-w-[11rem]"
               bind:value={draft}
               onkeydown={(e) => { if (e.key === 'Enter') save(); else if (e.key === 'Escape') editing = false; }} />
        <span class="edit-field__hint">
          Used as the anchor for every report's "Day N / Month N HRT" milestone.
          Leave blank and Save (or click Clear) to remove the anchor.
        </span>
      </label>
      <div class="flex gap-2 justify-end">
        {#if hrtStartIso}
          <button class="btn text-crit hover:bg-crit/10" disabled={busy} onclick={clear}>Clear</button>
        {/if}
        <button class="btn" disabled={busy} onclick={() => (editing = false)}>Cancel</button>
        <button class="btn-accent" disabled={busy} onclick={save}>
          {busy ? 'Saving…' : 'Save'}
        </button>
      </div>
    </div>
  {:else if hrtStartIso}
    <div class="hrt-card__since">
      <div class="hrt-card__since-row">
        <span class="hrt-card__since-label">Started</span>
        <span class="hrt-card__since-value">{formatDate(hrtStartIso)}</span>
      </div>
      {#if sinceStart}
        <div class="hrt-card__since-row">
          <span class="hrt-card__since-label">Today</span>
          <span class="hrt-card__since-value">
            <strong>{sinceStart.days}</strong> days
            <span class="text-fg3">·</span>
            <strong>{sinceStart.months}</strong> months
            {#if sinceStart.years > 0}
              <span class="text-fg3">·</span>
              <strong>{sinceStart.years}</strong> years
            {/if}
          </span>
        </div>
      {/if}
    </div>

    {#if milestoneBuckets.length > 0}
      <div class="hrt-card__timeline">
        <div class="hrt-card__timeline-label">Reports by milestone</div>
        <div class="hrt-card__rail">
          {#each milestoneBuckets as { r, m } (r.id)}
            <a href={`/report/${r.id}`}
               class="hrt-card__chip"
               class:hrt-card__chip--pre={m!.isPre}
               title="{r.nickname ?? formatDate(r.collection_date_iso)} — {m!.long}">
              <span class="hrt-card__chip-label">{m!.label}</span>
              <span class="hrt-card__chip-date">{formatDate(r.collection_date_iso)}</span>
              {#if r.nickname}
                <span class="hrt-card__chip-nick">{r.nickname}</span>
              {/if}
            </a>
          {/each}
        </div>
      </div>
    {:else}
      <p class="text-xs text-fg3 italic">No reports yet to plot on the timeline.</p>
    {/if}
  {:else}
    <p class="text-xs text-fg2">
      Add an HRT start date to track how many days / months each report sits from baseline,
      and to see all reports laid out on a milestone timeline (Baseline · Week 1–8 · Month N · Year N).
    </p>
  {/if}
</section>

<style>
  .hrt-card {
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-left: 3px solid rgb(var(--accent));
    border-radius: 0.6rem;
    padding: 0.85rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }
  .hrt-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
  }
  .hrt-card__title {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    font-weight: 600;
    color: rgb(var(--fg-1));
  }
  .hrt-card__pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.1rem 0.45rem;
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-size: 0.65rem;
    font-weight: 700;
    border-radius: 0.3rem;
    letter-spacing: 0.04em;
  }
  .hrt-card__edit {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding-top: 0.4rem;
    border-top: 1px dashed rgb(var(--line));
  }
  .hrt-card__since {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
    gap: 0.4rem 1rem;
    padding: 0.5rem 0.7rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
  }
  .hrt-card__since-row {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }
  .hrt-card__since-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--fg-3));
    min-width: 4rem;
  }
  .hrt-card__since-value {
    font-size: 0.85rem;
    color: rgb(var(--fg-1));
    font-variant-numeric: tabular-nums;
  }
  .hrt-card__since-value strong {
    font-weight: 600;
    color: rgb(var(--accent));
  }
  .hrt-card__timeline {
    padding-top: 0.3rem;
  }
  .hrt-card__timeline-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--fg-3));
    margin-bottom: 0.4rem;
  }
  .hrt-card__rail {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .hrt-card__chip {
    display: inline-flex;
    flex-direction: column;
    gap: 0.05rem;
    padding: 0.3rem 0.55rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    font-size: 0.72rem;
    color: rgb(var(--fg-2));
    text-decoration: none;
    transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
  }
  .hrt-card__chip:hover {
    background: rgb(var(--accent) / 0.08);
    border-color: rgb(var(--accent) / 0.5);
    color: rgb(var(--fg-1));
  }
  .hrt-card__chip-label {
    font-weight: 600;
    color: rgb(var(--accent));
  }
  .hrt-card__chip--pre .hrt-card__chip-label {
    color: rgb(var(--warn));
  }
  .hrt-card__chip-date {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    font-variant-numeric: tabular-nums;
  }
  .hrt-card__chip-nick {
    font-size: 0.65rem;
    color: rgb(var(--fg-2));
    font-style: italic;
  }
</style>
