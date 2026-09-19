<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { ask } from '@tauri-apps/plugin-dialog';
  import * as reportApi from '$api/report-detail';
  import * as admin from '$api/records-admin';
  import * as reparse from '$api/reparse';
  import { AppError } from '$api/errors';
  import { toasts } from '../../../lib/toasts/store.svelte';
  import FlagPill from '$charts/flag-pill.svelte';
  import { chartPrefs } from '$charts/prefs.svelte';
  import AnalyteLinkDialog from '$components/analyte-link-dialog.svelte';
  import { formatDate } from '$format/dates';
  import { formatNumber } from '$format/numbers';
  import { prettyUnit } from '$format/units';
  import { parseTiers, matchTier, tierToFlag, formatTierRange } from '$format/tiers';
  import { defaultRefFor, formatDefaultRef, flagForDefaultRef } from '$format/default-ref';
  import { phaseRefFor, flagForPhaseRef } from '$format/cycle-phase';
  import { openFileExternal } from '$api/shell';
  import { exportReportRowsCsv } from '$api/export';
  import { saveTextFile } from '$format/save';
  import { setReportCyclePhase, setReportNickname, setReportAnnotations, type CyclePhase } from '$api/records-admin';
  import { windowTitle, setPageTitle } from '$lib/title.svelte';
  import BackButton from '$components/back-button.svelte';
  import { hrtMilestoneFor } from '$format/hrt-milestone';
  import { formatRelativeSpan } from '$format/dates';
  import Icon from '$components/icon.svelte';
  import { patientAnalyteSummaries, type AnalyteReading, type PatientAnalyteSummary } from '$api/reports';
  import { t } from '$lib/i18n/index.svelte';
  import '$lib/i18n/data-routes';

  // Time gap to the previous (older) report for this patient — surfaces in
  // the page header so users immediately see the cadence between draws.
  let prevReportDateIso = $state<string | null>(null);
  $effect(() => {
    if (!detail?.prev_report_id) { prevReportDateIso = null; return; }
    // We already have prev_report_id; the cheapest way to learn its date is
    // to query the backend's `list_reports` for this patient and look up
    // the matching id. Refresh on the report's own load.
    (async () => {
      try {
        const { listReports } = await import('$api/reports');
        const all = await listReports(detail!.report.patient_id);
        const prev = all.find((r) => r.id === detail!.prev_report_id);
        prevReportDateIso = prev?.collection_date_iso ?? null;
      } catch { prevReportDateIso = null; }
    })();
  });

  let reportId = $derived($page.params.id ?? '');
  let detail = $state<reportApi.ReportDetail | null>(null);
  let analyteSeries = $state<PatientAnalyteSummary[]>([]);
  let err = $state<AppError | null>(null);
  let loading = $state(true);
  let filter = $state<'all' | 'matched' | 'unmatched' | 'abnormal' | 'inline_prior'>('all');
  let showPriors = $state(false);

  let linkDialogOpen = $state(false);
  let linkRawText = $state('');

  async function refresh() {
    loading = true;
    err = null;
    try {
      const nextDetail = await reportApi.get(reportId);
      detail = nextDetail;
      try {
        analyteSeries = await patientAnalyteSummaries(nextDetail.report.patient_id);
      } catch {
        // The report remains usable if longitudinal trend data is unavailable.
        analyteSeries = [];
      }
    } catch (e) {
      analyteSeries = [];
      err = AppError.fromUnknown(e);
      toasts.error(err);
    } finally {
      loading = false;
    }
  }

  $effect(() => { if (reportId) refresh(); });

  // Reflect the open report in the OS window title — friendly for taskbar
  // switching when several reports are open across separate sessions.
  $effect(() => {
    if (!detail) {
      setPageTitle(t('Report'));
      return;
    }
    const r = detail.report;
    const lead = r.nickname ?? r.collection_date_iso;
    setPageTitle(`${lead} · ${r.patient_name}`);
  });

  // Keyboard shortcuts for adjacent-report navigation. Skip while focus is in
  // a text field so typing in the link dialog still works.
  function onKeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
    if (linkDialogOpen) return;
    if (e.key === 'ArrowLeft' && detail?.prev_report_id) {
      e.preventDefault();
      goto(`/report/${detail.prev_report_id}`);
    } else if (e.key === 'ArrowRight' && detail?.next_report_id) {
      e.preventDefault();
      goto(`/report/${detail.next_report_id}`);
    }
  }

  function openLinkDialog(rawText: string) {
    linkRawText = rawText;
    linkDialogOpen = true;
  }

  async function onDeleteReport() {
    if (!detail) return;
    const ok = await ask(
      t('Delete report {date} for {name}?\n\nThis removes the report, all {rows} parsed rows, and the cached PDF. This cannot be undone.', { date: formatDate(detail.report.collection_date_iso), name: detail.report.patient_name, rows: detail.stats.total_rows }),
      { title: t('Delete report'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deleteReport(reportId);
      toasts.success(t('Report deleted'));
      await goto('/records');
    } catch (e) { toasts.error(e); }
  }

  async function onDeleteRow(rowId: number) {
    const ok = await ask(t('Delete this row?'), { title: t('Delete row'), kind: 'warning' });
    if (!ok) return;
    try {
      await admin.deleteResult(rowId);
      toasts.success(t('Row deleted'));
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  async function onOpenPdf() {
    if (!detail) return;
    try {
      await openFileExternal(detail.report.raw_pdf_path);
    } catch (e) { toasts.error(e); }
  }

  async function onExportCsv() {
    if (!detail) return;
    try {
      const out = await exportReportRowsCsv(reportId);
      const path = await saveTextFile(out.content, { defaultPath: out.filename });
      if (path) toasts.success(t('Exported'), path);
    } catch (e) { toasts.error(e); }
  }

  function diagnosticLabel(code: string): string {
    switch (code) {
      case 'analyte_unmatched':          return t('Unmatched analyte');
      case 'reference_range_unparsed':   return t('Unparsed range');
      case 'unit_unrecognized':          return t('Unit mismatch');
      case 'value_missing':              return t('Missing value');
      case 'low_confidence':             return t('Low confidence');
      default:                           return code.replaceAll('_', ' ');
    }
  }

  function diagnosticSummary(items: reportApi.ReportParseAudit[]): string {
    const counts = new Map<string, number>();
    for (const item of items) counts.set(item.diagnostic, (counts.get(item.diagnostic) ?? 0) + 1);
    return [...counts.entries()]
      .map(([code, count]) => `${diagnosticLabel(code)} ${count}`)
      .join(' · ');
  }

  let editingNickname = $state(false);
  let nicknameDraft = $state('');

  function startEditNickname() {
    if (!detail) return;
    nicknameDraft = detail.report.nickname ?? '';
    editingNickname = true;
  }

  async function saveNickname() {
    if (!detail) return;
    const next = nicknameDraft.trim();
    try {
      await setReportNickname(reportId, next.length === 0 ? null : next);
      detail.report.nickname = next.length === 0 ? null : next;
      editingNickname = false;
      toasts.success(t('Nickname saved'), next || t('cleared'));
    } catch (e) { toasts.error(e); }
  }

  function cancelEditNickname() {
    editingNickname = false;
  }

  // ── Annotations (per-report free-form notes) ──────────────────────────
  let editingAnnotations = $state(false);
  let annotationsDraft = $state('');
  let savingAnnotations = $state(false);

  function startEditAnnotations() {
    if (!detail) return;
    annotationsDraft = detail.report.annotations ?? '';
    editingAnnotations = true;
  }
  async function saveAnnotations() {
    if (!detail) return;
    savingAnnotations = true;
    try {
      const next = annotationsDraft.trim() ? annotationsDraft : null;
      await setReportAnnotations(reportId, next);
      detail.report.annotations = next;
      editingAnnotations = false;
      toasts.success(t('Annotations saved'));
    } catch (e) { toasts.error(e); }
    finally { savingAnnotations = false; }
  }
  function cancelEditAnnotations() {
    editingAnnotations = false;
  }

  async function onCyclePhaseChange(e: Event) {
    if (!detail) return;
    const v = (e.target as HTMLSelectElement).value;
    const phase: CyclePhase = v === '' ? null : (v as CyclePhase);
    try {
      await setReportCyclePhase(reportId, phase);
      detail.report.cycle_phase = phase;
      toasts.success(t('Cycle phase updated'), phase ?? t('cleared'));
    } catch (e) { toasts.error(e); }
  }

  let reparsing = $state(false);
  async function onReparse() {
    reparsing = true;
    try {
      const r = await reparse.reparseReport(reportId);
      const delta = r.rows_after - r.rows_before;
      const sign = delta >= 0 ? '+' : '';
      toasts.success(
        t('Re-parsed'),
        t('{rows} rows ({delta} vs before), {unmatched} unmatched, {diagnostics} diagnostics, conf {confidence}%', { rows: r.rows_after, delta: `${sign}${delta}`, unmatched: r.rows_unmatched, diagnostics: r.parse_audit_entries, confidence: (r.doc_confidence * 100).toFixed(0) })
      );
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { reparsing = false; }
  }

  const filteredRows = $derived.by(() => {
    if (!detail) return [];
    return detail.rows.filter((r) => {
      // Hide inline-prior-pdf rows by default — they're values from this
      // report's "Resultados anteriores" columns reaching back into history.
      // The user can flip the toggle to see them. The dedicated 'inline_prior'
      // filter pin overrides this.
      if (!showPriors && filter !== 'inline_prior' && r.inline_prior_pdf) return false;
      switch (filter) {
        case 'matched':       return r.analyte_id != null;
        case 'unmatched':     return r.analyte_id == null;
        case 'abnormal':      return r.flag === 'low' || r.flag === 'high' || r.flag === 'critical_low' || r.flag === 'critical_high' || r.flag === 'abnormal_qual';
        case 'inline_prior':  return r.inline_prior_pdf;
        default:              return true;
      }
    });
  });

  function trendReadings(row: reportApi.ReportRow): AnalyteReading[] {
    if (!row.analyte_id) return [];
    return analyteSeries.find((summary) => summary.analyte_id === row.analyte_id)?.readings
      .filter((reading) => reading.value != null && !reading.inline_prior) ?? [];
  }

  function sparklinePoints(readings: AnalyteReading[]): string {
    if (readings.length < 2) return '';
    const values = readings.map((reading) => reading.value as number);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const span = max - min || 1;
    return values.map((value, index) => {
      const x = (index / (values.length - 1)) * 100;
      const y = 24 - ((value - min) / span) * 18;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    }).join(' ');
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="space-y-4">
  <div class="flex items-end justify-between gap-3">
    <div>
      <BackButton fallback="/records" />
      <h1 class="text-xl font-semibold mt-1 flex items-baseline gap-2 flex-wrap">
        {#if detail}
          {#if detail.report.nickname && !editingNickname}
            <span title={detail.report.nickname}>{detail.report.nickname}</span>
            <span class="text-fg3 font-normal text-sm">·</span>
            <a class="text-accent hover:underline text-sm font-normal" href={`/patient/${detail.report.patient_id}`}>{detail.report.patient_name}</a>
            <span class="text-fg2 font-normal text-sm">· {formatDate(detail.report.collection_date_iso)}</span>
          {:else if !editingNickname}
            {t('Report')} ·
            <a class="text-accent hover:underline text-base" href={`/patient/${detail.report.patient_id}`}>{detail.report.patient_name}</a>
            <span class="text-fg2 font-normal text-sm">· {formatDate(detail.report.collection_date_iso)}</span>
          {/if}

          {#if editingNickname}
            <!-- svelte-ignore a11y_autofocus — explicit focus on entering edit mode is the desired UX -->
            <input
              class="input text-base flex-1 min-w-[12rem]"
              placeholder={t('e.g. Annual checkup, Pre-surgery panel…')}
              bind:value={nicknameDraft}
              autofocus
              onkeydown={(e) => {
                if (e.key === 'Enter') { e.preventDefault(); saveNickname(); }
                else if (e.key === 'Escape') { e.preventDefault(); cancelEditNickname(); }
              }}
            />
            <button class="btn text-xs" onclick={saveNickname}>{t('Save')}</button>
            <button class="btn text-xs" onclick={cancelEditNickname}>{t('Cancel')}</button>
          {:else}
            <button
              class="text-xs text-fg3 hover:text-accent ml-1"
              onclick={startEditNickname}
              title={t(detail.report.nickname ? 'Edit nickname' : 'Add a friendly nickname for this report')}
            ><Icon name={detail.report.nickname ? 'edit' : 'plus'} size={13} /> {t(detail.report.nickname ? 'rename' : 'nickname')}</button>
          {/if}
        {:else}
          {t('Report')} {reportId}
        {/if}
      </h1>
      {#if detail}
        {@const milestone = hrtMilestoneFor(detail.report.collection_date_iso, detail.report.hrt_start_iso)}
        {@const sincePrev = prevReportDateIso
          ? formatRelativeSpan(prevReportDateIso, detail.report.collection_date_iso)
          : null}
        {#if milestone || sincePrev}
          <p class="text-xs mt-1 flex items-center gap-2 flex-wrap">
            {#if milestone}
              <span class="hrt-tag {milestone.isPre ? 'hrt-tag--pre' : ''}" title={milestone.long}>
                {milestone.label} HRT
              </span>
              <span class="text-fg3">· {milestone.long}</span>
            {/if}
            {#if sincePrev}
              <span class="span-pill" title={t('Time since the previous report ({date})', { date: prevReportDateIso })}>
                {sincePrev} {t('since previous')}
              </span>
            {/if}
          </p>
        {/if}
      {/if}
    </div>
    {#if detail}
      <div class="flex items-center gap-2">
        <button
          class="btn"
          disabled={!detail.prev_report_id}
          title={t(detail.prev_report_id ? 'Previous report (older) for this patient' : 'No earlier report for this patient')}
          onclick={() => detail!.prev_report_id && goto(`/report/${detail!.prev_report_id}`)}
        ><Icon name="arrow-left" size={14} /> {t('Prev')}</button>
        <button
          class="btn"
          disabled={!detail.next_report_id}
          title={t(detail.next_report_id ? 'Next report (newer) for this patient' : 'No later report for this patient')}
          onclick={() => detail!.next_report_id && goto(`/report/${detail!.next_report_id}`)}
        >{t('Next')} <Icon name="arrow-right" size={14} /></button>
        <button class="btn" onclick={onOpenPdf} title={t('Open the original PDF in your default viewer')}>{t('Open PDF')}</button>
        <button class="btn" onclick={onExportCsv} title={t('Download every parsed row as CSV')}>{t('Export CSV')}</button>
        <button class="btn" disabled={reparsing} onclick={onReparse}>
          {t(reparsing ? 'Re-parsing…' : 'Re-parse')}
        </button>
        <button class="btn text-crit hover:bg-crit/10" onclick={onDeleteReport}>{t('Delete report')}</button>
      </div>
    {/if}
  </div>

  {#if err}
    <div class="card p-4 border-l-4 border-crit space-y-2">
      <div class="text-sm font-medium text-crit">{err.message}</div>
      <div class="text-[11px] text-fg3 font-mono">{err.code}</div>
    </div>
  {/if}

  {#if loading}
    <div class="card p-6 text-sm text-fg2">{t('Loading…')}</div>
  {:else if detail}
    <section class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Total rows')}</div>
        <div class="text-2xl font-semibold tabular-nums">{detail.stats.total_rows}</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Matched')}</div>
        <div class="text-2xl font-semibold tabular-nums text-ok">{detail.stats.matched_rows}</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Unmatched')}</div>
        <div class="text-2xl font-semibold tabular-nums {detail.stats.unmatched_rows > 0 ? 'text-warn' : ''}">{detail.stats.unmatched_rows}</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Abnormal')}</div>
        <div class="text-2xl font-semibold tabular-nums {detail.stats.abnormal_rows > 0 ? 'text-warn' : ''}">{detail.stats.abnormal_rows}</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Inline priors')}</div>
        <div class="text-2xl font-semibold tabular-nums">{detail.stats.inline_prior_rows}</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">{t('Avg confidence')}</div>
        <div class="text-2xl font-semibold tabular-nums">{(detail.stats.avg_confidence * 100).toFixed(0)}%</div>
        <div class="text-[10px] text-fg3">{t('min')} {(detail.stats.min_confidence * 100).toFixed(0)}%</div>
      </div>
    </section>

    <section class="card p-4 space-y-2">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-sm font-semibold">{t('Report metadata')}</h2>
        <span class="text-xs text-fg3 font-mono">{detail.report.id}</span>
      </div>
      <dl class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 text-xs">
        <dt class="text-fg2">{t('patient')}</dt>
        <dd>{detail.report.patient_name} ({detail.report.patient_sex})
          {#if detail.report.age_at_collection != null} · {detail.report.age_at_collection} y/o{/if}
        </dd>

        <dt class="text-fg2">{t('collection')}</dt>
        <dd>{formatDate(detail.report.collection_date_iso)}</dd>

        {#if detail.report.emission_date_iso}
          <dt class="text-fg2">{t('emission')}</dt>
          <dd>{formatDate(detail.report.emission_date_iso)}</dd>
        {/if}

        {#if detail.report.lab_entity}
          <dt class="text-fg2">{t('lab')}</dt>
          <dd>{detail.report.lab_entity}</dd>
        {/if}

        {#if detail.report.requesting_physician}
          <dt class="text-fg2">{t('requesting')}</dt>
          <dd>{detail.report.requesting_physician}</dd>
        {/if}

        {#if detail.report.inscription_id}
          <dt class="text-fg2">{t('inscription')}</dt>
          <dd class="font-mono">{detail.report.inscription_id}</dd>
        {/if}

        {#if detail.report.patient_sex === 'f'}
          <dt class="text-fg2">{t('cycle phase')}</dt>
          <dd>
            <select
              class="select text-xs"
              value={detail.report.cycle_phase ?? ''}
              onchange={onCyclePhaseChange}
              title={t('Used to pick the right reference range for cycle-dependent analytes (Estradiol, FSH, LH).')}
            >
              <option value="">{t('— unknown —')}</option>
              <option value="follicular">{t('Follicular (~days 1–13)')}</option>
              <option value="ovulation">{t('Ovulation (~day 14)')}</option>
              <option value="luteal">{t('Luteal (~days 15–28)')}</option>
              <option value="postmenopause">{t('Post-menopause')}</option>
            </select>
          </dd>
        {/if}

        <dt class="text-fg2">{t('tier')}</dt>
        <dd>{detail.report.ingest_tier} · parser {detail.report.parse_version}</dd>

        <dt class="text-fg2">{t('source')}</dt>
        <dd class="font-mono break-all">{detail.report.source_path}</dd>
      </dl>
    </section>

    <!-- ─── Annotations card ─── -->
    <section class="card p-4 space-y-2">
      <div class="flex items-baseline justify-between gap-3">
        <h2 class="text-sm font-semibold">{t('Annotations')}</h2>
        {#if !editingAnnotations}
          <button class="text-xs text-accent hover:underline" onclick={startEditAnnotations}>
            {t(detail.report.annotations ? 'Edit' : 'Add')}
          </button>
        {/if}
      </div>

      {#if editingAnnotations}
        <textarea
          class="block w-full bg-bg1 border border-line rounded-md px-3 py-2 text-sm font-mono"
          rows="6"
          placeholder={t("Context specific to this draw — e.g. 'fasting violated', 'first labs after starting estradiol valerate', 'redrawn after lab error'…")}
          bind:value={annotationsDraft}
        ></textarea>
        <div class="flex justify-end gap-2 pt-1">
          <button class="btn text-xs" onclick={cancelEditAnnotations}>{t('Cancel')}</button>
          <button class="btn-accent text-xs" disabled={savingAnnotations} onclick={saveAnnotations}>
            {t(savingAnnotations ? 'Saving…' : 'Save annotations')}
          </button>
        </div>
      {:else if detail.report.annotations}
        <pre class="text-sm whitespace-pre-wrap font-sans text-fg1 leading-relaxed">{detail.report.annotations}</pre>
      {:else}
        <p class="text-xs text-fg3 italic">
          {t("No annotations on this report — click Add to capture context that's specific to this draw (deviations from protocol, recent meds, milestone moments, etc.). Patient-level notes live on the patient page.")}
        </p>
      {/if}
    </section>

    {#if detail.parse_audit.length > 0}
      <section class="card p-4 space-y-2 border-l-4 border-warn">
        <div class="flex items-baseline justify-between gap-3">
          <h2 class="text-sm font-semibold text-warn">
            {t('{count} parse diagnostic(s)', { count: detail.parse_audit.length })}
          </h2>
          <span class="text-[10px] text-fg3 font-mono">{t('tier')} {detail.report.ingest_tier}</span>
        </div>
        <p class="text-xs text-fg2">{diagnosticSummary(detail.parse_audit)}</p>
        <ul class="grid grid-cols-1 md:grid-cols-2 gap-x-4 gap-y-1 text-xs">
          {#each detail.parse_audit.slice(0, 8) as item}
            <li class="flex items-center gap-2 min-w-0">
              <span class="font-mono text-fg3 shrink-0">{t('row')} {item.row_index + 1}</span>
              <span class="text-fg1 truncate" title={diagnosticLabel(item.diagnostic)}>
                {diagnosticLabel(item.diagnostic)}
              </span>
              {#if item.confidence != null}
                <span class="text-fg3 shrink-0">{(item.confidence * 100).toFixed(0)}%</span>
              {/if}
            </li>
          {/each}
        </ul>
        {#if detail.parse_audit.length > 8}
          <p class="text-[11px] text-fg3">{t('Showing first 8 diagnostics; filter unmatched rows or re-parse after Library fixes.')}</p>
        {/if}
      </section>
    {/if}

    {#if detail.unmatched_analytes.length > 0}
      <section class="card p-4 space-y-2 border-l-4 border-warn">
        <h2 class="text-sm font-semibold text-warn">
          {t('{count} unmatched analyte(s)', { count: detail.unmatched_analytes.length })}
        </h2>
        <p class="text-xs text-fg2">
          {t("The parser couldn't link these names to entries in the analyte Library. Click Link… to map a raw name to an existing analyte (creates a user alias and re-links every row that matches).")}
        </p>
        <ul class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-4 gap-y-1 text-xs">
          {#each detail.unmatched_analytes as name}
            <li class="flex items-center gap-2">
              <span class="font-mono text-fg2 truncate flex-1" title={name}>{name}</span>
              <button class="text-accent hover:underline text-xs" onclick={() => openLinkDialog(name)}>
                {t('Link…')}
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/if}

    <section class="space-y-2">
      <div class="flex items-center justify-between gap-3 flex-wrap">
        <h2 class="text-sm font-semibold">{t('Rows ({shown} of {total})', { shown: filteredRows.length, total: detail.rows.length })}</h2>
        <div class="flex items-center gap-3">
          <label class="text-xs text-fg2 flex items-center gap-1.5 cursor-pointer">
            <input type="checkbox" bind:checked={showPriors} />
            <span>{t('Show previous values from inline-prior columns')}</span>
          </label>
          <div class="flex items-center gap-1 text-xs">
            {#each ['all', 'matched', 'unmatched', 'abnormal', 'inline_prior'] as f}
              <button
                class="btn {filter === f ? 'border-accent text-accent' : ''}"
                onclick={() => (filter = f as typeof filter)}
              >{t(f.replace('_', ' '))}</button>
            {/each}
          </div>
        </div>
      </div>

      <div class="card overflow-x-auto">
        <table class="w-full text-sm">
          <thead class="text-fg2 text-xs uppercase tracking-wide">
            <tr class="border-b border-line">
              <th class="text-left px-3 py-2">{t('Analyte')}</th>
              <th class="text-left px-3 py-2 w-32">{t('Trend')}</th>
              <th class="text-right px-3 py-2">{t('Value')}</th>
              <th class="text-left px-3 py-2">{t('Unit')}</th>
              <th class="text-left px-3 py-2">{t('Ref')}</th>
              <th class="text-left px-3 py-2">{t('Flag')}</th>
              <th class="text-right px-3 py-2">{t('Conf')}</th>
              <th class="text-left px-3 py-2">{t('Method')}</th>
              <th class="text-left px-3 py-2 w-16">{t('Source')}</th>
              <th class="text-left px-3 py-2 w-20"></th>
            </tr>
          </thead>
          <tbody>
            {#each filteredRows as r (r.id)}
              {@const cleanRaw = r.raw_analyte_text.replace(/\s*\[[^\]]+\]\s*$/, '').trim()}
              {@const methodInBrackets = r.raw_analyte_text.match(/\[([^\]]+)\]\s*$/)?.[1] ?? null}
              {@const tiers = parseTiers(r.analyte_categorical_tiers_json)}
              {@const sex = detail.report.patient_sex}
              <!-- Cycle-phase ref (Estradiol, FSH, LH) takes precedence
                   over generic sex defaults when the report is tagged with
                   the phase that was active at collection. -->
              {@const phaseRef = r.value_numeric != null && r.analyte_cycle_dependent && sex === 'f'
                                 ? phaseRefFor(r.analyte_cycle_phases_json, detail.report.cycle_phase)
                                 : null}
              <!-- Compute the sex-aware default_ref. Note: we look it up
                   regardless of whether the row has a printed range, so we
                   can OVERRIDE the stored flag when the printed range was
                   sex-stratified or otherwise mis-keyed for this patient. -->
              {@const sexFallback = !phaseRef && r.value_numeric != null
                                   ? defaultRefFor(r.analyte_default_ref_json, sex) : null}
              {@const preferDefaultRef = r.analyte_cycle_dependent && sex === 'f' && sexFallback}
              {@const matched = !phaseRef && !preferDefaultRef && r.value_numeric != null && tiers.length > 0
                                ? matchTier(r.value_numeric, tiers) : null}
              {@const defaultRef = !matched ? sexFallback : null}
              {@const derivedFlag = phaseRef && r.value_numeric != null
                                  ? flagForPhaseRef(r.value_numeric, phaseRef)
                                  : matched ? tierToFlag(matched.label)
                                  : (defaultRef && r.value_numeric != null
                                       ? flagForDefaultRef(r.value_numeric, defaultRef)
                                       : null)}
              <!-- Reference-source policy honours the user's preference
                   (Settings ▸ Charts ▸ Reference source):
                   - auto    → derived (ontology) wins, fall back to stored.
                   - library → always ontology.
                   - printed → always the lab's printed range. -->
              {@const displayFlag =
                chartPrefs.referenceSource === 'printed'
                  ? r.flag
                  : chartPrefs.referenceSource === 'library'
                    ? derivedFlag
                    : (derivedFlag ?? r.flag)}
              {@const series = trendReadings(r)}
              <tr class="border-b border-line/50 hover:bg-bg3/50 {r.inline_prior_pdf ? 'opacity-70' : ''}">
                <td class="px-3 py-2">
                  {#if r.analyte_id}
                    <a class="text-accent hover:underline" href={`/analyte/${r.analyte_id}`}>
                      {r.analyte_pt_name ?? cleanRaw}
                    </a>
                    {#if r.analyte_method_annotation}
                      <div class="text-[10px] text-fg3 font-mono">{r.analyte_method_annotation}</div>
                    {:else if methodInBrackets}
                      <div class="text-[10px] text-fg3 font-mono">{methodInBrackets}</div>
                    {:else if r.analyte_pt_name && r.analyte_pt_name !== r.raw_analyte_text}
                      <div class="text-[10px] text-fg3 italic" title={t('Printed in PDF as')}>{r.raw_analyte_text}</div>
                    {/if}
                  {:else}
                    <button class="text-warn font-mono hover:underline" onclick={() => openLinkDialog(r.raw_analyte_text)}>
                      {cleanRaw}
                    </button>
                    {#if methodInBrackets}
                      <div class="text-[10px] text-fg3 font-mono">{methodInBrackets}</div>
                    {/if}
                  {/if}
                </td>
                <td class="px-3 py-2 align-middle">
                  {#if r.analyte_id && series.length > 0}
                    <a class="report-trend" href={`/analyte/${r.analyte_id}?patient=${detail.report.patient_id}`} title={t('Open the full longitudinal analyte history')}>
                      <svg class="report-trend__spark" viewBox="0 0 100 28" preserveAspectRatio="none" aria-label={t('{name} longitudinal trend', { name: r.analyte_pt_name ?? cleanRaw })} role="img">
                        {#if series.length >= 2}
                          <polyline points={sparklinePoints(series)} fill="none" stroke="currentColor" stroke-width="2" vector-effect="non-scaling-stroke" />
                        {:else}
                          <circle cx="50" cy="14" r="3" fill="currentColor" stroke="none" />
                        {/if}
                      </svg>
                      <span class="report-trend__count">{t('{count} reading(s)', { count: series.length })}</span>
                    </a>
                  {:else}
                    <span class="text-[11px] text-fg3">—</span>
                  {/if}
                </td>
                <td class="px-3 py-2 text-right tabular-nums">
                  {#if r.value_qualitative}
                    {r.value_qualitative}
                  {:else}
                    {formatNumber(r.value_numeric)}
                  {/if}
                </td>
                <td class="px-3 py-2 text-fg2">{prettyUnit(r.unit)}</td>
                <td class="px-3 py-2 text-fg2 text-xs">
                  {#if phaseRef}
                    {phaseRef.low ?? '—'}–{phaseRef.high ?? '—'}
                    <span class="text-fg3 ml-1">({phaseRef.source})</span>
                  {:else if r.ref_low != null && r.ref_high != null}
                    {formatNumber(r.ref_low)}–{formatNumber(r.ref_high)}
                  {:else if r.ref_high != null}
                    &lt; {formatNumber(r.ref_high)}
                  {:else if r.ref_low != null}
                    &gt; {formatNumber(r.ref_low)}
                  {:else if matched}
                    {formatTierRange(matched)}
                    <span class="text-fg3 ml-1">({matched.label})</span>
                  {:else if defaultRef}
                    {formatDefaultRef(defaultRef)}
                    <span class="text-fg3 ml-1">
                      ({t(defaultRef.source === 'all' ? 'reference' : defaultRef.source === 'm' ? 'male' : 'female')})
                    </span>
                  {:else if r.ref_grammar !== 'none'}
                    <span class="font-mono">{r.ref_grammar}</span>
                  {:else}
                    —
                  {/if}
                </td>
                <td class="px-3 py-2"><FlagPill flag={displayFlag} /></td>
                <td class="px-3 py-2 text-right text-xs tabular-nums {r.confidence < 0.7 ? 'text-warn' : 'text-fg3'}">
                  {(r.confidence * 100).toFixed(0)}%
                </td>
                <td class="px-3 py-2 text-fg3 text-xs truncate max-w-xs" title={r.method_annotation ?? ''}>
                  {r.method_annotation ?? '—'}
                </td>
                <td class="px-3 py-2 text-[10px] font-mono text-fg3">
                  {r.parse_method.replace('parse_', '')}
                  {#if r.inline_prior_pdf}<span class="ml-1 text-fg2">{t('prior')}</span>{/if}
                </td>
                <td class="px-3 py-2">
                  <button
                    class="text-xs text-fg3 hover:text-crit"
                    onclick={() => onDeleteRow(r.id)}
                    title={t('Delete row')}
                  >{t('Delete')}</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>

<AnalyteLinkDialog
  rawText={linkRawText}
  bind:open={linkDialogOpen}
  onLinked={() => refresh()}
/>

<style>
  .report-trend {
    display: inline-flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 7rem;
    color: rgb(var(--accent));
    vertical-align: middle;
  }
  .report-trend:hover { color: rgb(var(--fg-1)); }
  .report-trend__spark {
    display: block;
    width: 7rem;
    height: 1.75rem;
  }
  .report-trend__count {
    color: rgb(var(--fg-3));
    font-size: 0.625rem;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  /* Mirror of the patient page's HRT tag — same look, included here so the
     header chip renders without depending on the patient page's CSS being
     loaded (each page is its own SvelteKit chunk). */
  .hrt-tag {
    display: inline-flex;
    align-items: center;
    padding: 0.05rem 0.45rem;
    border-radius: 9999px;
    font-size: 0.7rem;
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
</style>
