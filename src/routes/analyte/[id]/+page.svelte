<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { page } from '$app/stores';
  import TimeSeries from '$charts/time-series.svelte';
  import FlagPill from '$charts/flag-pill.svelte';
  import DeltaBadge from '$charts/delta-badge.svelte';
  import { analyteTimeseries, type AnalyteReading } from '$api/reports';
  import * as analyteApi from '$api/analyte-info';
  import * as admin from '$api/records-admin';
  import { AppError } from '$api/errors';
  import { toasts } from '../../../lib/toasts/store.svelte';
  import { formatDate, formatRelativeSpan } from '$format/dates';
  import { formatNumber } from '$format/numbers';
  import { prettyUnit } from '$format/units';
  import { parseTiers, matchTier, tierToFlag, formatTierRange } from '$format/tiers';
  import { defaultRefFor, flagForDefaultRef } from '$format/default-ref';
  import { exportAnalyteTimeseriesCsv } from '$api/export';
  import { openUrl } from '$api/shell';
  import { saveTextFile } from '$format/save';
  import { setPageTitle } from '$lib/title.svelte';
  import BackButton from '$components/back-button.svelte';
  import { listOntologyEntries, type AnalyteOntologyEntry } from '$api/analyte-info';
  import ReferenceCard from '$components/reference-card.svelte';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { chartPrefs } from '$charts/prefs.svelte';

  let analyteId = $derived($page.params.id ?? '');
  let info = $state<analyteApi.AnalyteInfo | null>(null);
  let allReadings = $state<AnalyteReading[]>([]);
  let err = $state<string | null>(null);
  let showPriors = $state(false);
  // Sibling analytes in the same `panel` — only loaded when the current
  // entry turns out to be a panel-header placeholder, so we can point the
  // user to the actual measurable members of the group.
  let siblingAnalytes = $state<AnalyteOntologyEntry[]>([]);
  // The `?patient=<id>` query param overrides the auto-pick. Empty string
  // means "auto" (default to most-recent patient with readings).
  let selectedPatient = $state<string | null>(null);

  // Patients who have at least one reading for this analyte, with their most
  // recent reading date — used both for the picker and the auto-default.
  const patientsWithReadings = $derived.by(() => {
    const map = new Map<string, { id: string; name: string; sex: string; latestDate: string; count: number }>();
    for (const r of allReadings) {
      const cur = map.get(r.patient_id);
      if (!cur) {
        map.set(r.patient_id, { id: r.patient_id, name: r.patient_name, sex: r.patient_sex, latestDate: r.date, count: 1 });
      } else {
        cur.count++;
        if (r.date > cur.latestDate) cur.latestDate = r.date;
      }
    }
    return [...map.values()].sort((a, b) => b.latestDate.localeCompare(a.latestDate));
  });

  // Auto-pick the most-recent-reading patient when nothing is selected yet.
  // We never silently mix patients in the chart or table; if a user explicitly
  // wants the cross-patient view they can choose "All patients".
  const effectivePatient = $derived(
    selectedPatient && patientsWithReadings.some((p) => p.id === selectedPatient)
      ? selectedPatient
      : patientsWithReadings[0]?.id ?? null
  );

  let patientSearch = $state('');
  let patientMenuOpen = $state(false);
  let patientActiveIndex = $state(-1);
  let patientPickerElement = $state<HTMLElement | null>(null);
  let patientSearchElement = $state<HTMLInputElement | null>(null);

  const activePatient = $derived(
    patientsWithReadings.find((p) => p.id === effectivePatient) ?? null
  );

  const filteredPatients = $derived.by(() => {
    const query = patientSearch.trim().toLowerCase();
    if (!query) return patientsWithReadings;
    return patientsWithReadings.filter((p) =>
      p.name.toLowerCase().includes(query) || p.id.toLowerCase().includes(query)
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
    selectedPatient = id;
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

  onMount(() => {
    selectedPatient = $page.url.searchParams.get('patient');
  });

  // Strictly single-patient. The chart and stats below combine values into a
  // single line/series, so silently mixing patients would conflate distinct
  // people's trends. The picker above lets the user switch.
  const readings = $derived(
    allReadings
      .filter((r) => r.patient_id === effectivePatient)
      .sort((a, b) => a.date.localeCompare(b.date))
  );

  // The table is intentionally newest-first for review. Chart/statistics
  // consumers use the chronological copy so the line and first/latest values
  // still read left-to-right in time.
  const visibleReadings = $derived(
    [...(showPriors ? readings : readings.filter((r) => !r.inline_prior))].reverse()
  );
  const chronologicalVisibleReadings = $derived([...visibleReadings].reverse());

  async function refresh() {
    err = null;
    try {
      const [i, r] = await Promise.all([
        analyteApi.get(analyteId).catch((e) => {
          // Don't swallow silently — surface why we're missing the info card.
          const ae = AppError.fromUnknown(e);
          if (ae.kind !== 'not_found') {
            toasts.warn('Couldn’t load analyte info', ae.message);
          }
          return null;
        }),
        analyteTimeseries(analyteId)
      ]);
      info = i;
      allReadings = r;
      // If this entry turns out to be a panel header, load the rest of the
      // ontology in the background so we can list its measurable siblings.
      if (i?.is_panel_header && i.panel) {
        try {
          const all = await listOntologyEntries();
          siblingAnalytes = all.filter(
            (e) => e.panel === i.panel && e.id !== i.id && !e.is_panel_header
          );
        } catch { /* non-critical — page still renders without the sibling list */ }
      } else {
        siblingAnalytes = [];
      }
    } catch (e) {
      err = String(e);
      toasts.error(e);
    }
  }

  async function onExportCsv() {
    try {
      // Honour the active patient filter so the CSV matches what's on screen.
      const pid = effectivePatient && effectivePatient !== '__all__' ? effectivePatient : undefined;
      const out = await exportAnalyteTimeseriesCsv(analyteId, pid);
      const path = await saveTextFile(out.content, { defaultPath: out.filename });
      if (path) toasts.success('Exported', path);
    } catch (e) { toasts.error(e); }
  }

  let reloadingOntology = $state(false);
  async function reloadOntology() {
    const ok = await ask(
      'Reload analyte ontology from the bundled seed?\n\n' +
      '• Re-installs every seed-bundled analyte\'s descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n' +
      '• User-created analytes and user-added aliases are preserved.\n' +
      '• Existing parsed results are not touched.',
      { title: 'Reload ontology', kind: 'warning' }
    );
    if (!ok) return;
    reloadingOntology = true;
    try {
      const r = await admin.reloadOntology();
      toasts.success('Ontology reloaded', `${r.analytes_installed} analytes installed.`);
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { reloadingOntology = false; }
  }

  const hasContext = $derived(
    !!(info?.description || info?.high_means || info?.low_means || info?.unit_notes)
  );

  $effect(() => { if (analyteId) refresh(); });

  // Window title: "Hemoglobina · MARIANA · blevel-tracker" so the OS bar
  // tells users which analyte they're inspecting at a glance.
  $effect(() => {
    const name = info?.pt_name ?? analyteId;
    const p = patientsWithReadings.find((x) => x.id === effectivePatient);
    setPageTitle(p ? `${name} · ${p.name}` : name);
  });

  const points = $derived(
    chronologicalVisibleReadings
      .filter((r) => r.value != null)
      .map((r) => ({
        date: r.date,
        value: r.value as number,
        // Carry the source report's nickname into the chart so the
        // "labels" toolbar toggle can swap it onto the X axis.
        label: r.source_report_nickname ?? null,
        // Stored flag — drives the optional "colour by flag" toggle.
        // Falls back to null for unflagged rows.
        flag: r.flag ?? null,
      }))
  );

  // Reference band priority — matches the flag-derivation order so the chart
  // and the per-row badges agree:
  //   1. Sex-keyed ontology default_ref (when analyte is sex_dependent and
  //      readings have a single dominant patient sex)
  //   2. Categorical tier labelled "Normal" / "Suficiência" from the ontology
  //   3. First printed range we observe in any reading
  const refBands = $derived.by(() => {
    // Pick the dominant patient sex among visible readings (works whether the
    // page is filtered to one patient or showing many).
    const sexCounts: Record<string, number> = {};
    for (const r of chronologicalVisibleReadings) {
      const s = r.patient_sex ?? '?';
      sexCounts[s] = (sexCounts[s] ?? 0) + 1;
    }
    const dominantSex = Object.entries(sexCounts).sort((a, b) => b[1] - a[1])[0]?.[0] ?? '?';

    // 1. Sex-keyed default
    if (info?.sex_dependent) {
      const ref = defaultRefFor(info.default_ref_json, dominantSex);
      if (ref && (ref.source === 'm' || ref.source === 'f')) {
        return [{ low: ref.low, high: ref.high, tier: 'normal' as const }];
      }
    }
    // For non-sex-keyed analytes, an `all` default is still better than picking
    // a per-row printed range that may have been wrong.
    const allDefault = defaultRefFor(info?.default_ref_json, dominantSex);
    if (allDefault) {
      return [{ low: allDefault.low, high: allDefault.high, tier: 'normal' as const }];
    }

    // 2. Categorical "Normal" tier (Vit D Suficiência, PCR Normal, etc.)
    if (info?.categorical_tiers_json) {
      const tiers = parseTiers(info.categorical_tiers_json);
      const normal = tiers.find((t) =>
        ['normal', 'suficiência', 'suficiencia'].includes(t.label.toLowerCase())
      );
      if (normal && (normal.min != null || normal.max != null)) {
        return [{ low: normal.min ?? null, high: normal.max ?? null, tier: 'normal' as const }];
      }
    }

    // 3. Fallback: first printed range we observed
    const bounds = chronologicalVisibleReadings.find((r) => r.ref_low != null || r.ref_high != null);
    if (!bounds) return [];
    return [{ low: bounds.ref_low, high: bounds.ref_high, tier: 'normal' as const }];
  });

  const unit = $derived(prettyUnit(chronologicalVisibleReadings.find((r) => r.unit)?.unit ?? null));
  const displayName = $derived(info?.pt_name ?? analyteId);

  // Compact stats for the snapshot row.
  const stats = $derived.by(() => {
    const numeric = chronologicalVisibleReadings.filter((r) => r.value != null).map((r) => r.value as number);
    if (numeric.length === 0) return null;
    const sum = numeric.reduce((s, v) => s + v, 0);
    return {
      n: numeric.length,
      latest: numeric[numeric.length - 1],
      latestDate: chronologicalVisibleReadings[chronologicalVisibleReadings.length - 1].date,
      min: Math.min(...numeric),
      max: Math.max(...numeric),
      mean: sum / numeric.length,
      first: numeric[0],
      firstDate: chronologicalVisibleReadings[0].date
    };
  });

  const priorCount = $derived(readings.filter((r) => r.inline_prior).length);

  const activePatientLabel = $derived.by(() => {
    const p = patientsWithReadings.find((x) => x.id === effectivePatient);
    return p ? p.name : 'No patients';
  });

  // Sex driving every flag derivation on this page — taken from the
  // currently selected patient. Falls back to '?' when no patient is
  // selected (empty list, or fresh route load).
  const activeSex = $derived(
    patientsWithReadings.find((x) => x.id === effectivePatient)?.sex ?? '?'
  );

  async function openLoinc(code: string) {
    try {
      await openUrl(`https://loinc.org/${encodeURIComponent(code)}/`);
    } catch (e) {
      toasts.error(e);
    }
  }
</script>

<svelte:window onclick={onPatientPickerWindowClick} />

<div class="space-y-4">
  <div>
    <BackButton fallback="/" />
    <div class="flex items-baseline justify-between gap-3 mt-1 flex-wrap">
      <h1 class="text-xl font-semibold">{displayName}</h1>
      <div class="flex items-center gap-2">
        {#if patientsWithReadings.length > 0}
          <div class="relative" bind:this={patientPickerElement}>
            <button
              type="button"
              class="select analyte-patient-picker"
              aria-haspopup="listbox"
              aria-expanded={patientMenuOpen}
              aria-controls="analyte-patient-options"
              title="Choose which patient's readings to display. Trends are always patient-scoped — values from different patients are never combined into the same line."
              onclick={() => patientMenuOpen ? closePatientMenu() : openPatientMenu()}
            >
              <span class="truncate {activePatient ? '' : 'text-fg3'}">
                {activePatient ? `${activePatient.name} (${activePatient.count})` : 'Select patient…'}
              </span>
              <span aria-hidden="true" class="shrink-0 text-fg3">⌄</span>
            </button>
            {#if patientMenuOpen}
              <div class="analyte-patient-menu" role="presentation">
                <input
                  type="search"
                  class="input w-full"
                  placeholder="Search patients…"
                  aria-label="Search patients for this analyte"
                  bind:value={patientSearch}
                  bind:this={patientSearchElement}
                  onkeydown={onPatientSearchKeydown}
                />
                <div id="analyte-patient-options" class="analyte-patient-options" role="listbox" aria-label="Patients with readings for this analyte">
                  {#each filteredPatients as p, index (p.id)}
                    <button
                      type="button"
                      role="option"
                      aria-selected={p.id === effectivePatient}
                      class="analyte-patient-option {index === patientActiveIndex ? 'analyte-patient-option--active' : ''}"
                      onmousedown={(event) => { event.preventDefault(); selectPatient(p.id); }}
                    >
                      <span class="block truncate font-medium">{p.name}</span>
                      <span class="block text-[10px] text-fg3">{p.count} reading{p.count === 1 ? '' : 's'} · {p.id}</span>
                    </button>
                  {/each}
                  {#if filteredPatients.length === 0}
                    <div class="px-2 py-3 text-center text-xs text-fg3">No patients match.</div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/if}
        <button class="btn text-xs" onclick={onExportCsv} disabled={readings.length === 0}
          title="Download all readings of this analyte as CSV">Export CSV</button>
      </div>
      {#if info}
        <div class="flex items-center gap-2 text-xs text-fg3">
          {#if info.section}<span class="font-mono">{info.section}</span>{/if}
          {#if info.subsection}<span>·</span><span>{info.subsection}</span>{/if}
          {#if info.panel}<span>·</span><span class="pill-muted">{info.panel}</span>{/if}
          {#if info.loinc}
            {@const loinc = info.loinc}
            <span>·</span>
            <button
              type="button"
              class="cursor-pointer font-mono text-accent hover:underline"
              onclick={() => openLoinc(loinc)}
              title="Open this code in the default browser"
            >LOINC {loinc} ↗</button>
          {/if}
        </div>
      {/if}
    </div>
    {#if !info?.is_panel_header}
      <p class="text-xs text-fg3 mt-1">Showing readings for <span class="font-medium text-fg2">{activePatientLabel}</span></p>
    {/if}
    {#if info?.method_annotation}
      <p class="text-xs text-fg3 mt-0.5 font-mono">{info.method_annotation}</p>
    {/if}
    {#if info && info.aliases.length > 0}
      <p class="text-[11px] text-fg3 mt-1">
        Also known as: {#each info.aliases.slice(0, 8) as a, i}{i > 0 ? ', ' : ''}<span class="font-mono">{a}</span>{/each}
      </p>
    {/if}
  </div>

  {#if err}<div class="card p-3 text-sm text-crit">{err}</div>{/if}

  <!-- ─── Panel-header card ─── shown when the analyte ID is a section
       label rather than a measurable test (e.g. "Ionograma sérico").
       Short-circuits the rest of the page since there's nothing to chart
       and the timeseries will always be empty. -->
  {#if info?.is_panel_header}
    <section class="card p-4 border-l-4 border-accent space-y-3">
      <div class="flex items-baseline justify-between gap-3 flex-wrap">
        <h2 class="text-sm font-semibold flex items-center gap-2">
          <span class="ph-pill" aria-hidden="true">SECTION HEADER</span>
          Not a measurable test
        </h2>
        {#if info.panel}
          <span class="text-xs text-fg3">Panel: <span class="font-mono">{info.panel}</span></span>
        {/if}
      </div>
      <p class="text-sm text-fg2">
        <strong>{info.pt_name}</strong> is a heading the lab prints on the PDF
        to introduce a group of related tests — it doesn't have its own value.
        The actual measured analytes for this panel are listed below.
      </p>

      {#if siblingAnalytes.length > 0}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
          {#each siblingAnalytes as s (s.id)}
            <a class="card p-3 hover:border-accent transition-colors flex flex-col gap-0.5"
               href={`/analyte/${s.id}`}>
              <span class="text-sm font-medium">{s.pt_name}</span>
              <span class="text-[11px] text-fg3 font-mono">{s.id}</span>
              <span class="text-[11px] text-fg2 mt-1">
                {s.result_count} reading{s.result_count === 1 ? '' : 's'} on file
              </span>
            </a>
          {/each}
        </div>
      {:else}
        <p class="text-xs text-fg3 italic">No sibling analytes resolved for this panel yet.</p>
      {/if}

      <p class="text-[11px] text-fg3">
        Section headers exist in the ontology so the parser can recognise the
        line on the PDF and skip it. They're informational only.
      </p>
    </section>

  {:else if hasContext}
    <section class="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {#if info?.description}
        <div class="card p-4 lg:col-span-2">
          <h2 class="text-xs font-semibold text-fg2 uppercase tracking-wide mb-1">What it measures</h2>
          <p class="text-sm leading-relaxed">{info.description}</p>
        </div>
      {/if}
      {#if info?.high_means}
        <div class="card p-4 border-l-4 border-crit">
          <h2 class="text-xs font-semibold text-crit uppercase tracking-wide mb-1">When elevated</h2>
          <p class="text-sm leading-relaxed">{info.high_means}</p>
        </div>
      {/if}
      {#if info?.low_means}
        <div class="card p-4 border-l-4 border-warn">
          <h2 class="text-xs font-semibold text-warn uppercase tracking-wide mb-1">When reduced</h2>
          <p class="text-sm leading-relaxed">{info.low_means}</p>
        </div>
      {/if}
      {#if info?.unit_notes}
        <div class="card p-4 lg:col-span-2 border-l-4 border-accent">
          <h2 class="text-xs font-semibold text-accent uppercase tracking-wide mb-1">Reference range & units</h2>
          <p class="text-sm leading-relaxed">{info.unit_notes}</p>
        </div>
      {/if}
    </section>
  {:else if info}
    <!-- Analyte resolved but ontology fields are empty: prompt a reload. -->
    <section class="card p-4 border-l-4 border-warn space-y-2">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-sm font-semibold text-warn">No clinical context populated</h2>
        <button class="btn" disabled={reloadingOntology} onclick={reloadOntology}>
          {reloadingOntology ? 'Reloading…' : 'Reload ontology'}
        </button>
      </div>
      <p class="text-xs text-fg2">
        The analyte resolved (<span class="font-mono">{info.id}</span>) but its description, high/low meanings and unit notes are empty in the database.
        This usually means the seed file was updated after this DB was first set up. Click <em>Reload ontology</em> to re-install.
        If still empty after reloading, this analyte may not yet have descriptions in the bundled seed.
      </p>
    </section>
  {:else if !err}
    <section class="card p-4 border-l-4 border-fg3 space-y-2">
      <p class="text-sm text-fg2">
        Analyte <span class="font-mono">{analyteId}</span> isn't in the ontology. The values still display from your reports, but no clinical context is available.
      </p>
    </section>
  {/if}

  <!-- Categorical tier table when the analyte uses one (Vit D, Ferritina) -->
  {#if info?.categorical_tiers_json}
    {@const tiers = parseTiers(info.categorical_tiers_json)}
    {@const latestNumeric = readings.filter((r) => r.value != null).map((r) => r.value as number).at(-1)}
    {@const activeTier = latestNumeric != null ? matchTier(latestNumeric, tiers) : null}
    {#if tiers.length > 0}
      <section class="card p-4 space-y-2">
        <div class="flex items-baseline justify-between">
          <h2 class="text-sm font-semibold">Reference tiers</h2>
          {#if latestNumeric != null && activeTier}
            <span class="text-xs text-fg2">
              Latest <span class="tabular-nums font-medium">{formatNumber(latestNumeric)}</span> →
              <FlagPill flag={tierToFlag(activeTier.label)} />
            </span>
          {/if}
        </div>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">
          {#each tiers as t}
            {@const isActive = activeTier?.label === t.label}
            <div class="card-tight bg-bg1 {isActive ? 'border-accent' : ''}">
              <div class="text-xs font-medium {isActive ? 'text-accent' : ''}">
                {t.label}
                {#if isActive}<span class="text-[10px] ml-1">(current)</span>{/if}
              </div>
              <div class="text-sm tabular-nums">{formatTierRange(t)}</div>
            </div>
          {/each}
        </div>
      </section>
    {/if}
  {/if}

  {#if info?.is_panel_header}
    <!-- Panel headers never have their own readings — already handled
         above with the sibling-analyte cards. Skip the empty-state +
         chart + table entirely. -->
  {:else if readings.length === 0 && !err}
    <div class="card p-6 text-sm text-fg2">No readings yet for this analyte.</div>
  {:else}
    <!-- Snapshot stats — Latest / First / Min / Max / Mean / Reference.
         The reference card shares this row instead of taking a full-width
         block so all the at-a-glance numbers sit on one line. -->
    {#if stats}
      <section class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
        {#if info && !info.is_panel_header}
          <ReferenceCard
            info={info}
            patientSex={activeSex}
            unit={unit}
            compact
          />
        {/if}
        <div class="card p-3">
          <div class="text-xs text-fg2">Latest</div>
          <div class="text-2xl font-semibold tabular-nums">{formatNumber(stats.latest)}</div>
          <div class="text-[10px] text-fg3">{formatDate(stats.latestDate)}</div>
        </div>
        <div class="card p-3">
          <div class="text-xs text-fg2">First</div>
          <div class="text-xl font-semibold tabular-nums">{formatNumber(stats.first)}</div>
          <div class="text-[10px] text-fg3">{formatDate(stats.firstDate)}</div>
        </div>
        <div class="card p-3">
          <div class="text-xs text-fg2">Min</div>
          <div class="text-xl font-semibold tabular-nums text-warn">{formatNumber(stats.min)}</div>
        </div>
        <div class="card p-3">
          <div class="text-xs text-fg2">Max</div>
          <div class="text-xl font-semibold tabular-nums text-warn">{formatNumber(stats.max)}</div>
        </div>
        <div class="card p-3">
          <div class="text-xs text-fg2">Mean</div>
          <div class="text-xl font-semibold tabular-nums">{formatNumber(stats.mean)}</div>
          <div class="text-[10px] text-fg3">across {stats.n} readings</div>
        </div>
      </section>
    {/if}

    <!-- Reference-source selector — same value as Settings ▸ Charts but
         exposed inline so the user can flip it while reviewing readings.
         Updates the global preference, so the change applies everywhere. -->
    <div class="ref-source-bar">
      <span class="ref-source-bar__label">Reference source</span>
      <div class="seg seg--inline">
        {#each [
          { id: 'auto',    label: '⚖ Auto',         hint: 'Library when usable, fall back to printed' },
          { id: 'library', label: '📖 Library',     hint: 'Always use the analyte ontology' },
          { id: 'printed', label: '🧾 Per-report',  hint: 'Always use the lab\'s printed range' }
        ] as opt}
          <button type="button"
                  class="seg__opt {chartPrefs.referenceSource === opt.id ? 'seg__opt--on' : ''}"
                  title={opt.hint}
                  onclick={() => chartPrefs.setReferenceSource(opt.id as 'auto' | 'library' | 'printed')}>
            {opt.label}
          </button>
        {/each}
      </div>
    </div>

    <section class="card p-4">
      <TimeSeries
        {points}
        {refBands}
        {unit}
        height={340}
        exportName={`${analyteId}_${activePatientLabel.replace(/\s+/g, '_')}`}
      />
    </section>

    <section>
      <div class="flex items-center justify-between mb-2 gap-3 flex-wrap">
        <h2 class="text-sm font-semibold">
          Readings ({visibleReadings.length}{visibleReadings.length !== readings.length ? ` of ${readings.length}` : ''})
        </h2>
        {#if priorCount > 0}
          <label class="text-xs text-fg2 flex items-center gap-1.5 cursor-pointer">
            <input type="checkbox" bind:checked={showPriors} />
            <span>Include {priorCount} value{priorCount === 1 ? '' : 's'} from inline-prior columns of newer reports</span>
          </label>
        {/if}
      </div>
      <div class="card overflow-x-auto">
        <table class="w-full text-sm">
          <thead class="text-fg2 text-xs uppercase tracking-wide">
            <tr class="border-b border-line">
              <th class="text-left px-3 py-2">Date</th>
              <th class="text-left px-3 py-2"
                  title="Time elapsed since the previous reading for this patient. d = days, w = weeks, mo = months, y = years.">
                Δt <span class="text-fg3 text-[10px]" aria-hidden="true">ⓘ</span>
              </th>
              <th class="text-left px-3 py-2">Patient</th>
              <th class="text-right px-3 py-2">Value</th>
              <th class="text-left px-3 py-2">Unit</th>
              <th class="text-right px-3 py-2">Δ</th>
              <th class="text-left px-3 py-2">Flag</th>
              <th class="text-left px-3 py-2">Method</th>
              <th class="text-left px-3 py-2">Source</th>
            </tr>
          </thead>
          <tbody>
            {#each visibleReadings as r, i}
              {@const tiers = info?.categorical_tiers_json ? parseTiers(info.categorical_tiers_json) : []}
              {@const sex = r.patient_sex}
              <!-- Compute the sex-aware default_ref unconditionally — the
                   per-report printed range can be sex-stratified, parser-
                   misread, or simply the wrong sex's column, so we OVERRIDE
                   with the ontology truth whenever the analyte is sex- or
                   tier-dependent. -->
              {@const sexFallback = r.value != null
                                   ? defaultRefFor(info?.default_ref_json, sex) : null}
              {@const preferDefaultRef = info?.cycle_dependent && sex === 'f' && sexFallback}
              {@const matched = !preferDefaultRef && r.value != null && tiers.length > 0
                                ? matchTier(r.value, tiers) : null}
              {@const defaultRef = !matched ? sexFallback : null}
              {@const derivedFlag = matched ? tierToFlag(matched.label)
                                  : (defaultRef && r.value != null
                                       ? flagForDefaultRef(r.value, defaultRef)
                                       : null)}
              <!-- Reference-source policy honours the user's preference:
                   - auto    → derived (ontology) wins, falls back to stored.
                   - library → always ontology; ignores parser-stored flag.
                   - printed → always the lab's printed range as captured
                               by the parser; ignores ontology even when
                               sex/cycle/tier-aware. -->
              {@const finalFlag =
                chartPrefs.referenceSource === 'printed'
                  ? r.flag
                  : chartPrefs.referenceSource === 'library'
                    ? derivedFlag
                    : (derivedFlag ?? r.flag)}
              {@const prevReading = i + 1 < visibleReadings.length ? visibleReadings[i + 1] : null}
              {@const tDelta = prevReading ? formatRelativeSpan(prevReading.date, r.date) : null}
              <tr class="border-b border-line/50 hover:bg-bg3/50 {r.inline_prior ? 'opacity-70' : ''}">
                <td class="px-3 py-2 tabular-nums">{formatDate(r.date)}</td>
                <td class="px-3 py-2">
                  {#if tDelta}
                    <span class="span-pill" title="{tDelta} since {formatDate(prevReading!.date)}">{tDelta}</span>
                  {:else}
                    <span class="text-fg3 text-xs">—</span>
                  {/if}
                </td>
                <td class="px-3 py-2 truncate">{r.patient_name}</td>
                <td class="px-3 py-2 text-right tabular-nums">{formatNumber(r.value)}</td>
                <td class="px-3 py-2 text-fg2">{prettyUnit(r.unit)}</td>
                <td class="px-3 py-2 text-right">
                  <DeltaBadge current={r.value} previous={prevReading?.value ?? null} />
                </td>
                <td class="px-3 py-2"><FlagPill flag={finalFlag} /></td>
                <td class="px-3 py-2 text-fg3 truncate" title={r.method ?? ''}>{r.method ?? '—'}</td>
                <td class="px-3 py-2">
                  <a class="text-accent hover:underline" href={`/report/${r.source_report_id}`}>{r.source_report_id}</a>
                  {#if r.inline_prior}<span class="ml-1 text-[10px] text-fg3">prior</span>{/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>

<style>
  .analyte-patient-picker {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.55rem;
    width: 14rem;
    max-width: min(14rem, 42vw);
    padding: 0.4rem 0.55rem;
    text-align: left;
    cursor: pointer;
  }
  .analyte-patient-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 0.25rem);
    right: 0;
    width: 19rem;
    max-width: min(19rem, 80vw);
    padding: 0.5rem;
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    background: rgb(var(--bg-2));
    box-shadow: 0 0.75rem 2rem rgb(0 0 0 / 0.25);
  }
  .analyte-patient-options {
    max-height: 15rem;
    margin-top: 0.25rem;
    overflow-y: auto;
  }
  .analyte-patient-option {
    display: block;
    width: 100%;
    padding: 0.4rem 0.5rem;
    border: 1px solid transparent;
    border-radius: 0.35rem;
    color: rgb(var(--fg-1));
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease, transform 120ms ease;
  }
  .analyte-patient-option:hover,
  .analyte-patient-option--active {
    border-color: rgb(var(--accent) / 0.35);
    background: rgb(var(--accent) / 0.12);
    transform: translateX(2px);
  }

  /* Reference-source selector that sits right above the chart. Same
     visual vocabulary as the Settings page's segmented controls, but
     compact and inline so it doesn't crowd the analyte header. */
  .ref-source-bar {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    padding: 0.45rem 0.75rem;
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    flex-wrap: wrap;
  }
  .ref-source-bar__label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--fg-3));
    font-weight: 500;
  }
  .seg {
    display: inline-flex;
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
  }
  .seg--inline { width: max-content; }
  .seg__opt {
    padding: 0.3rem 0.65rem;
    font-size: 0.72rem;
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

  /* Caption pill on the panel-header card — distinguishes "this isn't a
     measurable test" from a normal analyte page. */
  .ph-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.45rem;
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-radius: 0.3rem;
  }
</style>
