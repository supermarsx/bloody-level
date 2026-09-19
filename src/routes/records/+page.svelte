<script lang="ts">
  import { onMount } from 'svelte';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { listPatients, listReports, type PatientSummary, type ReportSummary } from '$api/reports';
  import * as admin from '$api/records-admin';
  import * as reparse from '$api/reparse';
  import { AppError } from '$api/errors';
  import { toasts } from '../../lib/toasts/store.svelte';
  import Icon from '$components/icon.svelte';
  import { formatDate } from '$format/dates';
  import PatientPickerDialog from '$components/patient-picker-dialog.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import '$lib/i18n/data-routes';

  type ReportGroup = {
    key: string;
    patient_id: string;
    patient_name: string;
    collection_date_iso: string;
    sources: ReportSummary[];
    total_rows: number;
    avg_confidence: number;
    min_confidence: number;
  };

  let patients = $state<PatientSummary[]>([]);
  let reports = $state<ReportSummary[]>([]);
  let loading = $state(true);

  let editingPatient = $state<string | null>(null);
  let editPatient = $state<{ display_name: string; sex: 'm' | 'f' | 'x' | '?'; dob_iso: string }>({
    display_name: '', sex: '?', dob_iso: ''
  });

  let mergeDialogOpen = $state(false);
  let mergeSource = $state<PatientSummary | null>(null);

  async function onMergeInto(target: PatientSummary) {
    if (!mergeSource) return;
    const ok = await ask(
      t('Merge "{source}" INTO "{target}"?\n\nAll {count} report(s) will be reassigned to {target}, then {source} will be deleted. Useful for name changes (marriage / legal). This cannot be undone.', { source: mergeSource.display_name, target: target.display_name, count: mergeSource.report_count }),
      { title: t('Merge patients'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      const r = await admin.mergePatients({ source_id: mergeSource.id, target_id: target.id });
      toasts.success(
        t('Patients merged'),
        t('{count} report(s) reassigned to {target}.', { count: r.reports_moved, target: target.display_name })
      );
      mergeSource = null;
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  function openMergeFor(p: PatientSummary) {
    mergeSource = p;
    mergeDialogOpen = true;
  }

  let filterText = $state('');
  let filterPatient = $state<string | null>(null);

  // Bulk selection — by report ID, not by group, so partial selection works.
  let selected = $state<Set<string>>(new Set());
  let expanded = $state<Set<string>>(new Set());

  async function refresh() {
    loading = true;
    try {
      [patients, reports] = await Promise.all([listPatients(), listReports()]);
      // Drop selections for reports that no longer exist
      const valid = new Set(reports.map((r) => r.id));
      selected = new Set([...selected].filter((id) => valid.has(id)));
    } catch (e) {
      toasts.error(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  // ───────── Patient actions ─────────
  async function onDeletePatient(p: PatientSummary) {
    const ok = await ask(
      t('Delete patient "{name}" and ALL {count} report(s)?\n\nThis cannot be undone.', { name: p.display_name, count: p.report_count }),
      { title: t('Delete patient'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deletePatient(p.id);
      toasts.success(t('Deleted patient {name}', { name: p.display_name }));
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  function startEditPatient(p: PatientSummary) {
    editingPatient = p.id;
    editPatient = {
      display_name: p.display_name,
      sex: (p.sex as 'm' | 'f' | 'x' | '?') ?? '?',
      dob_iso: ''
    };
  }

  async function savePatient(p: PatientSummary) {
    try {
      await admin.updatePatient({
        id: p.id,
        display_name: editPatient.display_name.trim() || p.display_name,
        sex: editPatient.sex,
        dob_iso: editPatient.dob_iso.trim() || null
      });
      toasts.success(t('Patient updated'));
      editingPatient = null;
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  // ───────── Report grouping by (patient, date) ─────────
  const filteredReports = $derived.by(() => {
    let r = reports;
    if (filterPatient) r = r.filter((x) => x.patient_id === filterPatient);
    const q = filterText.toLowerCase().trim();
    if (q) {
      r = r.filter((x) =>
        x.patient_name.toLowerCase().includes(q) ||
        x.id.toLowerCase().includes(q) ||
        x.collection_date_iso.includes(q)
      );
    }
    return r;
  });

  const groups = $derived.by(() => {
    const m = new Map<string, ReportSummary[]>();
    for (const r of filteredReports) {
      const k = `${r.patient_id}|${r.collection_date_iso}`;
      const arr = m.get(k);
      if (arr) arr.push(r);
      else m.set(k, [r]);
    }
    const out: ReportGroup[] = [];
    for (const [key, sources] of m.entries()) {
      const total_rows = sources.reduce((s, r) => s + r.row_count, 0);
      const avg = sources.reduce((s, r) => s + r.doc_confidence, 0) / sources.length;
      const min = sources.reduce((s, r) => Math.min(s, r.doc_confidence), 1);
      out.push({
        key,
        patient_id: sources[0].patient_id,
        patient_name: sources[0].patient_name,
        collection_date_iso: sources[0].collection_date_iso,
        sources,
        total_rows,
        avg_confidence: avg,
        min_confidence: min
      });
    }
    out.sort((a, b) => b.collection_date_iso.localeCompare(a.collection_date_iso));
    return out;
  });

  // ───────── Bulk select / delete ─────────
  function toggleSelect(reportId: string) {
    if (selected.has(reportId)) selected.delete(reportId);
    else selected.add(reportId);
    selected = new Set(selected);
  }

  function toggleSelectGroup(g: ReportGroup) {
    const allSelected = g.sources.every((r) => selected.has(r.id));
    if (allSelected) {
      for (const r of g.sources) selected.delete(r.id);
    } else {
      for (const r of g.sources) selected.add(r.id);
    }
    selected = new Set(selected);
  }

  function toggleExpand(key: string) {
    if (expanded.has(key)) expanded.delete(key);
    else expanded.add(key);
    expanded = new Set(expanded);
  }

  function selectAllVisible() {
    for (const g of groups) for (const r of g.sources) selected.add(r.id);
    selected = new Set(selected);
  }
  function clearSelection() { selected = new Set(); }

  async function deleteSelected() {
    if (selected.size === 0) return;
    const n = selected.size;
    const ok = await ask(
      t('Delete {count} report(s)?\n\nAll parsed rows and cached PDFs will be removed. This cannot be undone.', { count: n }),
      { title: t('Bulk delete'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      const res = await admin.bulkDeleteReports([...selected]);
      if (res.failed.length === 0) {
        toasts.success(t('Deleted {count} report(s)', { count: res.deleted }));
      } else {
        toasts.warn(
          t('Deleted {deleted}, failed {failed}', { deleted: res.deleted, failed: res.failed.length }),
          t('Failed IDs: {ids}', { ids: res.failed.join(', ') })
        );
      }
      clearSelection();
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  let reparsingAll = $state(false);
  async function onReparseAll() {
    if (reports.length === 0) return;
    const ok = await ask(
      t("Re-parse all {count} reports with the current parser?\n\nDoesn't touch the source PDFs — just re-runs the parser against each report's stored raw text and replaces the parsed rows.", { count: reports.length }),
      { title: t('Re-parse all'), kind: 'info' }
    );
    if (!ok) return;
    reparsingAll = true;
    try {
      const r = await reparse.reparseAll();
      if (r.failed.length === 0) {
        toasts.success(
          t('Re-parse complete'),
          t('{succeeded}/{total} reports · {rows} total rows · {diagnostics} diagnostics', { succeeded: r.succeeded, total: r.total, rows: r.total_rows_after, diagnostics: r.total_parse_audit_entries })
        );
      } else {
        toasts.warn(
          t('Re-parse done with {count} failures', { count: r.failed.length }),
          t('{succeeded}/{total} succeeded. First failure: {id} — {error}', { succeeded: r.succeeded, total: r.total, id: r.failed[0][0], error: r.failed[0][1] })
        );
      }
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { reparsingAll = false; }
  }

  let backfillingSex = $state(false);
  async function onBackfillSex() {
    backfillingSex = true;
    try {
      const r = await admin.backfillPatientSex();
      toasts.success(
        t('Patient sex re-inferred'),
        t('{updated}/{scanned} updated · {unknown} still unknown', { updated: r.patients_updated, scanned: r.patients_scanned, unknown: r.patients_still_unknown })
      );
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { backfillingSex = false; }
  }

  async function onDeleteReport(r: ReportSummary) {
    const ok = await ask(
      t('Delete report {date} for {name}?', { date: formatDate(r.collection_date_iso), name: r.patient_name }),
      { title: t('Delete report'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      await admin.deleteReport(r.id);
      toasts.success(t('Report deleted'));
      await refresh();
    } catch (e) { toasts.error(e); }
  }

  function groupSelectionState(g: ReportGroup): 'none' | 'partial' | 'all' {
    const c = g.sources.filter((r) => selected.has(r.id)).length;
    if (c === 0) return 'none';
    if (c === g.sources.length) return 'all';
    return 'partial';
  }
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-xl font-semibold">{t('Records')}</h1>
    <p class="text-sm text-fg2 mt-1">
      {t('Manage patients, reports, and parsed data. Reports sharing the same date for the same patient are grouped — expand to see individual sources.')}
    </p>
  </div>

  {#if loading}
    <div class="card p-6 text-sm text-fg2">{t('Loading…')}</div>
  {:else}
    <!-- ───────────────────── Patients ───────────────────── -->
    <section class="space-y-2">
      <div class="flex items-baseline justify-between">
        <h2 class="text-sm font-semibold">{t('Patients')} ({patients.length})</h2>
        <div class="flex items-center gap-2">
          {#if reports.length > 0}
            <button class="btn" disabled={reparsingAll} onclick={onReparseAll} title={t('Re-run the current parser against every stored report')}>
              {reparsingAll ? t('Re-parsing…') : t('Re-parse all ({count})', { count: reports.length })}
            </button>
          {/if}
          <button class="btn" disabled={backfillingSex} onclick={onBackfillSex}
            title={t("Re-infer patient sex from each patient's stored raw_text — fixes legacy data ingested before the sex field was kept up to date.")}>
            {backfillingSex ? t('Re-inferring…') : t('Re-infer sex')}
          </button>
          <button class="btn" onclick={refresh}>{t('Refresh')}</button>
        </div>
      </div>

      {#if patients.length === 0}
        <div class="card p-6 text-sm text-fg2">{t('No patients yet.')}</div>
      {:else}
        <div class="card divide-y divide-line">
          {#each patients as p}
            <div class="px-3 py-2">
              {#if editingPatient === p.id}
                <div class="flex flex-wrap items-center gap-2">
                  <input
                    type="text"
                    class="input flex-1 min-w-[200px]"
                    bind:value={editPatient.display_name}
                    placeholder={t('Display name')}
                  />
                  <select
                    class="select"
                    bind:value={editPatient.sex}
                  >
                    <option value="?">?</option>
                    <option value="m">m</option>
                    <option value="f">f</option>
                    <option value="x">x</option>
                  </select>
                  <input
                    type="date"
                    class="input"
                    bind:value={editPatient.dob_iso}
                  />
                  <button class="btn-accent" onclick={() => savePatient(p)}>{t('Save')}</button>
                  <button class="btn" onclick={() => (editingPatient = null)}>{t('Cancel')}</button>
                </div>
              {:else}
                <div class="flex items-center justify-between gap-3">
                  <div class="flex flex-col min-w-0 flex-1">
                    <a href={`/patient/${p.id}`} class="text-sm font-medium hover:underline truncate">
                      {p.display_name}
                    </a>
                    <div class="text-xs text-fg3 flex items-center gap-2">
                      <span class="uppercase">{p.sex}</span>
                      <span>·</span>
                      <span>{t('{count} report(s)', { count: p.report_count })}</span>
                      {#if p.latest_collection_date_iso}
                        <span>·</span>
                        <span>{t('latest')} {formatDate(p.latest_collection_date_iso)}</span>
                      {/if}
                    </div>
                  </div>
                  <div class="flex items-center gap-1 shrink-0">
                    <button
                      class="btn"
                      onclick={() => (filterPatient = filterPatient === p.id ? null : p.id)}
                      title={t('Filter reports below')}
                    >{#if filterPatient === p.id}<Icon name="check" size={13} /> {t('Filtered')}{:else}<Icon name="filter" size={13} /> {t('Filter')}{/if}</button>
                    <button class="btn" onclick={() => startEditPatient(p)}>{t('Edit')}</button>
                    {#if patients.length > 1}
                      <button class="btn" onclick={() => openMergeFor(p)} title={t('Merge this patient into another (e.g. name change)')}>
                        {t('Merge')}…
                      </button>
                    {/if}
                    <button class="btn text-crit hover:bg-crit/10" onclick={() => onDeletePatient(p)}>
                      {t('Delete')}
                    </button>
                  </div>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- ───────────────────── Reports (grouped) ───────────────────── -->
    <section class="space-y-2">
      <div class="flex items-baseline justify-between gap-3 flex-wrap">
        <h2 class="text-sm font-semibold">
          {t('Reports')} — {groups.length} {t('group(s)')},
          {filteredReports.length} {t('source(s)')}
          {#if filteredReports.length !== reports.length}<span class="text-fg3"> {t('of')} {reports.length}</span>{/if}
        </h2>
        <div class="flex items-center gap-2 flex-wrap">
          {#if filterPatient}
            <button class="btn text-xs" onclick={() => (filterPatient = null)}>
              {t('Clear patient filter')}
            </button>
          {/if}
          <input
            type="search"
            class="search w-48"
            placeholder={t('Search…')}
            bind:value={filterText}
          />
        </div>
      </div>

      <!-- Bulk action bar -->
      {#if selected.size > 0}
        <div class="card-tight bg-accent/10 border-accent flex items-center justify-between gap-3">
          <span class="text-sm">
            <span class="font-medium">{selected.size}</span> {t('selected')}
          </span>
          <div class="flex items-center gap-2">
            <button class="btn text-xs" onclick={clearSelection}>{t('Clear')}</button>
            <button class="btn text-crit hover:bg-crit/10" onclick={deleteSelected}>
              {t('Delete {count} selected', { count: selected.size })}
            </button>
          </div>
        </div>
      {/if}

      {#if groups.length === 0}
        <div class="card p-6 text-sm text-fg2">{t('No reports match.')}</div>
      {:else}
        <div class="card overflow-hidden">
          <table class="w-full text-sm">
            <thead class="text-fg2 text-xs uppercase tracking-wide">
              <tr class="border-b border-line bg-bg3/30">
                <th class="text-left px-3 py-2 w-8">
                  <input
                    type="checkbox"
                    title={t('Select all visible')}
                    onchange={(e) => (e.currentTarget.checked ? selectAllVisible() : clearSelection())}
                    checked={filteredReports.length > 0 && filteredReports.every((r) => selected.has(r.id))}
                  />
                </th>
                <th class="text-left px-3 py-2 w-6"></th>
                <th class="text-left px-3 py-2">{t('Date')}</th>
                <th class="text-left px-3 py-2">{t('Patient')}</th>
                <th class="text-right px-3 py-2">{t('Sources')}</th>
                <th class="text-right px-3 py-2">{t('Rows')}</th>
                <th class="text-right px-3 py-2">{t('Conf')}</th>
                <th class="text-left px-3 py-2 w-24">{t('Actions')}</th>
              </tr>
            </thead>
            <tbody>
              {#each groups as g (g.key)}
                {@const sel = groupSelectionState(g)}
                {@const exp = expanded.has(g.key)}
                <tr class="border-b border-line/50 hover:bg-bg3/50">
                  <td class="px-3 py-2">
                    <input
                      type="checkbox"
                      checked={sel === 'all'}
                      indeterminate={sel === 'partial'}
                      onchange={() => toggleSelectGroup(g)}
                    />
                  </td>
                  <td class="px-3 py-2">
                    {#if g.sources.length > 1}
                      <button
                        class="text-fg3 hover:text-fg1 font-mono text-xs"
                        onclick={() => toggleExpand(g.key)}
                        title={t(exp ? 'Collapse' : 'Expand')}
                      >{exp ? '▾' : '▸'}</button>
                    {/if}
                  </td>
                  <td class="px-3 py-2 tabular-nums">
                    {formatDate(g.collection_date_iso)}
                    {#if g.sources.length === 1 && g.sources[0].nickname}
                      <div class="text-[11px] text-fg2 italic font-normal mt-0.5 truncate max-w-[14rem]" title={g.sources[0].nickname}>
                        {g.sources[0].nickname}
                      </div>
                    {/if}
                  </td>
                  <td class="px-3 py-2 truncate max-w-xs">
                    <a href={`/patient/${g.patient_id}`} class="hover:underline">{g.patient_name}</a>
                  </td>
                  <td class="px-3 py-2 text-right tabular-nums">
                    {#if g.sources.length > 1}
                      <span class="pill-warn">{g.sources.length}×</span>
                    {:else}
                      <span class="text-fg3">1</span>
                    {/if}
                  </td>
                  <td class="px-3 py-2 text-right tabular-nums">{g.total_rows}</td>
                  <td class="px-3 py-2 text-right tabular-nums {g.min_confidence < 0.7 ? 'text-warn' : ''}">
                    {(g.avg_confidence * 100).toFixed(0)}%
                    {#if g.sources.length > 1 && g.min_confidence !== g.avg_confidence}
                      <span class="text-[10px] text-fg3">/ {t('min')} {(g.min_confidence * 100).toFixed(0)}%</span>
                    {/if}
                  </td>
                  <td class="px-3 py-2">
                    {#if g.sources.length === 1}
                      <div class="flex items-center gap-1">
                        <a class="btn text-xs" href={`/report/${g.sources[0].id}`}>{t('Open')}</a>
                        <button class="btn text-xs text-crit hover:bg-crit/10" onclick={() => onDeleteReport(g.sources[0])}>
                          {t('Delete')}
                        </button>
                      </div>
                    {:else}
                      <button class="btn text-xs" onclick={() => toggleExpand(g.key)}>
                        {t(exp ? 'Collapse' : 'Expand')}
                      </button>
                    {/if}
                  </td>
                </tr>

                {#if exp && g.sources.length > 1}
                  {#each g.sources as r (r.id)}
                    <tr class="border-b border-line/30 bg-bg2/40">
                      <td class="px-3 py-1.5 pl-6">
                        <input
                          type="checkbox"
                          checked={selected.has(r.id)}
                          onchange={() => toggleSelect(r.id)}
                        />
                      </td>
                      <td colspan="2" class="px-3 py-1.5 text-xs text-fg3">
                        ↳ <span class="font-mono">{r.id}</span>
                        {#if r.nickname}
                          <span class="text-fg2 italic ml-1">— {r.nickname}</span>
                        {/if}
                      </td>
                      <td class="px-3 py-1.5 text-xs text-fg3">{t('tier')} {r.ingest_tier}</td>
                      <td colspan="1" class="px-3 py-1.5"></td>
                      <td class="px-3 py-1.5 text-right tabular-nums text-xs">{r.row_count}</td>
                      <td class="px-3 py-1.5 text-right tabular-nums text-xs {r.doc_confidence < 0.7 ? 'text-warn' : 'text-fg3'}">
                        {(r.doc_confidence * 100).toFixed(0)}%
                      </td>
                      <td class="px-3 py-1.5">
                        <div class="flex items-center gap-1">
                          <a class="btn text-xs" href={`/report/${r.id}`}>{t('Open')}</a>
                          <button class="btn text-xs text-crit hover:bg-crit/10" onclick={() => onDeleteReport(r)}>
                            {t('Delete')}
                          </button>
                        </div>
                      </td>
                    </tr>
                  {/each}
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  {/if}

  <PatientPickerDialog
    bind:open={mergeDialogOpen}
    excludeId={mergeSource?.id}
    title={mergeSource ? t('Merge "{name}" into…', { name: mergeSource.display_name }) : t('Pick a patient')}
    confirmLabel={t('Merge')}
    onPick={onMergeInto}
  />
</div>
