<script lang="ts">
  import { onMount } from 'svelte';
  import { ask } from '@tauri-apps/plugin-dialog';
  import {
    listAuditEntries,
    clearAuditLog,
    type AuditEntry,
    type AuditListFilters,
  } from '$api/audit';
  import { toasts } from '../../lib/toasts/store.svelte';
  import { format, parseISO } from 'date-fns';
  import { t } from '$lib/i18n/index.svelte';

  let entries = $state<AuditEntry[]>([]);
  let total = $state(0);
  let distinctActions = $state<string[]>([]);
  let distinctEntityTypes = $state<string[]>([]);
  let loading = $state(true);

  // Filters
  let actionFilter = $state('');
  let entityTypeFilter = $state('');
  let entityIdFilter = $state('');
  let textFilter = $state('');
  let sinceDate = $state('');
  let untilDate = $state('');

  // Pagination
  const PAGE_SIZE = 50;
  let page = $state(0);
  const lastPage = $derived(Math.max(0, Math.ceil(total / PAGE_SIZE) - 1));

  // Expanded details rows.
  let expanded = $state<Set<number>>(new Set());

  function toggleExpanded(id: number) {
    if (expanded.has(id)) expanded.delete(id);
    else expanded.add(id);
    expanded = new Set(expanded);
  }

  // ISO YYYY-MM-DD → unix seconds (since=00:00, until=23:59:59).
  function dateToUnix(iso: string, end = false): number | undefined {
    if (!iso) return undefined;
    try {
      const d = parseISO(end ? `${iso}T23:59:59` : `${iso}T00:00:00`);
      return Math.floor(d.getTime() / 1000);
    } catch {
      return undefined;
    }
  }

  function buildFilters(): AuditListFilters {
    return {
      action:      actionFilter || undefined,
      entity_type: entityTypeFilter || undefined,
      entity_id:   entityIdFilter || undefined,
      text:        textFilter || undefined,
      since_ts:    dateToUnix(sinceDate),
      until_ts:    dateToUnix(untilDate, true),
      limit:       PAGE_SIZE,
      offset:      page * PAGE_SIZE,
    };
  }

  async function refresh() {
    loading = true;
    try {
      const r = await listAuditEntries(buildFilters());
      entries = r.entries;
      total   = r.total;
      distinctActions      = r.distinct_actions;
      distinctEntityTypes  = r.distinct_entity_types;
    } catch (e) {
      toasts.error(e);
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  // Reset to page 0 whenever filters change. Debounce-light: re-fetch on every
  // change but the SQL is well-indexed and the dataset is bounded (tens of
  // thousands max). If it ever feels sluggish, add a 250ms debouncer.
  $effect(() => {
    void actionFilter;
    void entityTypeFilter;
    void entityIdFilter;
    void textFilter;
    void sinceDate;
    void untilDate;
    page = 0;
    refresh();
  });

  $effect(() => {
    void page;
    refresh();
  });

  function clearFilters() {
    actionFilter = '';
    entityTypeFilter = '';
    entityIdFilter = '';
    textFilter = '';
    sinceDate = '';
    untilDate = '';
    page = 0;
  }

  async function onClearLog() {
    const ok = await ask(
      t('Wipe ALL audit entries?\n\nThis erases the entire audit trail. The wipe itself will be recorded as a single tombstone entry. This cannot be undone.'),
      { title: t('Clear audit log'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      const r = await clearAuditLog();
      toasts.success(t(r.deleted === 1 ? 'Cleared {count} audit entry' : 'Cleared {count} audit entries', { count: r.deleted }));
      page = 0;
      await refresh();
    } catch (e) {
      toasts.error(e);
    }
  }

  function fmtTs(ts: number): string {
    try {
      return format(new Date(ts * 1000), 'yyyy-MM-dd HH:mm:ss');
    } catch {
      return String(ts);
    }
  }

  function pretty(json: string | null): string {
    if (!json) return '';
    try {
      return JSON.stringify(JSON.parse(json), null, 2);
    } catch {
      return json;
    }
  }

  // Stable colour pill per action — derived deterministically so the same
  // action always renders the same accent across pages. Tailwind utility
  // classes resolved here, not interpolated, so the JIT picks them up.
  function actionClass(action: string): string {
    switch (action) {
      case 'create':
      case 'ingest':       return 'bg-ok/10 text-ok border-ok/30';
      case 'update':
      case 'reparse':
      case 'reparse_all':  return 'bg-accent/10 text-accent border-accent/30';
      case 'delete':
      case 'bulk_delete':
      case 'wipe':         return 'bg-crit/10 text-crit border-crit/30';
      case 'merge':
      case 'link':
      case 'reload':       return 'bg-warn/10 text-warn border-warn/30';
      case 'duplicate':    return 'bg-fg3/10 text-fg2 border-fg3/30';
      default:             return 'bg-bg3 text-fg2 border-line';
    }
  }
</script>

<div class="space-y-3">
  <header class="flex items-baseline justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">{t('Audit log')}</h1>
      <p class="text-xs text-fg3">
        {t('Append-only trail of every mutation.')} {total} {t(total === 1 ? 'entry' : 'entries')}
        {#if (actionFilter || entityTypeFilter || entityIdFilter || textFilter || sinceDate || untilDate)}
          {t('(filtered)')}
        {/if}.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button class="btn" onclick={refresh} disabled={loading}>
          {loading ? t('Loading…') : t('Refresh')}
      </button>
      <button class="btn text-crit border-crit/40 hover:bg-crit/10" onclick={onClearLog}>
        {t('Clear log')}
      </button>
    </div>
  </header>

  <!-- Filter bar — single line on wide screens, wraps gracefully. -->
  <div class="card p-3 flex flex-wrap items-end gap-3">
    <label class="flex flex-col gap-1 text-xs text-fg2">
       <span>{t('Action')}</span>
      <select class="select" bind:value={actionFilter}>
         <option value="">{t('All actions')}</option>
        {#each distinctActions as a}<option value={a}>{a}</option>{/each}
      </select>
    </label>
    <label class="flex flex-col gap-1 text-xs text-fg2">
       <span>{t('Entity type')}</span>
      <select class="select" bind:value={entityTypeFilter}>
         <option value="">{t('All types')}</option>
        {#each distinctEntityTypes as t}<option value={t}>{t}</option>{/each}
      </select>
    </label>
    <label class="flex flex-col gap-1 text-xs text-fg2">
       <span>{t('Entity ID')}</span>
       <input class="input" type="text" placeholder={t('exact match')} bind:value={entityIdFilter} />
    </label>
    <label class="flex flex-col gap-1 text-xs text-fg2 flex-1 min-w-[180px]">
       <span>{t('Search')}</span>
       <input class="input" type="text" placeholder={t('summary or details…')} bind:value={textFilter} />
    </label>
    <label class="flex flex-col gap-1 text-xs text-fg2">
       <span>{t('Since')}</span>
      <input class="input" type="date" bind:value={sinceDate} />
    </label>
    <label class="flex flex-col gap-1 text-xs text-fg2">
       <span>{t('Until')}</span>
      <input class="input" type="date" bind:value={untilDate} />
    </label>
    <button class="btn" onclick={clearFilters}
            disabled={!actionFilter && !entityTypeFilter && !entityIdFilter && !textFilter && !sinceDate && !untilDate}>
       {t('Clear filters')}
    </button>
  </div>

  <div class="card overflow-x-auto">
    <table class="w-full text-sm">
      <thead>
        <tr class="text-left text-xs text-fg2 border-b border-line">
           <th class="px-3 py-2 font-medium w-44">{t('When')}</th>
           <th class="px-3 py-2 font-medium w-28">{t('Action')}</th>
           <th class="px-3 py-2 font-medium w-32">{t('Entity')}</th>
           <th class="px-3 py-2 font-medium">{t('Summary')}</th>
          <th class="px-3 py-2 font-medium w-12"></th>
        </tr>
      </thead>
      <tbody>
        {#each entries as e (e.id)}
          {@const isOpen = expanded.has(e.id)}
          {@const hasDetails = !!e.details_json}
          <tr class="border-b border-line/50 hover:bg-bg2/40">
            <td class="px-3 py-2 align-top whitespace-nowrap font-mono text-xs text-fg2">
              {fmtTs(e.ts)}
            </td>
            <td class="px-3 py-2 align-top">
              <span class="inline-block px-1.5 py-0.5 text-[10px] font-mono uppercase tracking-wide border rounded {actionClass(e.action)}">
                {e.action}
              </span>
            </td>
            <td class="px-3 py-2 align-top">
              <div class="text-xs">{e.entity_type}</div>
              {#if e.entity_id}
                <div class="text-[10px] text-fg3 font-mono truncate" title={e.entity_id}>
                  {e.entity_id}
                </div>
              {/if}
            </td>
            <td class="px-3 py-2 align-top text-fg1">{e.summary}</td>
            <td class="px-3 py-2 align-top text-right">
              {#if hasDetails}
                <button class="text-xs text-accent hover:underline" onclick={() => toggleExpanded(e.id)}>
                   {isOpen ? t('Hide') : t('Details')}
                </button>
              {/if}
            </td>
          </tr>
          {#if isOpen && hasDetails}
            <tr class="border-b border-line/50 bg-bg2/30">
              <td colspan="5" class="px-3 py-2">
                <pre class="text-[11px] text-fg2 font-mono whitespace-pre-wrap leading-snug">{pretty(e.details_json)}</pre>
              </td>
            </tr>
          {/if}
        {:else}
          <tr><td colspan="5" class="px-3 py-6 text-center text-fg3 text-sm">
             {loading ? t('Loading…') : t('No audit entries match your filters.')}
          </td></tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if total > PAGE_SIZE}
    <div class="flex items-center justify-between text-xs text-fg2">
       <span>{t('Page {page} of {pages}', { page: page + 1, pages: lastPage + 1 })}</span>
      <div class="flex items-center gap-1">
         <button class="btn" onclick={() => (page = 0)}             disabled={page === 0}>{t('« First')}</button>
         <button class="btn" onclick={() => (page = page - 1)}      disabled={page === 0}>{t('‹ Prev')}</button>
         <button class="btn" onclick={() => (page = page + 1)}      disabled={page >= lastPage}>{t('Next ›')}</button>
         <button class="btn" onclick={() => (page = lastPage)}      disabled={page >= lastPage}>{t('Last »')}</button>
      </div>
    </div>
  {/if}
</div>
