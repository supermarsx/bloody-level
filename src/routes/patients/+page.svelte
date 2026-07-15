<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { ask } from '@tauri-apps/plugin-dialog';
  import * as admin from '$api/records-admin';
  import { AppError } from '$api/errors';
  import { toasts } from '../../lib/toasts/store.svelte';
  import { formatDate } from '$format/dates';
  import PatientPickerDialog from '$components/PatientPickerDialog.svelte';
  import { ageFromDob } from '$format/dates';

  type SortKey = 'name' | 'reports' | 'latest' | 'abnormal' | 'critical' | 'analytes';

  let overviews = $state<admin.PatientOverview[]>([]);
  let loading = $state(true);
  let sortKey = $state<SortKey>('latest');
  let sortDir = $state<'asc' | 'desc'>('desc');

  // ── Filter bar (collapsed by default; "Filters" button toggles) ────────
  let filtersOpen = $state(false);
  // Each filter is independent — empty = not applied. The bar shows an
  // active-count pill when any filter is non-empty.
  let f = $state<{
    name: string;
    nickname: string;
    sex: '' | 'm' | 'f' | 'x' | '?';
    hasReports: '' | 'yes' | 'no';
    hasNotes: '' | 'yes' | 'no';
    hasNickname: '' | 'yes' | 'no';
    minReports: string;        // numeric string, '' means no constraint
    minAbnormal: string;
    minCritical: string;
    ageMin: string;            // years
    ageMax: string;
    activityWindow: '' | '12m'; // backend supplies a rolling 12-month count
  }>({
    name: '', nickname: '', sex: '',
    hasReports: '', hasNotes: '', hasNickname: '',
    minReports: '', minAbnormal: '', minCritical: '',
    ageMin: '', ageMax: '',
    activityWindow: ''
  });

  function clearFilters() {
    f = {
      name: '', nickname: '', sex: '',
      hasReports: '', hasNotes: '', hasNickname: '',
      minReports: '', minAbnormal: '', minCritical: '',
      ageMin: '', ageMax: '',
      activityWindow: ''
    };
  }

  const activeFilterCount = $derived.by(() => {
    let n = 0;
    for (const v of Object.values(f)) if (v !== '' && v !== null) n++;
    return n;
  });

  // ── Inline editing ─────────────────────────────────────────────────────
  let editingId = $state<string | null>(null);
  let editForm = $state<{
    display_name: string;
    nickname: string;
    sex: admin.PatientSex;
    dob_iso: string;
  }>({
    display_name: '',
    nickname: '',
    sex: '?',
    dob_iso: ''
  });

  // ── Notes dialog ───────────────────────────────────────────────────────
  let notesOpen = $state(false);
  let notesPatient = $state<admin.PatientOverview | null>(null);
  let notesDraft = $state('');
  let savingNotes = $state(false);

  function openNotes(p: admin.PatientOverview) {
    notesPatient = p;
    notesDraft = p.notes ?? '';
    notesOpen = true;
  }
  async function saveNotes() {
    if (!notesPatient) return;
    savingNotes = true;
    try {
      await admin.setPatientNotes(notesPatient.id, notesDraft.trim() ? notesDraft : null);
      toasts.success('Notes saved', notesPatient.display_name);
      notesOpen = false;
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { savingNotes = false; }
  }

  // ── New-patient dialog ─────────────────────────────────────────────────
  let creating = $state(false);
  let newForm = $state<{
    display_name: string;
    nickname: string;
    sex: admin.PatientSex;
    dob_iso: string;
  }>({
    display_name: '',
    nickname: '',
    sex: '?',
    dob_iso: ''
  });
  let savingNew = $state(false);

  // ── Merge dialog ───────────────────────────────────────────────────────
  let mergeOpen = $state(false);
  let mergeSource = $state<admin.PatientOverview | null>(null);

  async function refresh() {
    loading = true;
    try {
      overviews = await admin.patientOverviews();
    } catch (e) { toasts.error(e); }
    finally { loading = false; }
  }

  onMount(refresh);

  const filtered = $derived.by(() => {
    const name = f.name.trim().toLowerCase();
    const nick = f.nickname.trim().toLowerCase();
    const minReports  = f.minReports  ? +f.minReports  : null;
    const minAbnormal = f.minAbnormal ? +f.minAbnormal : null;
    const minCritical = f.minCritical ? +f.minCritical : null;
    const ageMin      = f.ageMin      ? +f.ageMin      : null;
    const ageMax      = f.ageMax      ? +f.ageMax      : null;

    let rows = overviews.filter((p) => {
      if (name && !p.display_name.toLowerCase().includes(name)
          && !p.id.toLowerCase().includes(name)) return false;
      if (nick) {
        const has = (p.nickname ?? '').toLowerCase().includes(nick);
        if (!has) return false;
      }
      if (f.sex && p.sex !== f.sex) return false;

      if (f.hasReports === 'yes' && p.report_count === 0) return false;
      if (f.hasReports === 'no'  && p.report_count !== 0) return false;

      if (f.hasNotes === 'yes' && !p.notes) return false;
      if (f.hasNotes === 'no'  &&  p.notes) return false;

      if (f.hasNickname === 'yes' && !p.nickname) return false;
      if (f.hasNickname === 'no'  &&  p.nickname) return false;

      if (minReports  != null && p.report_count        < minReports)  return false;
      if (minAbnormal != null && p.abnormal_row_count  < minAbnormal) return false;
      if (minCritical != null && p.critical_row_count  < minCritical) return false;

      if (ageMin != null || ageMax != null) {
        const age = ageFromDob(p.dob_iso);
        if (age == null) return false;
        if (ageMin != null && age < ageMin) return false;
        if (ageMax != null && age > ageMax) return false;
      }

      if (f.activityWindow === '12m' && p.recent_report_count === 0) return false;
      return true;
    });
    rows.sort((a, b) => {
      const dir = sortDir === 'asc' ? 1 : -1;
      switch (sortKey) {
        case 'name':     return a.display_name.localeCompare(b.display_name) * dir;
        case 'reports':  return (a.report_count - b.report_count) * dir;
        case 'latest':
          // Treat null as the oldest possible value so patients with no
          // reports sort to the bottom in desc, top in asc.
          return ((a.latest_collection_date_iso ?? '') > (b.latest_collection_date_iso ?? '') ? 1 : -1) * dir;
        case 'abnormal': return (a.abnormal_row_count - b.abnormal_row_count) * dir;
        case 'critical': return (a.critical_row_count - b.critical_row_count) * dir;
        case 'analytes': return (a.distinct_analyte_count - b.distinct_analyte_count) * dir;
      }
    });
    return rows;
  });

  function toggleSort(k: SortKey) {
    if (sortKey === k) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = k;
      sortDir = k === 'name' ? 'asc' : 'desc';
    }
  }

  function startEdit(p: admin.PatientOverview) {
    editingId = p.id;
    editForm = {
      display_name: p.display_name,
      nickname: p.nickname ?? '',
      sex: (p.sex as admin.PatientSex) ?? '?',
      dob_iso: p.dob_iso ?? ''
    };
  }

  async function saveEdit() {
    if (!editingId) return;
    const found = overviews.find((p) => p.id === editingId);
    try {
      await admin.updatePatient({
        id: editingId,
        display_name: editForm.display_name.trim(),
        sex: editForm.sex,
        dob_iso: editForm.dob_iso || null,
        nickname: editForm.nickname.trim() || null,
        // Don't overwrite notes from this form — they have their own editor.
        notes: found?.notes ?? null
      });
      toasts.success('Patient updated');
      editingId = null;
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  async function onDeletePatient(p: admin.PatientOverview) {
    const ok = await ask(
      `Delete patient "${p.display_name}" and ALL ${p.report_count} report${p.report_count === 1 ? '' : 's'}?\n\nThis cannot be undone.`,
      { title: 'Delete patient', kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deletePatient(p.id);
      toasts.success('Patient deleted', p.display_name);
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  function openMerge(p: admin.PatientOverview) {
    mergeSource = p;
    mergeOpen = true;
  }

  async function onMergeInto(target: { id: string; display_name: string }) {
    if (!mergeSource) return;
    const ok = await ask(
      `Merge "${mergeSource.display_name}" INTO "${target.display_name}"?\n\nAll ${mergeSource.report_count} report${mergeSource.report_count === 1 ? '' : 's'} will be reassigned, then "${mergeSource.display_name}" will be deleted. This cannot be undone.`,
      { title: 'Merge patients', kind: 'warning' }
    );
    if (!ok) return;
    try {
      const r = await admin.mergePatients({ source_id: mergeSource.id, target_id: target.id });
      toasts.success('Patients merged', `${r.reports_moved} report${r.reports_moved === 1 ? '' : 's'} moved.`);
      mergeSource = null;
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  function openCreate() {
    newForm = { display_name: '', nickname: '', sex: '?', dob_iso: '' };
    creating = true;
  }

  async function saveCreate() {
    const name = newForm.display_name.trim();
    if (!name) return;
    savingNew = true;
    try {
      const r = await admin.createPatient({
        display_name: name,
        sex: newForm.sex,
        dob_iso: newForm.dob_iso || null
      });
      if (r.created) {
        toasts.success('Patient created', name);
      } else {
        toasts.warn('Patient already existed', `Updated metadata for "${name}".`);
      }
      // Persist nickname separately so the create flow always lands the
      // chosen value, even if the patient row pre-existed.
      const nick = newForm.nickname.trim();
      if (nick) {
        await admin.setPatientNickname(r.id, nick);
      }
      creating = false;
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { savingNew = false; }
  }

  let backfilling = $state(false);
  async function onBackfillSex() {
    backfilling = true;
    try {
      const r = await admin.backfillPatientSex();
      toasts.success(
        'Patient sex re-inferred',
        `${r.patients_updated}/${r.patients_scanned} updated · ${r.patients_still_unknown} still unknown`
      );
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { backfilling = false; }
  }

  let backfillingDob = $state(false);
  async function onBackfillDob() {
    backfillingDob = true;
    try {
      const r = await admin.backfillPatientDob();
      toasts.success(
        'Patient DOB seeded',
        `${r.patients_updated}/${r.patients_scanned} updated (${r.exact_updates} exact · ${r.approximate_updates} approximate) · ${r.patients_still_unknown} still unknown`
      );
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { backfillingDob = false; }
  }

  // KPI cards for the page header — scoped to the last 12 months so the
  // numbers reflect *current* activity rather than lifetime cumulative
  // counts that only ever grow. "Active patients" counts a patient as
  // active iff they have at least one report in the window.
  const totals = $derived.by(() => ({
    activePatients: overviews.filter((p) => p.recent_report_count > 0).length,
    totalPatients:  overviews.length,
    recentReports:  overviews.reduce((s, p) => s + p.recent_report_count, 0),
    recentAbnormal: overviews.reduce((s, p) => s + p.recent_abnormal_row_count, 0),
    recentCritical: overviews.reduce((s, p) => s + p.recent_critical_row_count, 0)
  }));
</script>

<div class="space-y-4">
  <!-- ─── Header + actions ─── -->
  <div class="flex items-end justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">Patients</h1>
      <p class="text-xs text-fg2">
        Every patient with reports in this vault. Edit metadata, merge duplicates, or pre-create patients before their first PDF arrives.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button class="btn" disabled={backfilling} onclick={onBackfillSex}
        title="Re-infer sex from each patient's report raw_text — fixes legacy '?' values.">
        {backfilling ? 'Re-inferring…' : 'Re-infer sex'}
      </button>
      <button class="btn" disabled={backfillingDob} onclick={onBackfillDob}
        title="Seed DOB for patients without one — uses literal 'Data de Nascimento' lines if present, else age + collection date.">
        {backfillingDob ? 'Seeding…' : 'Seed DOB'}
      </button>
      <button class="btn-accent" onclick={openCreate}>+ New patient</button>
    </div>
  </div>

  <!-- ─── KPI strip — labelled once at the group level so we don't repeat
       "last 12 mo" four times across each tile. -->
  <section aria-labelledby="kpi-window-label" class="kpi-section">
    <div id="kpi-window-label" class="kpi-section__label">
      <span class="kpi-section__dot" aria-hidden="true"></span>
      Last 12 months
    </div>
    <div class="kpi-strip">
      <div class="kpi-tile">
        <span class="kpi-tile__label">Active patients</span>
        <span class="kpi-tile__value">{totals.activePatients}</span>
        <span class="kpi-tile__hint">of {totals.totalPatients}</span>
      </div>
      <div class="kpi-tile">
        <span class="kpi-tile__label">Reports</span>
        <span class="kpi-tile__value">{totals.recentReports}</span>
      </div>
      <div class="kpi-tile">
        <span class="kpi-tile__label">Abnormal</span>
        <span class="kpi-tile__value {totals.recentAbnormal > 0 ? 'text-warn' : ''}">{totals.recentAbnormal}</span>
      </div>
      <div class="kpi-tile">
        <span class="kpi-tile__label">Critical</span>
        <span class="kpi-tile__value {totals.recentCritical > 0 ? 'text-crit' : ''}">{totals.recentCritical}</span>
      </div>
    </div>
  </section>

  {#if loading}
    <div class="card p-6 text-sm text-fg2">Loading…</div>
  {:else if overviews.length === 0}
    <div class="card p-6 text-sm text-fg2 space-y-2">
      <p>No patients yet.</p>
      <p>Either ingest a PDF on the <a href="/ingest" class="text-accent hover:underline">Ingest tab</a>
        or click <em>+ New patient</em> to create one manually.</p>
    </div>
  {:else}
    <!-- ─── Filter bar — collapsed by default, expand for column filters ─── -->
    <section class="filter-bar" class:filter-bar--open={filtersOpen}>
      <header class="filter-bar__head">
        <button type="button" class="filter-bar__toggle"
                aria-expanded={filtersOpen}
                onclick={() => (filtersOpen = !filtersOpen)}>
          <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor"
               stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 5 H17 M5 10 H15 M8 15 H12"/>
          </svg>
          <span>Filters</span>
          {#if activeFilterCount > 0}
            <span class="filter-bar__badge">{activeFilterCount}</span>
          {/if}
          <svg viewBox="0 0 20 20" width="12" height="12" fill="none" stroke="currentColor"
               stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
               aria-hidden="true"
               style="transform: rotate({filtersOpen ? 180 : 0}deg); transition: transform 150ms ease;">
            <path d="M5 8 L10 13 L15 8"/>
          </svg>
        </button>
        <span class="filter-bar__count">
          Showing <strong>{filtered.length}</strong> of {overviews.length}
        </span>
        {#if activeFilterCount > 0}
          <button type="button" class="filter-bar__clear" onclick={clearFilters}>
            Clear filters
          </button>
        {/if}
      </header>

      {#if filtersOpen}
        <div class="filter-bar__body">
          <label class="filter-field">
            <span class="filter-field__label">Name / id</span>
            <input class="filter-field__input" type="search"
                   placeholder="contains…" bind:value={f.name} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Nickname</span>
            <input class="filter-field__input" type="search"
                   placeholder="contains…" bind:value={f.nickname} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Sex</span>
            <select class="filter-field__input" bind:value={f.sex}>
              <option value="">Any</option>
              <option value="m">♂ Male</option>
              <option value="f">♀ Female</option>
              <option value="x">⚧ Other</option>
              <option value="?">? Unknown</option>
            </select>
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Has reports</span>
            <select class="filter-field__input" bind:value={f.hasReports}>
              <option value="">Any</option>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Has notes</span>
            <select class="filter-field__input" bind:value={f.hasNotes}>
              <option value="">Any</option>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Has nickname</span>
            <select class="filter-field__input" bind:value={f.hasNickname}>
              <option value="">Any</option>
              <option value="yes">Yes</option>
              <option value="no">No</option>
            </select>
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Recent activity</span>
            <select class="filter-field__input" bind:value={f.activityWindow}
              title="Limits to patients with at least one report inside the rolling 12-month window.">
              <option value="">Any</option>
              <option value="12m">Active in last 12 months</option>
            </select>
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Min reports</span>
            <input class="filter-field__input" type="number" min="0" inputmode="numeric"
                   placeholder="≥" bind:value={f.minReports} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Min abnormal flags</span>
            <input class="filter-field__input" type="number" min="0" inputmode="numeric"
                   placeholder="≥" bind:value={f.minAbnormal} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Min critical flags</span>
            <input class="filter-field__input" type="number" min="0" inputmode="numeric"
                   placeholder="≥" bind:value={f.minCritical} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Age ≥</span>
            <input class="filter-field__input filter-field__input--narrow" type="number" min="0" max="150"
                   inputmode="numeric" placeholder="years" bind:value={f.ageMin} />
          </label>

          <label class="filter-field">
            <span class="filter-field__label">Age ≤</span>
            <input class="filter-field__input filter-field__input--narrow" type="number" min="0" max="150"
                   inputmode="numeric" placeholder="years" bind:value={f.ageMax} />
          </label>
        </div>
      {/if}
    </section>

    <!-- ─── Patients table ─── -->
    <div class="card overflow-x-auto">
      <table class="w-full text-sm">
        <thead class="text-fg2 text-xs uppercase tracking-wide">
          <tr class="border-b border-line">
            {#snippet th(key: SortKey, label: string, align: 'left' | 'right' = 'left', help: string = '')}
              <th class="px-3 py-2 text-{align} cursor-pointer select-none hover:text-fg1"
                  title={help}
                  onclick={() => toggleSort(key)}>
                {label}
                {#if sortKey === key}
                  <span class="text-fg3">{sortDir === 'asc' ? '↑' : '↓'}</span>
                {/if}
                {#if help}
                  <span class="text-fg3 text-[10px] ml-0.5" aria-hidden="true">ⓘ</span>
                {/if}
              </th>
            {/snippet}
            {@render th('name', 'Patient')}
            <th class="px-3 py-2 text-left">Sex</th>
            <th class="px-3 py-2 text-left">DOB</th>
            {@render th('reports', 'Reports', 'right',
              'Number of ingested PDF reports for this patient.')}
            {@render th('latest', 'Latest', 'left',
              'Most recent report collection date.')}
            {@render th('analytes', 'Analytes', 'right',
              'Distinct analytes measured directly on this patient\'s reports — values surfaced via the "Resultados anteriores" inline-prior columns are NOT counted.')}
            {@render th('abnormal', 'Abnormal', 'right',
              'Result rows flagged low / high / abnormal_qual across all reports (excludes inline priors).')}
            {@render th('critical', 'Critical', 'right',
              'Result rows flagged critical_low / critical_high (excludes inline priors).')}
            <th class="px-3 py-2 text-left w-64">Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as p (p.id)}
            {@const isEditing = editingId === p.id}
            <tr class="border-b border-line/40 hover:bg-bg3/40">
              {#if !isEditing}
                <td class="px-3 py-2">
                  <a class="text-accent hover:underline font-medium" href={`/patient/${p.id}`}>
                    {p.nickname ?? p.display_name}
                  </a>
                  {#if p.nickname}
                    <div class="text-[11px] text-fg2">{p.display_name}</div>
                  {/if}
                  <div class="text-[10px] text-fg3 font-mono">{p.id}</div>
                </td>
                <!-- Read-only sex on the overview — sex is set on the
                     patient detail page (or via the inline Edit row), so
                     hover-clicks here can't accidentally flip the global
                     value that drives every flag derivation. -->
                <td class="px-3 py-2"
                    title="Edit sex on the patient detail page (or via the row's edit action).">
                  <span class="sex-pill sex-pill--{p.sex}">
                    {p.sex === 'm' ? 'Male'
                     : p.sex === 'f' ? 'Female'
                     : p.sex === 'x' ? 'Other'
                     : 'Unknown'}
                  </span>
                </td>
                <td class="px-3 py-2 text-xs text-fg2 tabular-nums">
                  {#if p.dob_iso}
                    {formatDate(p.dob_iso)}
                    {#if ageFromDob(p.dob_iso) !== null}
                      <span class="text-fg3">· {ageFromDob(p.dob_iso)} y</span>
                    {/if}
                  {:else}
                    —
                  {/if}
                </td>
                <td class="px-3 py-2 text-right tabular-nums">{p.report_count}</td>
                <td class="px-3 py-2 text-xs tabular-nums">
                  {p.latest_collection_date_iso ? formatDate(p.latest_collection_date_iso) : '—'}
                  {#if p.earliest_collection_date_iso && p.report_count > 1}
                    <div class="text-[10px] text-fg3">since {formatDate(p.earliest_collection_date_iso)}</div>
                  {/if}
                </td>
                <td class="px-3 py-2 text-right tabular-nums">{p.distinct_analyte_count}</td>
                <td class="px-3 py-2 text-right tabular-nums {p.abnormal_row_count > 0 ? 'text-warn' : 'text-fg3'}">{p.abnormal_row_count}</td>
                <td class="px-3 py-2 text-right tabular-nums {p.critical_row_count > 0 ? 'text-crit' : 'text-fg3'}">{p.critical_row_count}</td>
                <td class="px-3 py-2">
                  <div class="flex items-center gap-1">
                    <!-- Icon-only actions; tooltips on hover, accessible
                         names via aria-label. -->
                    <a class="row-icon" href={`/patient/${p.id}`}
                       title="Open patient detail" aria-label="Open">
                      <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M4 10 H16 M11 5 L16 10 L11 15"/>
                      </svg>
                    </a>
                    <button type="button" class="row-icon"
                            onclick={() => openNotes(p)}
                            title={p.notes ? 'Edit clinical notes' : 'Add clinical notes'}
                            aria-label="Notes">
                      <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M5 3 H13 L16 6 V16 a1 1 0 0 1 -1 1 H5 a1 1 0 0 1 -1 -1 V4 a1 1 0 0 1 1 -1 z"/>
                        <path d="M7 8 H13 M7 11 H13 M7 14 H10"/>
                      </svg>
                      {#if p.notes}<span class="row-icon__dot" aria-hidden="true"></span>{/if}
                    </button>
                    <button type="button" class="row-icon"
                            onclick={() => startEdit(p)}
                            title="Edit name, nickname, sex, DOB"
                            aria-label="Edit">
                      <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
                        <path d="M11 6 L14 9"/>
                      </svg>
                    </button>
                    <button type="button" class="row-icon"
                            onclick={() => openMerge(p)}
                            title="Merge this patient into another (e.g. legal name change)"
                            aria-label="Merge">
                      <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M5 4 v5 a3 3 0 0 0 3 3 h6"/>
                        <path d="M15 4 v5 a3 3 0 0 1 -3 3"/>
                        <path d="M11 9 L14 12 L11 15"/>
                      </svg>
                    </button>
                    <button type="button" class="row-icon row-icon--danger"
                            onclick={() => onDeletePatient(p)}
                            title="Delete patient and all reports"
                            aria-label="Delete">
                      <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M5 6 H15 M8 6 V4 H12 V6 M6 6 L7 16 a1 1 0 0 0 1 1 H12 a1 1 0 0 0 1 -1 L14 6"/>
                        <path d="M9 9 V14 M11 9 V14"/>
                      </svg>
                    </button>
                  </div>
                </td>
              {:else}
                <td class="edit-cell" colspan="9">
                  <div class="edit-panel">
                    <div class="edit-panel__header">
                      <span class="edit-panel__title">
                        <span class="edit-panel__pencil" aria-hidden="true">
                          <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                               stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
                            <path d="M11 6 L14 9"/>
                          </svg>
                        </span>
                        Editing patient
                      </span>
                      <span class="edit-panel__id" title="Patient slug — derived from the display name">
                        {p.id}
                      </span>
                    </div>

                    <div class="edit-grid">
                      <label class="edit-field">
                        <span class="edit-field__label">Display name</span>
                        <input class="edit-field__input"
                               bind:value={editForm.display_name}
                               placeholder="LASTNAME GIVEN NAME"
                               onkeydown={(e) => { if (e.key === 'Enter') saveEdit(); else if (e.key === 'Escape') editingId = null; }} />
                        <span class="edit-field__hint">As printed on the lab PDF.</span>
                      </label>

                      <label class="edit-field">
                        <span class="edit-field__label">Nickname <span class="edit-field__opt">— optional</span></span>
                        <input class="edit-field__input"
                               placeholder="e.g. Mom, Dad, J.A."
                               bind:value={editForm.nickname}
                               onkeydown={(e) => { if (e.key === 'Enter') saveEdit(); else if (e.key === 'Escape') editingId = null; }} />
                        <span class="edit-field__hint">Friendly label shown in lists & titles.</span>
                      </label>

                      <div class="edit-field">
                        <span class="edit-field__label">Sex</span>
                        <div class="edit-segmented" role="radiogroup" aria-label="Sex">
                          {#each ['m','f','x','?'] as v}
                            {@const labels: Record<string, string> = { m: '♂ Male', f: '♀ Female', x: '⚧ Other', '?': '? Unknown' }}
                            <button type="button" role="radio"
                                    aria-checked={editForm.sex === v}
                                    class="edit-segmented__opt {editForm.sex === v ? 'edit-segmented__opt--on' : ''}"
                                    onclick={() => (editForm.sex = v as admin.PatientSex)}>
                              {labels[v]}
                            </button>
                          {/each}
                        </div>
                        <span class="edit-field__hint">Used for every flag derivation across this patient's reports.</span>
                      </div>

                      <label class="edit-field">
                        <span class="edit-field__label">Date of birth <span class="edit-field__opt">— optional</span></span>
                        <input type="date" class="edit-field__input edit-field__input--narrow"
                               bind:value={editForm.dob_iso} />
                        <span class="edit-field__hint">
                          {#if editForm.dob_iso && ageFromDob(editForm.dob_iso) !== null}
                            ≈ {ageFromDob(editForm.dob_iso)} years old
                          {:else}
                            Drives the calculated age column.
                          {/if}
                        </span>
                      </label>
                    </div>

                    <div class="edit-panel__footer">
                      <span class="edit-panel__hotkeys">⏎ save · ⎋ cancel</span>
                      <div class="flex gap-2">
                        <button class="btn" onclick={() => (editingId = null)}>Cancel</button>
                        <button class="btn-accent" onclick={saveEdit}>Save changes</button>
                      </div>
                    </div>
                  </div>
                </td>
              {/if}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- ─── New-patient modal ─── -->
{#if creating}
  <div class="fixed inset-0 z-30 bg-black/50 grid place-items-center p-4">
    <button type="button" class="absolute inset-0 cursor-default"
            aria-label="Close dialog"
            onclick={() => (creating = false)}></button>
    <div class="relative card p-5 w-full max-w-md space-y-3"
         role="dialog" aria-modal="true" tabindex="-1">
      <h2 class="text-base font-semibold">New patient</h2>
      <p class="text-xs text-fg2">
        Pre-create a patient before their first PDF arrives. The next ingested
        report with the same name will UPSERT against this row, keeping the
        sex and DOB you set here.
      </p>
      <label class="block text-xs text-fg2">
        Display name
        <input class="input mt-1 block w-full"
               bind:value={newForm.display_name}
               placeholder="e.g. JOÃO PEDRO ALMEIDA"
               onkeydown={(e) => e.key === 'Enter' && saveCreate()} />
      </label>
      <label class="block text-xs text-fg2">
        Nickname <span class="text-fg3">(optional)</span>
        <input class="input mt-1 block w-full"
               bind:value={newForm.nickname}
               placeholder="e.g. Mom, Dad, J.A." />
      </label>
      <div class="grid grid-cols-2 gap-2">
        <label class="block text-xs text-fg2">
          Sex
          <select class="select mt-1 block w-full"
                  bind:value={newForm.sex}>
            <option value="?">Unknown</option>
            <option value="m">Male</option>
            <option value="f">Female</option>
            <option value="x">Other</option>
          </select>
        </label>
        <label class="block text-xs text-fg2">
          DOB
          <input type="date"
                 class="input mt-1 block w-full"
                 bind:value={newForm.dob_iso} />
        </label>
      </div>
      <div class="flex justify-end gap-2 pt-1">
        <button class="btn" onclick={() => (creating = false)}>Cancel</button>
        <button class="btn-accent" disabled={savingNew || !newForm.display_name.trim()}
                onclick={saveCreate}>
          {savingNew ? 'Saving…' : 'Create patient'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- ─── Merge picker (reuses existing dialog) ─── -->
<PatientPickerDialog
  bind:open={mergeOpen}
  excludeId={mergeSource?.id ?? undefined}
  title={mergeSource ? `Merge "${mergeSource.display_name}" into…` : 'Merge into…'}
  onPick={onMergeInto}
/>

<!-- ─── Notes editor ─── -->
{#if notesOpen && notesPatient}
  <div class="fixed inset-0 z-30 bg-black/50 grid place-items-center p-4">
    <button type="button" class="absolute inset-0 cursor-default"
            aria-label="Close dialog"
            onclick={() => (notesOpen = false)}></button>
    <div class="relative card p-5 w-full max-w-xl space-y-3"
         role="dialog" aria-modal="true" tabindex="-1">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-base font-semibold">
          Notes · {notesPatient.nickname ?? notesPatient.display_name}
        </h2>
        <span class="text-[11px] text-fg3">Markdown not rendered — plain text only</span>
      </div>
      <p class="text-xs text-fg2">
        Free-form clinical context that doesn't fit the structured fields:
        allergies, family history, treatment plan, ongoing conditions.
      </p>
      <textarea
        class="input block w-full font-mono"
        rows="10"
        placeholder="e.g. Type 2 diabetes since 2018, on metformin 1000mg BD…"
        bind:value={notesDraft}
      ></textarea>
      <div class="flex justify-end gap-2 pt-1">
        <button class="btn" onclick={() => (notesOpen = false)}>Cancel</button>
        <button class="btn-accent" disabled={savingNotes} onclick={saveNotes}>
          {savingNotes ? 'Saving…' : 'Save notes'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Group-level "Last 12 months" caption sits above the strip, visually
     wrapping the four tiles as one logical unit. The pill on the left is a
     visual anchor; saves repeating "last 12 mo" four times. */
  .kpi-section {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }
  .kpi-section__label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    width: max-content;
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--fg-2));
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 9999px;
    padding: 0.15rem 0.55rem;
  }
  .kpi-section__dot {
    width: 6px;
    height: 6px;
    border-radius: 9999px;
    background: rgb(var(--accent));
  }
  .kpi-strip {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.4rem;
  }
  @media (min-width: 640px) {
    .kpi-strip { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  }

  /* ── Collapsible filter bar ──────────────────────────────────────────
     Sits directly above the table. Header is always visible; body is
     hidden until the user opens it. */
  .filter-bar {
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    overflow: hidden;
  }
  .filter-bar--open { border-color: rgb(var(--accent) / 0.4); }
  .filter-bar__head {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.4rem 0.75rem;
  }
  .filter-bar__toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.3rem 0.65rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    font-size: 0.75rem;
    color: rgb(var(--fg-1));
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease;
  }
  .filter-bar__toggle:hover {
    background: rgb(var(--bg-3));
    border-color: rgb(var(--accent) / 0.5);
  }
  .filter-bar__badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.2rem;
    height: 1.2rem;
    padding: 0 0.35rem;
    border-radius: 9999px;
    background: rgb(var(--accent));
    color: white;
    font-size: 0.65rem;
    font-weight: 600;
  }
  .filter-bar__count {
    font-size: 0.75rem;
    color: rgb(var(--fg-2));
    flex: 1 1 auto;
  }
  .filter-bar__count strong {
    color: rgb(var(--fg-1));
    font-weight: 600;
  }
  .filter-bar__clear {
    font-size: 0.7rem;
    color: rgb(var(--fg-3));
    background: transparent;
    border: 0;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: 0.3rem;
    transition: background 120ms ease, color 120ms ease;
  }
  .filter-bar__clear:hover {
    color: rgb(var(--crit));
    background: rgb(var(--crit) / 0.08);
  }
  .filter-bar__body {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.7rem 1rem;
    padding: 0.6rem 0.85rem 0.85rem;
    border-top: 1px dashed rgb(var(--line));
  }
  @media (min-width: 720px) {
    .filter-bar__body { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  }
  @media (min-width: 1100px) {
    .filter-bar__body { grid-template-columns: repeat(6, minmax(0, 1fr)); }
  }
  .filter-field { display: flex; flex-direction: column; gap: 0.25rem; min-width: 0; }
  .filter-field__label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
    color: rgb(var(--fg-3));
  }
  .filter-field__input {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    padding: 0.3rem 0.5rem;
    font-size: 0.8rem;
    color: rgb(var(--fg-1));
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  .filter-field__input:hover { border-color: rgb(var(--fg-3)); }
  .filter-field__input:focus {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18);
  }
  .filter-field__input--narrow { max-width: 7rem; }

  /* Compact KPI tile — label + value on one line, optional sub-hint. About
     half the vertical footprint of the previous card layout, while still
     readable at a glance. */
  .kpi-tile {
    display: inline-flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.35rem 0.6rem;
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    min-height: 1.85rem;
    overflow: hidden;
  }
  .kpi-tile__label {
    font-size: 0.7rem;
    color: rgb(var(--fg-2));
    white-space: nowrap;
    flex: 1 1 auto;
  }
  .kpi-tile__value {
    font-size: 1rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: rgb(var(--fg-1));
    line-height: 1;
  }
  .kpi-tile__hint {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    white-space: nowrap;
  }

  /* ── Inline edit panel ───────────────────────────────────────────────
     Replaces the row content in-place when a patient is being edited.
     Goal: make this feel like a deliberate "drawer" instead of a cramped
     row of inputs. */
  .edit-cell {
    padding: 0 !important; /* let the panel handle its own padding */
    background: rgb(var(--accent) / 0.05);
    border-left: 3px solid rgb(var(--accent));
  }
  .edit-panel {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 0.9rem 1rem 0.85rem;
  }
  .edit-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }
  .edit-panel__title {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--accent));
  }
  .edit-panel__pencil {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    border-radius: 9999px;
    background: rgb(var(--accent) / 0.15);
    color: rgb(var(--accent));
  }
  .edit-panel__id {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.7rem;
    color: rgb(var(--fg-3));
  }
  .edit-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.75rem 1rem;
  }
  @media (min-width: 720px) {
    .edit-grid { grid-template-columns: 1.4fr 1fr 1.4fr 0.8fr; }
  }
  .edit-field { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; }
  .edit-field__label {
    font-size: 0.7rem;
    font-weight: 500;
    color: rgb(var(--fg-2));
  }
  .edit-field__opt {
    color: rgb(var(--fg-3));
    font-weight: 400;
    font-style: italic;
  }
  .edit-field__input {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
    color: rgb(var(--fg-1));
    transition: border-color 120ms ease, box-shadow 120ms ease, background 120ms ease;
  }
  .edit-field__input:hover { border-color: rgb(var(--fg-3)); }
  .edit-field__input:focus {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18);
    background: rgb(var(--bg-1));
  }
  .edit-field__input--narrow { max-width: 11rem; }
  .edit-field__hint {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    line-height: 1.3;
  }
  .edit-segmented {
    display: inline-flex;
    flex-wrap: wrap;
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
    width: max-content;
  }
  .edit-segmented__opt {
    padding: 0.35rem 0.6rem;
    font-size: 0.7rem;
    font-weight: 500;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 0;
    border-right: 1px solid rgb(var(--line));
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }
  .edit-segmented__opt:last-child { border-right: 0; }
  .edit-segmented__opt:hover { background: rgb(var(--bg-2)); color: rgb(var(--fg-1)); }
  .edit-segmented__opt--on {
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-weight: 600;
  }
  .edit-panel__footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding-top: 0.55rem;
    border-top: 1px dashed rgb(var(--line));
  }
  .edit-panel__hotkeys {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  /* Read-only sex badge for the overview table. Same colour mapping the
     sex toggle uses on the detail page, but rendered as a flat pill so
     it's clearly non-interactive. */
  .sex-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.7rem;
    font-weight: 500;
    border: 1px solid;
    cursor: default;
  }
  .sex-pill--m {
    color: rgb(var(--accent));
    background: rgb(var(--accent) / 0.10);
    border-color: rgb(var(--accent) / 0.35);
  }
  .sex-pill--f {
    color: rgb(var(--warn));
    background: rgb(var(--warn) / 0.10);
    border-color: rgb(var(--warn) / 0.35);
  }
  .sex-pill--x {
    color: rgb(var(--fg-2));
    background: rgb(var(--bg-3));
    border-color: rgb(var(--line));
  }
  /* `?` (unknown) — muted style so it reads as "missing data" rather
     than a confident value. */
  .sex-pill--\? {
    color: rgb(var(--fg-3));
    background: rgb(var(--bg-3));
    border-color: rgb(var(--line));
    border-style: dashed;
  }

  /* Slightly chunkier icon-only action buttons used in the row's last column.
     Hover/focus brings full opacity + subtle accent ring. The interior SVGs
     auto-scale via `width="14"` set inline on each icon — bumping the
     button frame to 2rem keeps the visual weight closer to a real button
     while still reading as icon-only. */
  :global(.row-icon) {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    border: 1px solid transparent;
    border-radius: 0.45rem;
    color: rgb(var(--fg-3));
    background: transparent;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  :global(.row-icon svg) { width: 16px; height: 16px; }
  :global(.row-icon:hover) {
    background: rgb(var(--bg-2));
    color: rgb(var(--accent));
    border-color: rgb(var(--accent) / 0.4);
  }
  :global(.row-icon:focus-visible) {
    outline: 2px solid rgb(var(--accent));
    outline-offset: 1px;
  }
  :global(.row-icon--danger:hover) {
    color: rgb(var(--crit));
    border-color: rgb(var(--crit) / 0.4);
    background: rgb(var(--crit) / 0.08);
  }
  /* Tiny indicator on the Notes icon when notes exist. */
  :global(.row-icon__dot) {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 6px;
    height: 6px;
    border-radius: 9999px;
    background: rgb(var(--accent));
  }
</style>
