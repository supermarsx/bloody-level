<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { ask } from '@tauri-apps/plugin-dialog';
  import {
    listReports,
    listPatients,
    patientAnalyteSummaries,
    listFlaggedAnalytes,
    type ReportSummary,
    type PatientSummary,
    type AnalyteReading,
    type PatientAnalyteSummary,
  } from '$api/reports';
  import * as admin from '$api/records-admin';
  import { AppError } from '$api/errors';
  import { toasts } from '../../../lib/toasts/store.svelte';
  import { formatDate, ageFromDob, formatRelativeSpan } from '$format/dates';
  import PatientPickerDialog from '$components/patient-picker-dialog.svelte';
  import { setPageTitle } from '$lib/title.svelte';
  import BackButton from '$components/back-button.svelte';
  import HrtSection from '$components/hrt-section.svelte';
  import { hrtMilestoneFor } from '$format/hrt-milestone';
  import FlagPill from '$charts/flag-pill.svelte';
  import { formatNumber, formatDelta } from '$format/numbers';
  import Icon, { type IconName } from '$components/icon.svelte';

  let patientId = $derived($page.params.id ?? '');
  let reports = $state<ReportSummary[]>([]);
  let patient = $state<PatientSummary | null>(null);
  let err = $state<AppError | null>(null);
  let analyteSummaries = $state<PatientAnalyteSummary[]>([]);
  let abnormalAnalyteIds = $state<Set<string>>(new Set());
  let subclinicalAnalyteIds = $state<Set<string>>(new Set());
  let dashboardLoading = $state(false);

  function sexIcon(sex: string): IconName {
    return sex === 'm' ? 'male' : sex === 'f' ? 'female' : sex === 'x' ? 'gender' : 'info';
  }
  function sexLabel(sex: string): string {
    return sex === 'm' ? 'Male' : sex === 'f' ? 'Female' : sex === 'x' ? 'Other' : 'Unknown';
  }

  type AnalyteCategory = 'abnormal' | 'elevated' | 'subclinical';
  type TrendWindowDays = 30 | 90 | 180 | 365;
  type PatientAnalyteCard = {
    id: string;
    name: string;
    readings: AnalyteReading[];
    latest: AnalyteReading;
    previous: AnalyteReading | null;
    delta: ReturnType<typeof formatDelta> | null;
    category: AnalyteCategory | null;
  };

  let reportFilter = $state('');
  let analyteFilter = $state('');
  let analyteCategory = $state<'all' | AnalyteCategory>('all');
  let trendWindowDays = $state<TrendWindowDays>(365);
  const trendWindows: { id: TrendWindowDays; label: string }[] = [
    { id: 30, label: '30 days' },
    { id: 90, label: '3 months' },
    { id: 180, label: '6 months' },
    { id: 365, label: '1 year' },
  ];

  // ── Edit metadata (inline form) ────────────────────────────────────────
  let editing = $state(false);
  let editForm = $state<{
    display_name: string;
    nickname: string;
    sex: 'm' | 'f' | 'x' | '?';
    dob_iso: string;
    hrt_start_iso: string;
  }>({ display_name: '', nickname: '', sex: '?', dob_iso: '', hrt_start_iso: '' });

  // ── Inline nickname rename in the header ───────────────────────────────
  let editingNickname = $state(false);
  let nicknameDraft = $state('');

  // ── Notes editor ───────────────────────────────────────────────────────
  let editingNotes = $state(false);
  let notesDraft = $state('');
  let savingNotes = $state(false);

  // ── Merge dialog ───────────────────────────────────────────────────────
  let mergeDialogOpen = $state(false);

  async function refresh() {
    err = null;
    dashboardLoading = true;
    try {
      const [reps, patients, summaries, flagged] = await Promise.all([
        listReports(patientId),
        listPatients(),
        patientAnalyteSummaries(patientId),
        listFlaggedAnalytes(patientId),
      ]);
      reports = reps;
      patient = patients.find((p) => p.id === patientId) ?? null;
      analyteSummaries = summaries;
      abnormalAnalyteIds = new Set(flagged.abnormal);
      subclinicalAnalyteIds = new Set(flagged.subclinical);
    } catch (e) { err = AppError.fromUnknown(e); }
    finally { dashboardLoading = false; }
  }

  $effect(() => { if (patientId) refresh(); });
  $effect(() => {
    setPageTitle(patient?.nickname ?? patient?.display_name ?? 'Patient');
  });

  // ── Edit metadata ──────────────────────────────────────────────────────
  function startEdit() {
    if (!patient) return;
    editForm = {
      display_name: patient.display_name,
      nickname: patient.nickname ?? '',
      sex: (patient.sex as 'm' | 'f' | 'x' | '?') ?? '?',
      dob_iso: patient.dob_iso ?? '',
      hrt_start_iso: patient.hrt_start_iso ?? ''
    };
    editing = true;
  }

  async function saveEdit() {
    if (!patient) return;
    try {
      await admin.updatePatient({
        id: patientId,
        display_name: editForm.display_name.trim() || patient.display_name,
        sex: editForm.sex,
        dob_iso: editForm.dob_iso.trim() || null,
        nickname: editForm.nickname.trim() || null,
        // Don't overwrite notes from this form — they have their own editor.
        notes: patient.notes ?? null
      });
      // HRT start uses a dedicated command; only call it when the value
      // actually changed so we don't churn the column unnecessarily.
      const next = editForm.hrt_start_iso.trim() || null;
      if (next !== (patient.hrt_start_iso ?? null)) {
        await admin.setPatientHrtStart(patientId, next);
      }
      toasts.success('Patient updated');
      editing = false;
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  // ── Inline nickname (header) ───────────────────────────────────────────
  function startEditNickname() {
    nicknameDraft = patient?.nickname ?? '';
    editingNickname = true;
  }
  async function saveNickname() {
    if (!patient) return;
    const next = nicknameDraft.trim();
    try {
      await admin.setPatientNickname(patientId, next.length === 0 ? null : next);
      toasts.success('Nickname saved', next || 'cleared');
      editingNickname = false;
      await refresh();
    } catch (e) { toasts.error(e); }
  }
  function cancelEditNickname() {
    editingNickname = false;
  }

  // ── Notes editor ───────────────────────────────────────────────────────
  function startEditNotes() {
    notesDraft = patient?.notes ?? '';
    editingNotes = true;
  }
  async function saveNotes() {
    if (!patient) return;
    savingNotes = true;
    try {
      await admin.setPatientNotes(patientId, notesDraft.trim() ? notesDraft : null);
      toasts.success('Notes saved');
      editingNotes = false;
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { savingNotes = false; }
  }
  function cancelEditNotes() {
    editingNotes = false;
  }

  // ── Per-row report nickname editing ────────────────────────────────────
  let editingReportId = $state<string | null>(null);
  let reportNickDraft = $state('');

  function startEditReportNickname(r: ReportSummary) {
    editingReportId = r.id;
    reportNickDraft = r.nickname ?? '';
  }
  async function saveReportNickname() {
    if (!editingReportId) return;
    const next = reportNickDraft.trim();
    try {
      await admin.setReportNickname(editingReportId, next.length === 0 ? null : next);
      // Optimistic local update so the row text changes without a full refresh.
      const target = reports.find((rr) => rr.id === editingReportId);
      if (target) target.nickname = next.length === 0 ? null : next;
      reports = [...reports];
      editingReportId = null;
      toasts.success('Report nickname saved', next || 'cleared');
    } catch (e) { toasts.error(e); }
  }
  function cancelEditReportNickname() {
    editingReportId = null;
  }

  // ── Merge / delete ─────────────────────────────────────────────────────
  async function onMergeInto(target: PatientSummary) {
    if (!patient) return;
    const ok = await ask(
      `Merge "${patient.display_name}" INTO "${target.display_name}"?\n\nAll ${patient.report_count} report${patient.report_count === 1 ? '' : 's'} will be reassigned to ${target.display_name}, then ${patient.display_name} will be deleted. This cannot be undone.`,
      { title: 'Merge patients', kind: 'warning' }
    );
    if (!ok) return;
    try {
      const r = await admin.mergePatients({ source_id: patientId, target_id: target.id });
      toasts.success(
        'Patients merged',
        `${r.reports_moved} report${r.reports_moved === 1 ? '' : 's'} moved to ${target.display_name}.`
      );
      await goto(`/patient/${target.id}`);
    } catch (e) { toasts.error(e); }
  }

  async function onDeletePatient() {
    if (!patient) return;
    const ok = await ask(
      `Delete patient "${patient.display_name}" and ALL ${patient.report_count} report${patient.report_count === 1 ? '' : 's'}?\n\nThis cannot be undone.`,
      { title: 'Delete patient', kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deletePatient(patientId);
      toasts.success('Patient deleted');
      await goto('/patients');
    } catch (e) { toasts.error(e); }
  }

  async function onDeleteReport(r: ReportSummary) {
    const ok = await ask(
      `Delete report ${formatDate(r.collection_date_iso)}?`,
      { title: 'Delete report', kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deleteReport(r.id);
      toasts.success('Report deleted');
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  const filteredReports = $derived.by(() => {
    const q = reportFilter.trim().toLowerCase();
    if (!q) return reports;
    return reports.filter((r) =>
      [r.nickname, r.collection_date_iso, r.id, r.annotations]
        .some((value) => value?.toLowerCase().includes(q))
    );
  });

  const analyteCards = $derived.by<PatientAnalyteCard[]>(() => {
    return analyteSummaries.flatMap((summary) => {
      const readings = summary.readings.filter((r) => !r.inline_prior);
      const latest = readings.at(-1);
      if (!latest) return [];
      const numeric = readings.filter((r) => r.value != null);
      const current = numeric.at(-1);
      const previous = numeric.length > 1 ? numeric.at(-2) ?? null : null;
      const delta = current && previous
        ? formatDelta(current.value as number, previous.value as number)
        : null;
      const latestFlag = latest.flag ?? null;
      const category: AnalyteCategory | null = latestFlag === 'high' || latestFlag === 'critical_high'
        ? 'elevated'
        : abnormalAnalyteIds.has(summary.analyte_id)
          ? 'abnormal'
          : subclinicalAnalyteIds.has(summary.analyte_id)
            ? 'subclinical'
            : null;
      return [{
        id: summary.analyte_id,
        name: summary.analyte_name,
        readings,
        latest,
        previous,
        delta,
        category,
      }];
    });
  });

  const flaggedAnalytes = $derived.by(() => {
    const q = analyteFilter.trim().toLowerCase();
    return analyteCards
      .filter((a) => a.category !== null)
      .filter((a) => analyteCategory === 'all' || a.category === analyteCategory)
      .filter((a) => !q || a.name.toLowerCase().includes(q) || a.id.toLowerCase().includes(q))
      .sort((a, b) => b.latest.date.localeCompare(a.latest.date) || a.name.localeCompare(b.name));
  });

  const trendCards = $derived.by<PatientAnalyteCard[]>(() => {
    const dates = analyteCards
      .map((a) => Date.parse(a.latest.date))
      .filter((date) => Number.isFinite(date));
    const anchor = dates.length > 0 ? Math.max(...dates) : Date.now();
    const cutoff = anchor - Math.min(trendWindowDays, 365) * 24 * 60 * 60 * 1000;

    return analyteCards.flatMap((analyte) => {
      const readings = analyte.readings.filter((reading) => {
        const timestamp = Date.parse(reading.date);
        return Number.isFinite(timestamp) && timestamp >= cutoff;
      });
      const latest = readings.at(-1);
      if (!latest) return [];
      const numeric = readings.filter((reading) => reading.value != null);
      const current = numeric.at(-1);
      const previous = numeric.length > 1 ? numeric.at(-2) ?? null : null;
      return [{
        ...analyte,
        readings,
        latest,
        previous,
        delta: current && previous
          ? formatDelta(current.value as number, previous.value as number)
          : null,
      }];
    });
  });

  function setTrendWindow(value: number) {
    const candidate = trendWindows.find((window) => window.id === value)?.id;
    trendWindowDays = candidate ?? 365;
  }

  const mainAnalytes = $derived(
    [...trendCards]
      .sort((a, b) => b.readings.length - a.readings.length || a.name.localeCompare(b.name))
      .slice(0, 6)
  );

  const biggestDeltas = $derived(
    [...trendCards]
      .filter((a) => a.delta !== null)
      .sort((a, b) => {
        const aMagnitude = Math.abs(a.delta!.pct) || Math.abs(a.delta!.abs);
        const bMagnitude = Math.abs(b.delta!.pct) || Math.abs(b.delta!.abs);
        return bMagnitude - aMagnitude;
      })
      .slice(0, 6)
  );

  function sparklinePoints(readings: AnalyteReading[]): string {
    const values = readings
      .filter((r) => r.value != null)
      .map((r) => r.value as number);
    if (values.length < 2) return '';
    const min = Math.min(...values);
    const max = Math.max(...values);
    const span = max - min || 1;
    return values.map((value, index) => {
      const x = (index / (values.length - 1)) * 100;
      const y = 25 - ((value - min) / span) * 20;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
  }

  function readingValue(reading: AnalyteReading): string {
    return reading.value != null ? formatNumber(reading.value) : reading.qualitative ?? '—';
  }

  function categoryLabel(category: AnalyteCategory | null): string {
    return category === 'elevated' ? 'Elevated'
      : category === 'subclinical' ? 'Subclinical'
        : 'Abnormal';
  }

  function categoryCount(category: 'all' | AnalyteCategory): number {
    return category === 'all'
      ? analyteCards.filter((a) => a.category !== null).length
      : analyteCards.filter((a) => a.category === category).length;
  }

  // The header title prefers the nickname if set; the canonical name then
  // appears as a smaller subtitle so it's still visible/auditable.
  const headerTitle = $derived(patient?.nickname ?? patient?.display_name ?? patientId);
  const headerSub   = $derived(patient?.nickname ? patient?.display_name : null);
</script>

<div class="space-y-4">
  <!-- ─── Header ─── -->
  <div class="flex items-end justify-between gap-3 flex-wrap">
    <div class="min-w-0 flex-1">
      <BackButton fallback="/patients" />
      <h1 class="text-xl font-semibold mt-1 flex items-baseline gap-2 flex-wrap">
        {#if !editingNickname}
          <span class="truncate" title={headerTitle}>{headerTitle}</span>
          <button
            type="button"
            class="text-xs text-fg3 hover:text-accent ml-1"
            onclick={startEditNickname}
            title={patient?.nickname ? 'Edit nickname' : 'Add a friendly nickname for this patient'}
          ><Icon name={patient?.nickname ? 'edit' : 'plus'} size={13} /> {patient?.nickname ? 'rename' : 'nickname'}</button>
        {:else}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="input text-base flex-1 min-w-[12rem]"
            placeholder="e.g. Mom, Dad, J.A."
            bind:value={nicknameDraft}
            autofocus
            onkeydown={(e) => {
              if (e.key === 'Enter') { e.preventDefault(); saveNickname(); }
              else if (e.key === 'Escape') { e.preventDefault(); cancelEditNickname(); }
            }}
          />
          <button class="btn text-xs" onclick={saveNickname}>Save</button>
          <button class="btn text-xs" onclick={cancelEditNickname}>Cancel</button>
        {/if}
      </h1>
      {#if headerSub}
        <p class="text-sm text-fg2 mt-0.5">{headerSub}</p>
      {/if}
      {#if patient}
        <p class="text-xs text-fg3 mt-1 flex items-center gap-2 flex-wrap">
          {#if patient.sex}
            <button type="button"
                    class="sex-pill sex-pill--{patient.sex}"
                    onclick={startEdit}
                    title="Click to change — sex is the global truth driving every flag derivation. Edit it through the patient metadata form.">
              <Icon name={sexIcon(patient.sex)} size={13} /> {sexLabel(patient.sex)}
            </button>
          {/if}
          <span>{patient.report_count} report{patient.report_count === 1 ? '' : 's'}</span>
          {#if patient.latest_collection_date_iso}
            <span>· latest {formatDate(patient.latest_collection_date_iso)}</span>
          {/if}
          {#if patient.dob_iso}
            <span>· DOB {formatDate(patient.dob_iso)}{#if ageFromDob(patient.dob_iso) !== null}<span class="text-fg2 ml-1">({ageFromDob(patient.dob_iso)} y)</span>{/if}</span>
          {/if}
          <span>· <span class="font-mono">{patient.id}</span></span>
        </p>
      {/if}
    </div>

    {#if patient && !editing}
      <!-- Icon-only action cluster, matching the patients list. -->
      <div class="flex items-center gap-1">
        <button type="button" class="row-icon"
                onclick={startEditNotes}
                title={patient.notes ? 'Edit clinical notes' : 'Add clinical notes'}
                aria-label="Notes">
          <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M5 3 H13 L16 6 V16 a1 1 0 0 1 -1 1 H5 a1 1 0 0 1 -1 -1 V4 a1 1 0 0 1 1 -1 z"/>
            <path d="M7 8 H13 M7 11 H13 M7 14 H10"/>
          </svg>
          {#if patient.notes}<span class="row-icon__dot" aria-hidden="true"></span>{/if}
        </button>
        <button type="button" class="row-icon"
                onclick={startEdit}
                title="Edit name, nickname, sex, DOB"
                aria-label="Edit">
          <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
            <path d="M11 6 L14 9"/>
          </svg>
        </button>
        <button type="button" class="row-icon"
                onclick={() => (mergeDialogOpen = true)}
                title="Merge this patient into another (e.g. legal name change)"
                aria-label="Merge">
          <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M5 4 v5 a3 3 0 0 0 3 3 h6"/>
            <path d="M15 4 v5 a3 3 0 0 1 -3 3"/>
            <path d="M11 9 L14 12 L11 15"/>
          </svg>
        </button>
        <button type="button" class="row-icon row-icon--danger"
                onclick={onDeletePatient}
                title="Delete patient and all reports"
                aria-label="Delete">
          <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M5 6 H15 M8 6 V4 H12 V6 M6 6 L7 16 a1 1 0 0 0 1 1 H12 a1 1 0 0 0 1 -1 L14 6"/>
            <path d="M9 9 V14 M11 9 V14"/>
          </svg>
        </button>
      </div>
    {/if}
  </div>

  <PatientPickerDialog
    bind:open={mergeDialogOpen}
    excludeId={patientId}
    title="Merge into which patient?"
    confirmLabel="Merge"
    onPick={onMergeInto}
  />

  <!-- ─── Edit metadata ─── -->
  {#if editing && patient}
    <section class="edit-card">
      <header class="edit-card__header">
        <span class="edit-card__title">
          <span class="edit-card__pencil" aria-hidden="true">
            <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                 stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
              <path d="M11 6 L14 9"/>
            </svg>
          </span>
          Editing patient
        </span>
        <span class="edit-card__id" title="Patient slug — derived from the display name">{patient.id}</span>
      </header>

      <div class="edit-card__grid">
        <label class="edit-field">
          <span class="edit-field__label">Display name</span>
          <input class="edit-field__input"
                 placeholder="LASTNAME GIVEN NAME"
                 bind:value={editForm.display_name}
                 onkeydown={(e) => { if (e.key === 'Enter') saveEdit(); else if (e.key === 'Escape') editing = false; }} />
          <span class="edit-field__hint">As printed on the lab PDF.</span>
        </label>

        <label class="edit-field">
          <span class="edit-field__label">Nickname <span class="edit-field__opt">— optional</span></span>
          <input class="edit-field__input"
                 placeholder="e.g. Mom, Dad, J.A."
                 bind:value={editForm.nickname}
                 onkeydown={(e) => { if (e.key === 'Enter') saveEdit(); else if (e.key === 'Escape') editing = false; }} />
          <span class="edit-field__hint">Friendly label shown in lists & titles.</span>
        </label>

        <div class="edit-field">
          <span class="edit-field__label">Sex</span>
          <div class="edit-segmented" role="radiogroup" aria-label="Sex">
            {#each ['m','f','x','?'] as v}
              <button type="button" role="radio"
                      aria-checked={editForm.sex === v}
                      class="edit-segmented__opt {editForm.sex === v ? 'edit-segmented__opt--on' : ''}"
                      onclick={() => (editForm.sex = v as 'm' | 'f' | 'x' | '?')}>
                <Icon name={sexIcon(v)} size={13} /> {sexLabel(v)}
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
              Drives the calculated age in the header.
            {/if}
          </span>
        </label>

        <label class="edit-field">
          <span class="edit-field__label">
            HRT start date <span class="edit-field__opt">— optional</span>
          </span>
          <input type="date" class="edit-field__input edit-field__input--narrow"
                 bind:value={editForm.hrt_start_iso} />
          <span class="edit-field__hint">
            Anchors the HRT timeline. Each report gets a "Day N / Month N / Year N" milestone
            measured from this date. Leave blank to hide the timeline section.
          </span>
        </label>
      </div>

      <footer class="edit-card__footer">
        <span class="edit-card__hotkeys">⏎ save · ⎋ cancel</span>
        <div class="flex gap-2">
          <button class="btn" onclick={() => (editing = false)}>Cancel</button>
          <button class="btn-accent" onclick={saveEdit}>Save changes</button>
        </div>
      </footer>
    </section>
  {/if}

  <!-- ─── HRT timeline (only when set; editing happens in the metadata form) ─── -->
  {#if patient?.hrt_start_iso}
    <HrtSection
      patientId={patient.id}
      hrtStartIso={patient.hrt_start_iso}
      reports={reports}
      readOnly
    />
  {/if}

  <!-- ─── Notes card (always shown when patient loaded) ─── -->
  {#if patient}
    <section class="card p-4 space-y-2">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-sm font-semibold">Clinical notes</h2>
        {#if !editingNotes}
          <button class="text-xs text-accent hover:underline" onclick={startEditNotes}>
            {patient.notes ? 'Edit' : 'Add'}
          </button>
        {/if}
      </div>

      {#if editingNotes}
        <textarea
          class="block w-full bg-bg1 border border-line rounded-md px-3 py-2 text-sm font-mono"
          rows="8"
          placeholder="Free-form clinical context: allergies, family hx, treatment plan, ongoing conditions…"
          bind:value={notesDraft}
        ></textarea>
        <div class="flex justify-end gap-2 pt-1">
          <button class="btn text-xs" onclick={cancelEditNotes}>Cancel</button>
          <button class="btn-accent text-xs" disabled={savingNotes} onclick={saveNotes}>
            {savingNotes ? 'Saving…' : 'Save notes'}
          </button>
        </div>
      {:else if patient.notes}
        <pre class="text-sm whitespace-pre-wrap font-sans text-fg1 leading-relaxed">{patient.notes}</pre>
      {:else}
        <p class="text-xs text-fg3 italic">No notes yet — click <em>Add</em> to capture allergies, family history, ongoing conditions, etc.</p>
      {/if}
    </section>
  {/if}

  {#if err}<div class="card p-3 text-sm text-crit">{err.message}</div>{/if}

  <!-- ─── Patient dashboard ─── -->
  {#if patient}
    <section class="patient-dashboard-grid">
      <!-- Reports column -->
      <section class="dashboard-column">
        <header class="dashboard-column__header">
          <div>
            <h2 class="text-sm font-semibold">Reports <span class="text-fg3">({filteredReports.length}/{reports.length})</span></h2>
            <p class="dashboard-column__hint">Recent source documents and review status.</p>
          </div>
          <input class="input dashboard-filter" type="search" placeholder="Filter reports…" aria-label="Filter patient reports" bind:value={reportFilter} />
        </header>

        {#if reports.length === 0 && !err}
          <div class="dashboard-empty">No reports yet for this patient.</div>
        {:else if filteredReports.length === 0}
          <div class="dashboard-empty">No reports match “{reportFilter}”.</div>
        {:else}
          <div class="card dashboard-list divide-y divide-line">
        {#each filteredReports as r}
          {@const milestone = hrtMilestoneFor(r.collection_date_iso, patient?.hrt_start_iso ?? null)}
          <!-- reports are newest-first → the "previous" (in time) report is
               the one one slot DOWN in the array. -->
          {@const reportIndex = reports.findIndex((report) => report.id === r.id)}
          {@const prevReport = reports[reportIndex + 1]}
          {@const sinceLast = prevReport
            ? formatRelativeSpan(prevReport.collection_date_iso, r.collection_date_iso)
            : null}
          <div class="report-row group flex items-center justify-between gap-3 px-3 py-2 hover:bg-bg3/50
               {editingReportId === r.id ? 'report-row--editing' : ''}">
            {#if editingReportId === r.id}
              <!-- Inline rename — soft accent backdrop + framed input that
                   visually pops out from the row, signalling "edit mode"
                   without the user losing the row's identity. -->
              <div class="rename-shell">
                <div class="rename-shell__head">
                  <span class="rename-shell__pill" aria-hidden="true">
                    <svg viewBox="0 0 20 20" width="11" height="11" fill="none" stroke="currentColor"
                         stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
                      <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
                      <path d="M11 6 L14 9"/>
                    </svg>
                    Rename
                  </span>
                  <span class="rename-shell__date">{formatDate(r.collection_date_iso)}</span>
                  <span class="rename-shell__meta">
                    {r.row_count} rows · tier {r.ingest_tier} · conf {(r.doc_confidence * 100).toFixed(0)}%
                  </span>
                </div>

                <div class="rename-shell__row">
                  <!-- svelte-ignore a11y_autofocus -->
                  <input
                    class="rename-shell__input"
                    placeholder="e.g. Annual checkup, Pre-surgery panel…"
                    bind:value={reportNickDraft}
                    autofocus
                    onkeydown={(e) => {
                      if (e.key === 'Enter') { e.preventDefault(); saveReportNickname(); }
                      else if (e.key === 'Escape') { e.preventDefault(); cancelEditReportNickname(); }
                    }}
                  />
                  <button class="rename-shell__save" onclick={saveReportNickname}
                          title="Save (⏎)">
                    <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                         stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                      <path d="M4 10 L8 14 L16 6"/>
                    </svg>
                    Save
                  </button>
                  <button class="rename-shell__cancel" onclick={cancelEditReportNickname}
                          title="Cancel (Esc)">
                    <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                         stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                      <path d="M5 5 L15 15 M15 5 L5 15"/>
                    </svg>
                  </button>
                </div>

                <span class="rename-shell__hint">
                  ⏎ save · ⎋ cancel · empty input clears the nickname
                </span>
              </div>
            {:else}
              <a class="flex flex-col min-w-0 flex-1" href={`/report/${r.id}`}>
                <span class="text-sm font-medium flex items-center gap-2 flex-wrap">
                  <span>{r.nickname ?? formatDate(r.collection_date_iso)}</span>
                  <!-- Pencil icon — appears on hover. Click stops the link
                       navigation and flips the row into edit mode. -->
                  <button type="button"
                          class="row-edit-pencil"
                          title={r.nickname ? 'Rename report' : 'Add a nickname for this report'}
                          aria-label="Rename report"
                          onclick={(e) => { e.preventDefault(); e.stopPropagation(); startEditReportNickname(r); }}>
                    <svg viewBox="0 0 20 20" width="12" height="12" fill="none" stroke="currentColor"
                         stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                      <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
                      <path d="M11 6 L14 9"/>
                    </svg>
                  </button>
                  {#if milestone}
                    <span class="hrt-tag {milestone.isPre ? 'hrt-tag--pre' : ''}" title={milestone.long}>
                      {milestone.label}
                    </span>
                  {/if}
                </span>
                <span class="text-xs text-fg3 flex items-center gap-1.5 flex-wrap">
                  {#if r.nickname}{formatDate(r.collection_date_iso)} · {/if}
                  {r.row_count} rows · tier {r.ingest_tier} · conf {(r.doc_confidence * 100).toFixed(0)}%
                  {#if r.annotations}
                    <span class="text-fg2 inline-flex items-center gap-1">· <Icon name="edit" size={11} /> annotated</span>
                  {/if}
                  {#if sinceLast}
                    <span class="span-pill" title="Time since the previous report ({formatDate(prevReport.collection_date_iso)})">
                      {sinceLast}
                    </span>
                  {/if}
                </span>
              </a>
            {/if}
            <button
              type="button"
              class="row-icon row-icon--danger"
              onclick={() => onDeleteReport(r)}
              title="Delete report"
              aria-label="Delete report"
            >
              <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M5 6 H15 M8 6 V4 H12 V6 M6 6 L7 16 a1 1 0 0 0 1 1 H12 a1 1 0 0 0 1 -1 L14 6"/>
                <path d="M9 9 V14 M11 9 V14"/>
              </svg>
            </button>
          </div>
        {/each}
      </div>
        {/if}
      </section>

      <!-- Signals column -->
      <section class="dashboard-column">
        <header class="dashboard-column__header">
          <div>
            <h2 class="text-sm font-semibold">Signals <span class="text-fg3">({flaggedAnalytes.length})</span></h2>
            <p class="dashboard-column__hint">Review-worthy analytes from the latest history.</p>
          </div>
        </header>

        <div class="dashboard-segmented" role="tablist" aria-label="Filter analyte signals">
          {#each [
            { id: 'all', label: 'All' },
            { id: 'abnormal', label: 'Abnormal' },
            { id: 'elevated', label: 'Elevated' },
            { id: 'subclinical', label: 'Subclinical' },
          ] as option}
            <button
              type="button"
              role="tab"
              aria-selected={analyteCategory === option.id}
              class="dashboard-segmented__option {analyteCategory === option.id ? 'dashboard-segmented__option--active' : ''}"
              onclick={() => (analyteCategory = option.id as 'all' | AnalyteCategory)}
            >
              {option.label} <span class="tabular-nums">{categoryCount(option.id as 'all' | AnalyteCategory)}</span>
            </button>
          {/each}
        </div>
        <input class="input w-full mb-2" type="search" placeholder="Search analytes…" aria-label="Search flagged analytes" bind:value={analyteFilter} />

        {#if dashboardLoading}
          <div class="dashboard-empty">Loading analyte signals…</div>
        {:else if flaggedAnalytes.length === 0}
          <div class="dashboard-empty">No analytes match this filter.</div>
        {:else}
          <div class="dashboard-list space-y-1">
            {#each flaggedAnalytes as analyte (analyte.id)}
              <a class="signal-row" href={`/analyte/${analyte.id}?patient=${patientId}`}>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1.5 min-w-0">
                    <span class="truncate text-xs font-medium">{analyte.name}</span>
                    <span class="signal-category signal-category--{analyte.category}">{categoryLabel(analyte.category)}</span>
                  </div>
                  <div class="text-[10px] text-fg3 mt-0.5">
                    {readingValue(analyte.latest)} {analyte.latest.unit ?? ''} · {formatDate(analyte.latest.date)}
                  </div>
                </div>
                <FlagPill flag={analyte.latest.flag} />
              </a>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Trends column -->
      <section class="dashboard-column">
        <header class="dashboard-column__header">
          <div>
            <h2 class="text-sm font-semibold">Trends</h2>
            <p class="dashboard-column__hint">Most measured analytes and largest changes.</p>
          </div>
          <label class="trend-window">
            <span>Window</span>
            <select
              class="select trend-window__select"
              value={trendWindowDays}
              aria-label="Trend time window"
              onchange={(event) => setTrendWindow(Number((event.currentTarget as HTMLSelectElement).value))}
            >
              {#each trendWindows as window}
                <option value={window.id}>{window.label}</option>
              {/each}
            </select>
          </label>
        </header>

        <div class="trend-group">
          <h3 class="trend-group__title">Main analytes</h3>
          {#if mainAnalytes.length === 0}
            <div class="dashboard-empty">No numeric history yet.</div>
          {:else}
            <div class="space-y-1">
              {#each mainAnalytes as analyte (analyte.id)}
                <a class="trend-row" href={`/analyte/${analyte.id}?patient=${patientId}`}>
                  <div class="flex items-baseline justify-between gap-2 min-w-0">
                    <span class="truncate text-xs font-medium">{analyte.name}</span>
                    <span class="shrink-0 text-[10px] text-fg2 tabular-nums">{readingValue(analyte.latest)} {analyte.latest.unit ?? ''}</span>
                  </div>
                  <svg class="sparkline" viewBox="0 0 100 28" preserveAspectRatio="none" aria-label={`${analyte.name} trend`} role="img">
                    <polyline points={sparklinePoints(analyte.readings)} fill="none" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke" />
                  </svg>
                </a>
              {/each}
            </div>
          {/if}
        </div>

        <div class="trend-group">
          <h3 class="trend-group__title">Biggest deltas</h3>
          {#if biggestDeltas.length === 0}
            <div class="dashboard-empty">Need at least two numeric readings.</div>
          {:else}
            <div class="space-y-1">
              {#each biggestDeltas as analyte (analyte.id)}
                <a class="trend-row" href={`/analyte/${analyte.id}?patient=${patientId}`}>
                  <div class="flex items-baseline justify-between gap-2 min-w-0">
                    <span class="truncate text-xs font-medium">{analyte.name}</span>
                    {#if analyte.delta}
                      <span class="shrink-0 text-[10px] tabular-nums {analyte.delta.dir === 'up' ? 'text-crit' : analyte.delta.dir === 'down' ? 'text-accent' : 'text-fg3'}">
                        {analyte.delta.abs > 0 ? '+' : ''}{formatNumber(analyte.delta.abs)} ({analyte.delta.pct > 0 ? '+' : ''}{analyte.delta.pct.toFixed(0)}%)
                      </span>
                    {/if}
                  </div>
                  <svg class="sparkline sparkline--delta" viewBox="0 0 100 28" preserveAspectRatio="none" aria-label={`${analyte.name} delta trend`} role="img">
                    <polyline points={sparklinePoints(analyte.readings)} fill="none" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke" />
                  </svg>
                </a>
              {/each}
            </div>
          {/if}
        </div>
      </section>
    </section>
  {/if}
</div>

<style>
  .patient-dashboard-grid {
    display: grid;
    grid-template-columns: minmax(0, 1.1fr) minmax(0, 1fr) minmax(0, 1fr);
    gap: 0.75rem;
    align-items: start;
  }
  .dashboard-column {
    min-width: 0;
    min-height: 22rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    padding: 0.85rem;
    border: 1px solid rgb(var(--line));
    border-radius: 0.6rem;
    background: rgb(var(--bg-1));
  }
  .dashboard-column__header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.65rem;
  }
  .dashboard-column__hint {
    margin-top: 0.15rem;
    color: rgb(var(--fg-3));
    font-size: 0.65rem;
    line-height: 1.3;
  }
  .trend-window {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    flex: 0 0 auto;
    color: rgb(var(--fg-3));
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .trend-window__select {
    min-width: 5.8rem;
    padding: 0.25rem 1.45rem 0.25rem 0.4rem;
    color: rgb(var(--fg-1));
    font-size: 0.65rem;
    text-transform: none;
    letter-spacing: normal;
  }
  .dashboard-filter {
    width: 8.5rem;
    min-width: 0;
    font-size: 0.7rem;
  }
  .dashboard-list {
    min-height: 0;
    max-height: 34rem;
    overflow-y: auto;
  }
  .dashboard-empty {
    padding: 1.5rem 0.5rem;
    color: rgb(var(--fg-3));
    font-size: 0.7rem;
    text-align: center;
  }
  .dashboard-segmented {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
  }
  .dashboard-segmented__option {
    flex: 1 1 auto;
    padding: 0.3rem 0.45rem;
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    color: rgb(var(--fg-2));
    background: rgb(var(--bg-2));
    font-size: 0.65rem;
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
  }
  .dashboard-segmented__option:hover,
  .dashboard-segmented__option--active {
    border-color: rgb(var(--accent) / 0.55);
    color: rgb(var(--accent));
    background: rgb(var(--accent) / 0.12);
  }
  .signal-row,
  .trend-row {
    display: block;
    min-width: 0;
    padding: 0.45rem;
    border: 1px solid transparent;
    border-radius: 0.4rem;
    color: rgb(var(--fg-1));
    transition: background 120ms ease, border-color 120ms ease, transform 120ms ease;
  }
  .signal-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .signal-row:hover,
  .trend-row:hover {
    border-color: rgb(var(--accent) / 0.35);
    background: rgb(var(--bg-2));
    transform: translateX(2px);
  }
  .signal-category {
    flex: 0 0 auto;
    padding: 0.08rem 0.3rem;
    border-radius: 9999px;
    font-size: 0.55rem;
    font-weight: 600;
    line-height: 1.2;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .signal-category--abnormal {
    color: rgb(var(--warn));
    background: rgb(var(--warn) / 0.12);
  }
  .signal-category--elevated {
    color: rgb(var(--crit));
    background: rgb(var(--crit) / 0.12);
  }
  .signal-category--subclinical {
    color: rgb(var(--accent));
    background: rgb(var(--accent) / 0.12);
  }
  .trend-group {
    padding-top: 0.2rem;
  }
  .trend-group + .trend-group {
    margin-top: 0.35rem;
    padding-top: 0.65rem;
    border-top: 1px dashed rgb(var(--line));
  }
  .trend-group__title {
    margin-bottom: 0.3rem;
    color: rgb(var(--fg-2));
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .sparkline {
    display: block;
    width: 100%;
    height: 1.65rem;
    margin-top: 0.25rem;
    color: rgb(var(--accent));
    opacity: 0.9;
  }
  .sparkline--delta { color: rgb(var(--warn)); }
  @media (max-width: 1050px) {
    .patient-dashboard-grid { grid-template-columns: 1fr 1fr; }
    .dashboard-column:last-child { grid-column: 1 / -1; }
  }
  @media (max-width: 680px) {
    .patient-dashboard-grid { grid-template-columns: 1fr; }
    .dashboard-column:last-child { grid-column: auto; }
    .dashboard-column__header { flex-direction: column; }
    .dashboard-filter { width: 100%; }
  }

  /* ── Edit-metadata card ───────────────────────────────────────────────
     Mirrors the inline panel on the Patients list so the editing UX is
     consistent across both surfaces. */
  .edit-card {
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-left: 3px solid rgb(var(--accent));
    border-radius: 0.6rem;
    padding: 1rem 1.1rem 0.9rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }
  .edit-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }
  .edit-card__title {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--accent));
  }
  .edit-card__pencil {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    border-radius: 9999px;
    background: rgb(var(--accent) / 0.15);
    color: rgb(var(--accent));
  }
  .edit-card__id {
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.7rem;
    color: rgb(var(--fg-3));
  }
  .edit-card__grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.85rem 1.25rem;
  }
  @media (min-width: 720px) {
    .edit-card__grid { grid-template-columns: 1fr 1fr; }
  }
  .edit-card__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding-top: 0.55rem;
    border-top: 1px dashed rgb(var(--line));
  }
  .edit-card__hotkeys {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  /* Field primitives — same names as on the Patients list, scoped to this
     page so each component stays self-contained. */
  :global(.edit-field) { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; }
  :global(.edit-field__label) {
    font-size: 0.7rem; font-weight: 500; color: rgb(var(--fg-2));
  }
  :global(.edit-field__opt) {
    color: rgb(var(--fg-3)); font-weight: 400; font-style: italic;
  }
  :global(.edit-field__input) {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    padding: 0.45rem 0.65rem;
    font-size: 0.9rem;
    color: rgb(var(--fg-1));
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  :global(.edit-field__input:hover) { border-color: rgb(var(--fg-3)); }
  :global(.edit-field__input:focus) {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18);
  }
  :global(.edit-field__input--narrow) { max-width: 11rem; }
  :global(.edit-field__hint) {
    font-size: 0.65rem; color: rgb(var(--fg-3)); line-height: 1.3;
  }
  :global(.edit-segmented) {
    display: inline-flex;
    flex-wrap: wrap;
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
    width: max-content;
  }
  :global(.edit-segmented__opt) {
    padding: 0.4rem 0.7rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 0;
    border-right: 1px solid rgb(var(--line));
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }
  :global(.edit-segmented__opt:last-child) { border-right: 0; }
  :global(.edit-segmented__opt:hover) { background: rgb(var(--bg-2)); color: rgb(var(--fg-1)); }
  :global(.edit-segmented__opt--on) {
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-weight: 600;
  }

  /* Read-only sex badge in the patient meta line. Doubles as a button —
     clicking opens the metadata edit form. Same colour vocabulary as the
     overview list so the cross-page UX feels stitched together. */
  .sex-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.5rem;
    border-radius: 9999px;
    font-size: 0.7rem;
    font-weight: 500;
    border: 1px solid;
    cursor: pointer;
    background: transparent;
    transition: background 120ms ease, border-color 120ms ease;
  }
  .sex-pill:hover { background: rgb(var(--bg-2)); }
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
  .sex-pill--\? {
    color: rgb(var(--fg-3));
    background: rgb(var(--bg-3));
    border-color: rgb(var(--line));
    border-style: dashed;
  }

  /* ── Inline report-rename shell ─────────────────────────────────────
     A focused, "modal-feeling" inline editor that replaces the row body
     when the user clicks the pencil. Soft accent backdrop + 3px left
     rail keeps it grounded inside the row while visually distinct. */
  .report-row--editing {
    background: rgb(var(--accent) / 0.06);
    border-left: 3px solid rgb(var(--accent));
    padding-left: calc(0.75rem - 3px); /* compensate so content doesn't shift */
  }
  .rename-shell {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }
  .rename-shell__head {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .rename-shell__pill {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.1rem 0.55rem;
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-radius: 9999px;
  }
  .rename-shell__date {
    font-size: 0.7rem;
    font-weight: 600;
    color: rgb(var(--fg-1));
    font-variant-numeric: tabular-nums;
  }
  .rename-shell__meta {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
  }
  .rename-shell__row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .rename-shell__input {
    flex: 1 1 auto;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--accent) / 0.4);
    border-radius: 0.45rem;
    padding: 0.4rem 0.7rem;
    font-size: 0.9rem;
    color: rgb(var(--fg-1));
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  .rename-shell__input:focus {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 3px rgb(var(--accent) / 0.22);
  }
  .rename-shell__save,
  .rename-shell__cancel {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.4rem 0.7rem;
    border-radius: 0.45rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid;
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
  }
  .rename-shell__save {
    color: white;
    background: rgb(var(--accent));
    border-color: rgb(var(--accent));
  }
  .rename-shell__save:hover {
    background: rgb(var(--accent) / 0.85);
  }
  .rename-shell__cancel {
    color: rgb(var(--fg-2));
    background: transparent;
    border-color: rgb(var(--line));
    width: 2.1rem;
    justify-content: center;
    padding: 0.4rem;
  }
  .rename-shell__cancel:hover {
    color: rgb(var(--crit));
    border-color: rgb(var(--crit) / 0.4);
    background: rgb(var(--crit) / 0.08);
  }
  .rename-shell__hint {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  }

  /* Pencil affordance next to a report's title; muted by default and
     fades up to accent on hover or when the row itself is hovered, so
     users discover the inline-rename without crowding the resting state. */
  .row-edit-pencil {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    border: 0;
    background: transparent;
    color: rgb(var(--fg-3));
    border-radius: 0.25rem;
    cursor: pointer;
    opacity: 0;
    transition: opacity 120ms ease, background 120ms ease, color 120ms ease;
  }
  /* Show the pencil when the row is hovered or the pencil itself has focus —
     keeps the resting state clean but discoverable. */
  .report-row:hover .row-edit-pencil,
  .row-edit-pencil:focus-visible {
    opacity: 1;
  }
  .row-edit-pencil:hover {
    background: rgb(var(--accent) / 0.12);
    color: rgb(var(--accent));
  }
  .row-edit-pencil:focus-visible {
    outline: 2px solid rgb(var(--accent));
    outline-offset: 1px;
  }

  /* HRT-milestone tag attached to each report in the patient's report list.
     Compact pill — accent for post-start milestones, warn for pre-start
     baseline draws so the visual contrast matches HrtSection's chips. */
  .hrt-tag {
    display: inline-flex;
    align-items: center;
    padding: 0.05rem 0.4rem;
    border-radius: 9999px;
    font-size: 0.65rem;
    font-weight: 600;
    color: rgb(var(--accent));
    background: rgb(var(--accent) / 0.12);
    border: 1px solid rgb(var(--accent) / 0.35);
    letter-spacing: 0.02em;
  }
  .hrt-tag--pre {
    color: rgb(var(--warn));
    background: rgb(var(--warn) / 0.12);
    border-color: rgb(var(--warn) / 0.35);
  }

  /* Same icon-button visual language as the Patients list — kept page-local
     here so the module CSS guard keeps each page from leaking styles to
     siblings. The `:global` makes the rule reachable from the template
     above (Svelte component-scoped CSS would otherwise add a hash). */
  :global(.row-icon) {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.6rem;
    height: 1.6rem;
    border: 1px solid transparent;
    border-radius: 0.35rem;
    color: rgb(var(--fg-3));
    background: transparent;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
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
