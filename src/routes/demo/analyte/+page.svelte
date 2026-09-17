<script lang="ts">
  import TimeSeries from '$charts/time-series.svelte';
  import FlagPill from '$charts/flag-pill.svelte';
  import DeltaBadge from '$charts/delta-badge.svelte';
  import { openUrl } from '$api/shell';
  import { toasts } from '../../../lib/toasts/store.svelte';
  import { formatDate, formatDateLong, formatRelativeSpan } from '$format/dates';
  import { formatNumber } from '$format/numbers';

  // This is deliberately explicit and deterministic: the screenshot and the
  // public demo route always show the same fictional patient and values.
  const demoReadings = [
    { date: '2026-09-17', value: 1.8, flag: 'normal' },
    { date: '2026-06-17', value: 2.1, flag: 'normal' },
    { date: '2026-03-17', value: 2.5, flag: 'normal' },
    { date: '2025-12-17', value: 1.9, flag: 'normal' },
    { date: '2025-09-17', value: 2.3, flag: 'normal' },
  ];

  const values = demoReadings.map((reading) => reading.value);
  const latest = demoReadings[0];
  const first = demoReadings[demoReadings.length - 1];
  const min = Math.min(...values);
  const max = Math.max(...values);
  const mean = values.reduce((sum, value) => sum + value, 0) / values.length;

  async function openLoinc() {
    try {
      await openUrl('https://loinc.org/3016-3/');
    } catch (e) {
      toasts.error(e);
    }
  }
</script>

<svelte:head>
  <title>TSH · Alex Silva · Demo | blevel-tracker</title>
</svelte:head>

<div class="mx-auto max-w-screen-2xl space-y-4">
  <section class="card border-accent/40 bg-bg2 p-4 sm:p-5">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2 text-xs text-fg3">
          <span class="pill-muted font-semibold tracking-wide">DEMO DATA</span>
          <span>Analyte review</span>
          <span>·</span>
          <span>Five readings · 12 months</span>
        </div>
        <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <h1 class="text-2xl font-semibold tracking-tight">Thyroid stimulating hormone</h1>
          <span class="font-mono text-sm text-fg3">TSH</span>
        </div>
        <p class="text-sm text-fg2">
          A longitudinal view for <span class="font-medium text-fg1">Alex Silva</span>, a fictional demo patient.
          Every value below is synthetic and exists only to show the review workflow.
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <div class="select flex items-center gap-2 px-3 py-2 text-sm" aria-label="Demo patient">
          <span class="text-fg3">Patient</span>
          <span class="font-medium">Alex Silva</span>
          <span class="text-fg3">⌄</span>
        </div>
        <button type="button" class="btn text-xs" onclick={openLoinc} title="Open LOINC 3016-3 in the default browser">
          LOINC 3016-3 ↗
        </button>
      </div>
    </div>
  </section>

  <section class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">Latest</div>
      <div class="mt-1 text-2xl font-semibold tabular-nums">{formatNumber(latest.value)}</div>
      <div class="text-[10px] text-fg3">{formatDate(latest.date)}</div>
    </div>
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">Previous</div>
      <div class="mt-1 text-2xl font-semibold tabular-nums">{formatNumber(demoReadings[1].value)}</div>
      <div class="text-[10px] text-fg3">{formatRelativeSpan(demoReadings[1].date, latest.date)}</div>
    </div>
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">First</div>
      <div class="mt-1 text-xl font-semibold tabular-nums">{formatNumber(first.value)}</div>
      <div class="text-[10px] text-fg3">{formatDate(first.date)}</div>
    </div>
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">Range</div>
      <div class="mt-1 text-xl font-semibold tabular-nums">0.27–4.20</div>
      <div class="text-[10px] text-fg3">mIU/L · reference</div>
    </div>
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">Mean</div>
      <div class="mt-1 text-xl font-semibold tabular-nums">{formatNumber(mean)}</div>
      <div class="text-[10px] text-fg3">{demoReadings.length} readings</div>
    </div>
    <div class="card p-3 sm:p-4">
      <div class="text-xs text-fg2">Latest flag</div>
      <div class="mt-2"><FlagPill flag={latest.flag} /></div>
      <div class="mt-1 text-[10px] text-fg3">within reference</div>
    </div>
  </section>

  <section class="card p-3 sm:p-4">
    <div class="mb-2 flex flex-wrap items-baseline justify-between gap-2">
      <div>
        <h2 class="text-sm font-semibold">TSH over time</h2>
        <p class="text-xs text-fg3">Quarterly readings from {formatDateLong(first.date)} to {formatDateLong(latest.date)}</p>
      </div>
      <span class="text-xs text-fg2">Unit: <span class="font-mono">mIU/L</span></span>
    </div>
    <TimeSeries
      points={demoReadings}
      refBands={[{ low: 0.27, high: 4.2, tier: 'normal' }]}
      unit="mIU/L"
      height={330}
      exportName="demo-tsh-alex-silva"
    />
  </section>

  <section class="card overflow-hidden">
    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4 py-3">
      <div>
        <h2 class="text-sm font-semibold">Readings</h2>
        <p class="text-xs text-fg3">Most recent first · synthetic source records</p>
      </div>
      <div class="flex items-center gap-3 text-xs text-fg2">
        <span>Latest change</span>
        <DeltaBadge current={latest.value} previous={demoReadings[1].value} />
      </div>
    </div>
    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead class="bg-bg1 text-left text-xs uppercase tracking-wide text-fg2">
          <tr class="border-b border-line">
            <th class="px-4 py-2.5">Date</th>
            <th class="px-4 py-2.5">Patient</th>
            <th class="px-4 py-2.5 text-right">Value</th>
            <th class="px-4 py-2.5">Unit</th>
            <th class="px-4 py-2.5">Flag</th>
            <th class="px-4 py-2.5">Spacing</th>
          </tr>
        </thead>
        <tbody>
          {#each demoReadings as reading, index}
            {@const older = index + 1 < demoReadings.length ? demoReadings[index + 1] : null}
            <tr class="border-b border-line/50 transition-colors hover:bg-bg3/50">
              <td class="px-4 py-2.5 tabular-nums">{formatDate(reading.date)}</td>
              <td class="px-4 py-2.5">Alex Silva</td>
              <td class="px-4 py-2.5 text-right font-medium tabular-nums">{formatNumber(reading.value)}</td>
              <td class="px-4 py-2.5 font-mono text-xs text-fg2">mIU/L</td>
              <td class="px-4 py-2.5"><FlagPill flag={reading.flag} /></td>
              <td class="px-4 py-2.5 text-xs text-fg2">{older ? formatRelativeSpan(older.date, reading.date) : '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <p class="pb-2 text-center text-xs text-fg3">
    This public preview contains no real patient information. Use the application locally to import and review your own reports.
  </p>
</div>
