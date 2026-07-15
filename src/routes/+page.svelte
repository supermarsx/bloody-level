<script lang="ts">
  import { onMount } from 'svelte';
  import { listReports, type ReportSummary } from '$api/reports';
  import { patientOverviews, type PatientOverview } from '$api/records-admin';
  import { listAuditEntries, type AuditEntry } from '$api/audit';
  import { AppError } from '$api/errors';
  import { toasts } from '../lib/toasts/store.svelte';
  import { formatDate, formatRelativeSpan } from '$format/dates';
  import { format } from 'date-fns';

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
        listAuditEntries({ limit: 6 }).catch(() => ({ entries: [] as AuditEntry[], total: 0, distinct_actions: [], distinct_entity_types: [] })),
      ]);
      overviews = pos;
      recentReports = rs.slice(0, 8);
      recentAudit = audit.entries;
    } catch (e) {
      err = AppError.fromUnknown(e);
      if (err.kind !== 'locked') toasts.error(err, { retry: refresh });
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

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
      .slice(0, 6)
  );

  // ── Sorted patient grid (latest activity first) ─────────────────────────
  const patientsByActivity = $derived(
    [...overviews].sort((a, b) =>
      (b.latest_collection_date_iso ?? '').localeCompare(a.latest_collection_date_iso ?? '')
    )
  );

  function fmtTs(ts: number): string {
    try { return format(new Date(ts * 1000), 'yyyy-MM-dd HH:mm'); }
    catch { return String(ts); }
  }
</script>

<div class="space-y-5">
  <div class="flex items-end justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">Dashboard</h1>
      <p class="text-sm text-fg2 mt-1">
        {#if overviews.length === 0 && !loading && !err}
          No data yet — drop PDFs in <a class="text-accent hover:underline" href="/ingest">Ingest</a>.
        {:else if !err}
          {kpis.patients} patient{kpis.patients === 1 ? '' : 's'} ·
          {kpis.totalReports} report{kpis.totalReports === 1 ? '' : 's'} on file ·
          {kpis.totalRecent} in the last 12 months.
        {/if}
      </p>
    </div>
    <button class="btn" onclick={refresh} disabled={loading}>{loading ? 'Refreshing…' : 'Refresh'}</button>
  </div>

  {#if err}
    <div class="card p-4 border-l-4 border-crit space-y-2">
      <div class="text-sm font-medium text-crit">{err.message}</div>
      <div class="text-[11px] text-fg3 font-mono">{err.code}</div>
      {#if err.retryable}
        <button class="btn" onclick={refresh}>Retry</button>
      {/if}
    </div>
  {/if}

  {#if loading && !err}
    <div class="card p-6 text-sm text-fg2">Loading…</div>
  {:else if overviews.length > 0}

    <!-- ───── KPI strip ───── -->
    <section class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <div class="card p-3">
        <div class="text-xs text-fg2">Patients</div>
        <div class="text-2xl font-semibold tabular-nums">{kpis.patients}</div>
        <div class="text-[10px] text-fg3 mt-0.5">tracked</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">Reports</div>
        <div class="text-2xl font-semibold tabular-nums">{kpis.totalReports}</div>
        <div class="text-[10px] text-fg3 mt-0.5">{kpis.totalRecent} in last 12 mo</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">Abnormal results</div>
        <div class="text-2xl font-semibold tabular-nums {kpis.recentAbnormal > 0 ? 'text-warn' : ''}">{kpis.totalAbnormal}</div>
        <div class="text-[10px] text-fg3 mt-0.5">{kpis.recentAbnormal} recent</div>
      </div>
      <div class="card p-3">
        <div class="text-xs text-fg2">Critical results</div>
        <div class="text-2xl font-semibold tabular-nums {kpis.recentCritical > 0 ? 'text-crit' : ''}">{kpis.totalCritical}</div>
        <div class="text-[10px] text-fg3 mt-0.5">{kpis.recentCritical} recent</div>
      </div>
    </section>

    <!-- ───── Spotlight: patients with recent flags ───── -->
    {#if spotlight.length > 0}
      <section>
        <h2 class="text-sm font-semibold mb-2 flex items-center gap-2">
          <span class="inline-block w-2 h-2 rounded-full bg-warn"></span>
          Recent abnormal flags
          <span class="text-xs text-fg3 font-normal">(last 12 months)</span>
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {#each spotlight as p}
            <a class="card p-3 border-l-4 {p.recent_critical_row_count > 0 ? 'border-crit' : 'border-warn'} hover:bg-bg3 transition-colors" href={`/patient/${p.id}`}>
              <div class="flex items-baseline justify-between gap-2">
                <span class="text-sm font-medium truncate">{p.display_name}</span>
                <span class="text-[10px] text-fg3 uppercase font-mono">{p.sex}</span>
              </div>
              <div class="flex items-center gap-3 mt-1.5 text-xs">
                {#if p.recent_critical_row_count > 0}
                  <span class="text-crit font-medium tabular-nums">{p.recent_critical_row_count} critical</span>
                {/if}
                {#if p.recent_abnormal_row_count > 0}
                  <span class="text-warn font-medium tabular-nums">{p.recent_abnormal_row_count} abnormal</span>
                {/if}
              </div>
              <div class="text-[11px] text-fg3 mt-1">
                {p.recent_report_count} recent report{p.recent_report_count === 1 ? '' : 's'}
                {#if p.latest_collection_date_iso}
                  · {formatDate(p.latest_collection_date_iso)}
                {/if}
              </div>
            </a>
          {/each}
        </div>
      </section>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <!-- ───── Recent reports ───── -->
      <section class="lg:col-span-2">
        <h2 class="text-sm font-semibold mb-2">Recent reports</h2>
        {#if recentReports.length === 0}
          <div class="card p-3 text-xs text-fg3">No reports yet.</div>
        {:else}
          <div class="card divide-y divide-line">
            {#each recentReports as r}
              <a class="flex items-center justify-between gap-3 px-3 py-2 hover:bg-bg3 transition-colors" href={`/report/${r.id}`}>
                <div class="flex flex-col min-w-0">
                  <span class="text-sm font-medium truncate">
                    {r.patient_name}
                    {#if r.nickname}
                      <span class="ml-1 text-fg3 font-normal">— {r.nickname}</span>
                    {/if}
                  </span>
                  <span class="text-xs text-fg3">{formatDate(r.collection_date_iso)} · {r.row_count} rows · tier {r.ingest_tier}</span>
                </div>
                <span class="text-xs text-fg2 tabular-nums">conf {(r.doc_confidence * 100).toFixed(0)}%</span>
              </a>
            {/each}
          </div>
        {/if}
      </section>

      <!-- ───── Recent activity (audit trail tail) ───── -->
      <section>
        <h2 class="text-sm font-semibold mb-2 flex items-baseline justify-between">
          <span>Recent activity</span>
          <a href="/audit" class="text-xs text-accent hover:underline font-normal">Open audit log →</a>
        </h2>
        {#if recentAudit.length === 0}
          <div class="card p-3 text-xs text-fg3">No activity yet.</div>
        {:else}
          <div class="card divide-y divide-line">
            {#each recentAudit as a}
              <div class="px-3 py-2 text-xs">
                <div class="flex items-center justify-between gap-2">
                  <span class="font-mono uppercase text-[10px] text-fg3">{a.action}</span>
                  <span class="text-[10px] text-fg3 tabular-nums">{fmtTs(a.ts)}</span>
                </div>
                <div class="text-fg1 mt-0.5">{a.summary}</div>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <!-- ───── Patient grid (full set, by activity) ───── -->
    <section>
      <h2 class="text-sm font-semibold mb-2">Patients</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        {#each patientsByActivity as p}
          <a class="card p-3 hover:bg-bg3 transition-colors" href={`/patient/${p.id}`}>
            <div class="flex items-baseline justify-between gap-2">
              <span class="text-sm font-medium truncate">
                {p.display_name}
                {#if p.nickname}
                  <span class="ml-1 text-fg3 font-normal">"{p.nickname}"</span>
                {/if}
              </span>
              <span class="text-xs text-fg3 uppercase">{p.sex}</span>
            </div>
            <div class="text-xs text-fg2 mt-1">
              {p.report_count} report{p.report_count === 1 ? '' : 's'} ·
              {p.distinct_analyte_count} analytes
            </div>
            <div class="text-[11px] text-fg3 mt-0.5">
              {#if p.latest_collection_date_iso}
                latest {formatDate(p.latest_collection_date_iso)}
                {#if p.earliest_collection_date_iso && p.earliest_collection_date_iso !== p.latest_collection_date_iso}
                  · span {formatRelativeSpan(p.earliest_collection_date_iso, p.latest_collection_date_iso)}
                {/if}
              {:else}
                no reports yet
              {/if}
            </div>
            {#if p.abnormal_row_count > 0 || p.critical_row_count > 0}
              <div class="flex items-center gap-2 mt-1 text-[11px]">
                {#if p.critical_row_count > 0}
                  <span class="text-crit tabular-nums">{p.critical_row_count} crit</span>
                {/if}
                {#if p.abnormal_row_count > 0}
                  <span class="text-warn tabular-nums">{p.abnormal_row_count} abn</span>
                {/if}
              </div>
            {/if}
          </a>
        {/each}
      </div>
    </section>
  {/if}
</div>
