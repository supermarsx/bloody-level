<script lang="ts">
  import { onMount } from 'svelte';
  import { listReports, type ReportSummary } from '$api/reports';
  import { patientOverviews, type PatientOverview } from '$api/records-admin';
  import { listAuditEntries, type AuditEntry } from '$api/audit';
  import { AppError } from '$api/errors';
  import { toasts } from '../lib/toasts/store.svelte';
  import { formatDate, formatRelativeSpan } from '$format/dates';
  import { format } from 'date-fns';
  import Icon from '$components/icon.svelte';
  import { dashboardPrefs, type DashboardSection } from '$lib/dashboard/prefs.svelte';
  import { t } from '$lib/i18n/index.svelte';

  let overviews = $state<PatientOverview[]>([]);
  let recentReports = $state<ReportSummary[]>([]);
  let recentAudit = $state<AuditEntry[]>([]);
  let loading = $state(true);
  let err = $state<AppError | null>(null);

  async function refresh() {
    loading = true;
    err = null;
    try {
      const [pos, rs, audit] = await Promise.all([
        patientOverviews(),
        listReports(),
        listAuditEntries({ limit: dashboardPrefs.activityLimit }).catch(() => ({ entries: [] as AuditEntry[], total: 0, distinct_actions: [], distinct_entity_types: [] })),
      ]);
      overviews = pos;
      recentReports = rs.slice(0, dashboardPrefs.reportLimit);
      recentAudit = audit.entries;
    } catch (e) {
      err = AppError.fromUnknown(e);
      if (err.kind !== 'locked') toasts.error(err, { retry: refresh });
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await dashboardPrefs.load();
    await refresh();
  });

  // ── Aggregate KPIs (whole library) ──────────────────────────────────────
  const kpis = $derived.by(() => {
    const totalReports = overviews.reduce((s, p) => s + p.report_count, 0);
    const totalRecent  = overviews.reduce((s, p) => s + p.recent_report_count, 0);
    const totalAbnormal = overviews.reduce((s, p) => s + p.abnormal_row_count, 0);
    const totalCritical = overviews.reduce((s, p) => s + p.critical_row_count, 0);
    const recentAbnormal = overviews.reduce((s, p) => s + p.recent_abnormal_row_count, 0);
    const recentCritical = overviews.reduce((s, p) => s + p.recent_critical_row_count, 0);
    return {
      patients: overviews.length,
      totalReports, totalRecent,
      totalAbnormal, totalCritical,
      recentAbnormal, recentCritical,
    };
  });

  // ── Spotlight: patients with recent abnormal/critical findings ──────────
  const spotlight = $derived(
    overviews
      .filter((p) => p.recent_abnormal_row_count + p.recent_critical_row_count > 0)
      .sort((a, b) => {
        const sa = a.recent_critical_row_count * 10 + a.recent_abnormal_row_count;
        const sb = b.recent_critical_row_count * 10 + b.recent_abnormal_row_count;
        return sb - sa;
      })
      .slice(0, dashboardPrefs.spotlightLimit)
  );

  // ── Sorted patient grid (latest activity first) ─────────────────────────
  const patientsByActivity = $derived(
    [...overviews]
      .sort((a, b) =>
      (b.latest_collection_date_iso ?? '').localeCompare(a.latest_collection_date_iso ?? '')
      )
      .slice(0, dashboardPrefs.patientLimit || undefined)
  );

  const displayedSections = $derived<DashboardSection[]>(
    dashboardPrefs.order.filter((section) => dashboardPrefs.visible[section])
  );

  function fmtTs(ts: number): string {
    try { return format(new Date(ts * 1000), 'yyyy-MM-dd HH:mm'); }
    catch { return String(ts); }
  }
</script>

<div class="space-y-5">
  <div class="flex items-end justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">{t('Dashboard')}</h1>
      <p class="text-sm text-fg2 mt-1">
        {#if overviews.length === 0 && !loading && !err}
          {t('No data yet — drop PDFs in')} <a class="text-accent hover:underline" href="/ingest">{t('Ingest')}</a>.
        {:else if !err}
          {t(kpis.patients === 1 ? '{count} patient' : '{count} patients', { count: kpis.patients })} ·
          {t(kpis.totalReports === 1 ? '{count} report' : '{count} reports', { count: kpis.totalReports })} {t('on file')} ·
          {kpis.totalRecent} {t('in the last 12 months.')}
        {/if}
      </p>
    </div>
    <div class="flex items-center gap-2">
      <a class="btn" href="/settings#dashboard">
        <Icon name="settings" size={14} />
        {t('Customize')}
      </a>
      <button class="btn" onclick={refresh} disabled={loading}>
        {#if loading}<Icon name="chart" size={14} />{/if}
        {loading ? t('Refreshing…') : t('Refresh')}
      </button>
    </div>
  </div>

  {#if err}
    <div class="card p-4 border-l-4 border-crit space-y-2">
      <div class="text-sm font-medium text-crit">{err.message}</div>
      <div class="text-[11px] text-fg3 font-mono">{err.code}</div>
      {#if err.retryable}
        <button class="btn" onclick={refresh}>{t('Retry')}</button>
      {/if}
    </div>
  {/if}

  {#if loading && !err}
    <div class="card p-6 text-sm text-fg2">{t('Loading…')}</div>
  {:else if overviews.length > 0}

    {#if displayedSections.length === 0}
      <div class="card p-5 text-sm text-fg2">
        <div class="flex items-center gap-2 font-medium text-fg1"><Icon name="dashboard" size={16} /> {t('Dashboard sections are hidden')}</div>
        <p class="mt-1 text-xs text-fg3">{t('Open Customize to choose which information appears here.')}</p>
      </div>
    {:else}
      <div class="dashboard-sections">
      {#each displayedSections as section (section)}
        {#if section === 'kpis'}
          <section class="dashboard-section dashboard-section--full grid grid-cols-2 md:grid-cols-4 gap-3" aria-label={t('Summary cards')}>
            <div class="card p-3"><div class="text-xs text-fg2">{t('Patients')}</div><div class="text-2xl font-semibold tabular-nums">{kpis.patients}</div><div class="text-[10px] text-fg3 mt-0.5">{t('tracked')}</div></div>
            <div class="card p-3"><div class="text-xs text-fg2">{t('Reports')}</div><div class="text-2xl font-semibold tabular-nums">{kpis.totalReports}</div><div class="text-[10px] text-fg3 mt-0.5">{t('{count} in last 12 mo', { count: kpis.totalRecent })}</div></div>
            <div class="card p-3"><div class="text-xs text-fg2">{t('Abnormal results')}</div><div class="text-2xl font-semibold tabular-nums {kpis.recentAbnormal > 0 ? 'text-warn' : ''}">{kpis.totalAbnormal}</div><div class="text-[10px] text-fg3 mt-0.5">{t('{count} recent', { count: kpis.recentAbnormal })}</div></div>
            <div class="card p-3"><div class="text-xs text-fg2">{t('Critical results')}</div><div class="text-2xl font-semibold tabular-nums {kpis.recentCritical > 0 ? 'text-crit' : ''}">{kpis.totalCritical}</div><div class="text-[10px] text-fg3 mt-0.5">{t('{count} recent', { count: kpis.recentCritical })}</div></div>
          </section>
        {:else if section === 'spotlight'}
          {#if spotlight.length > 0}
            <section class="dashboard-section dashboard-section--full">
              <h2 class="text-sm font-semibold mb-2 flex items-center gap-2"><span class="inline-block w-2 h-2 rounded-full bg-warn"></span>{t('Recent abnormal flags')} <span class="text-xs text-fg3 font-normal">{t('(last 12 months)')}</span></h2>
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                {#each spotlight as p}
                  <a class="card p-3 border-l-4 {p.recent_critical_row_count > 0 ? 'border-crit' : 'border-warn'} hover:bg-bg3 transition-colors" href={`/patient/${p.id}`}>
                    <div class="flex items-baseline justify-between gap-2"><span class="text-sm font-medium truncate">{p.display_name}</span><span class="text-[10px] text-fg3 uppercase font-mono">{p.sex}</span></div>
                    <div class="flex items-center gap-3 mt-1.5 text-xs">{#if p.recent_critical_row_count > 0}<span class="text-crit font-medium tabular-nums">{p.recent_critical_row_count} {t('critical')}</span>{/if}{#if p.recent_abnormal_row_count > 0}<span class="text-warn font-medium tabular-nums">{p.recent_abnormal_row_count} {t('abnormal')}</span>{/if}</div>
                    <div class="text-[11px] text-fg3 mt-1">{t(p.recent_report_count === 1 ? '{count} recent report' : '{count} recent reports', { count: p.recent_report_count })}{#if p.latest_collection_date_iso} · {formatDate(p.latest_collection_date_iso)}{/if}</div>
                  </a>
                {/each}
              </div>
            </section>
          {/if}
        {:else if section === 'reports'}
          <section class="dashboard-section dashboard-section--column">
            <h2 class="text-sm font-semibold mb-2">{t('Recent reports')}</h2>
            {#if recentReports.length === 0}<div class="card p-3 text-xs text-fg3">{t('No reports yet.')}</div>{:else}<div class="card divide-y divide-line">{#each recentReports as r}<a class="flex items-center justify-between gap-3 px-3 py-2 hover:bg-bg3 transition-colors" href={`/report/${r.id}`}><div class="flex flex-col min-w-0"><span class="text-sm font-medium truncate">{r.patient_name}{#if r.nickname}<span class="ml-1 text-fg3 font-normal">— {r.nickname}</span>{/if}</span><span class="text-xs text-fg3">{formatDate(r.collection_date_iso)} · {t('{count} rows', { count: r.row_count })} · {t('tier {tier}', { tier: r.ingest_tier })}</span></div><span class="text-xs text-fg2 tabular-nums">{t('conf {confidence}%', { confidence: (r.doc_confidence * 100).toFixed(0) })}</span></a>{/each}</div>{/if}
          </section>
        {:else if section === 'activity'}
          <section class="dashboard-section dashboard-section--column">
            <h2 class="text-sm font-semibold mb-2 flex items-baseline justify-between"><span>{t('Recent activity')}</span><a href="/audit" class="text-xs text-accent hover:underline font-normal">{t('Open audit log')} <Icon name="arrow-right" size={12} /></a></h2>
            {#if recentAudit.length === 0}<div class="card p-3 text-xs text-fg3">{t('No activity yet.')}</div>{:else}<div class="card divide-y divide-line">{#each recentAudit as a}<div class="px-3 py-2 text-xs"><div class="flex items-center justify-between gap-2"><span class="font-mono uppercase text-[10px] text-fg3">{a.action}</span><span class="text-[10px] text-fg3 tabular-nums">{fmtTs(a.ts)}</span></div><div class="text-fg1 mt-0.5">{a.summary}</div></div>{/each}</div>{/if}
          </section>
        {:else if section === 'patients'}
          <section class="dashboard-section dashboard-section--column">
            <h2 class="text-sm font-semibold mb-2">{t('Patients')}</h2>
            <div class="grid grid-cols-1 gap-3">
              {#each patientsByActivity as p}
                <a class="card p-3 hover:bg-bg3 transition-colors" href={`/patient/${p.id}`}>
                  <div class="flex items-baseline justify-between gap-2"><span class="text-sm font-medium truncate">{p.display_name}{#if p.nickname}<span class="ml-1 text-fg3 font-normal">"{p.nickname}"</span>{/if}</span><span class="text-xs text-fg3 uppercase">{p.sex}</span></div>
                  <div class="text-xs text-fg2 mt-1">{t(p.report_count === 1 ? '{count} report' : '{count} reports', { count: p.report_count })} · {t('{count} analytes', { count: p.distinct_analyte_count })}</div>
                  <div class="text-[11px] text-fg3 mt-0.5">{#if p.latest_collection_date_iso}{t('latest {date}', { date: formatDate(p.latest_collection_date_iso) })}{#if p.earliest_collection_date_iso && p.earliest_collection_date_iso !== p.latest_collection_date_iso} · {t('span {span}', { span: formatRelativeSpan(p.earliest_collection_date_iso, p.latest_collection_date_iso) })}{/if}{:else}{t('no reports yet')}{/if}</div>
                  {#if p.abnormal_row_count > 0 || p.critical_row_count > 0}<div class="flex items-center gap-2 mt-1 text-[11px]">{#if p.critical_row_count > 0}<span class="text-crit tabular-nums">{p.critical_row_count} {t('crit')}</span>{/if}{#if p.abnormal_row_count > 0}<span class="text-warn tabular-nums">{p.abnormal_row_count} {t('abn')}</span>{/if}</div>{/if}
                </a>
              {/each}
            </div>
          </section>
        {/if}
      {/each}
      </div>
    {/if}
  {/if}
</div>

<style>
  .dashboard-sections {
    display: grid;
    grid-template-columns: repeat(3, minmax(18rem, 1fr));
    gap: 1.25rem;
    align-items: stretch;
  }

  .dashboard-section--full { grid-column: 1 / -1; }
  .dashboard-section--column { min-width: 0; }

  @media (max-width: 980px) {
    .dashboard-sections { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }

  @media (max-width: 680px) {
    .dashboard-sections { grid-template-columns: minmax(0, 1fr); }
    .dashboard-section--full { grid-column: auto; }
  }
</style>
