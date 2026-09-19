<script lang="ts">
  import { onMount } from 'svelte';
  import { listOntologyEntries, get as getInfo, deleteAnalyte, type AnalyteOntologyEntry, type AnalyteInfo } from '$api/analyte-info';
  import * as admin from '$api/records-admin';
  import { toasts } from '../../lib/toasts/store.svelte';
  import { parseTiers, formatTierRange } from '$format/tiers';
  import AnalyteEditorDialog from '$components/analyte-editor-dialog.svelte';
  import ReferenceCard from '$components/reference-card.svelte';
  import { openUrl } from '$api/shell';
  import { ask } from '@tauri-apps/plugin-dialog';
  import Icon from '$components/icon.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { localizeAnalyteText } from '$lib/i18n/analytes';

  let entries = $state<AnalyteOntologyEntry[]>([]);
  let loading = $state(true);
  let reloading = $state(false);

  // ── Filters ────────────────────────────────────────────────────────────
  let filter = $state('');
  let sectionFilter = $state<string>('');
  let dataFilter = $state<'' | 'has_default_ref' | 'no_default_ref' | 'has_tiers' | 'has_phases' | 'incomplete' | 'unused' | 'panel_headers'>('');
  /** Panel-header rows (e.g. `Ionograma sérico`) clutter the list — they
   *  exist in the ontology so the parser can recognise the section heading
   *  on the PDF, not because they're measurable. Hide by default. */
  let includeHeaders = $state(false);
  let sortKey = $state<'pt_name' | 'section' | 'results'>('section');
  let sortDir = $state<'asc' | 'desc'>('asc');

  // ── Detail panel ───────────────────────────────────────────────────────
  let selectedId = $state<string | null>(null);
  let selectedInfo = $state<AnalyteInfo | null>(null);
  let loadingDetail = $state(false);

  // ── Editor dialog ──────────────────────────────────────────────────────
  let editorOpen = $state(false);
  let editorTarget = $state<AnalyteOntologyEntry | null>(null);

  async function openLoinc(code: string) {
    try {
      await openUrl(`https://loinc.org/${encodeURIComponent(code)}/`);
    } catch (e) {
      toasts.error(e);
    }
  }

  function openCreate() {
    editorTarget = null;
    editorOpen = true;
  }
  function openEdit(e: AnalyteOntologyEntry) {
    editorTarget = e;
    editorOpen = true;
  }
  async function onSaved(id: string) {
    await refresh();
    await selectAnalyte(id);
  }

  async function onDelete(e: AnalyteOntologyEntry) {
    if (e.source !== 'user') {
      toasts.warn(t('Cannot delete'), t('Seed-bundled analytes can only be removed by editing the JSON file.'));
      return;
    }
    if (e.result_count > 0) {
      toasts.warn(
        t('Cannot delete'),
        t(e.result_count === 1 ? '{count} stored result row still reference this analyte. Re-link them first.' : '{count} stored result rows still reference this analyte. Re-link them first.', { count: e.result_count })
      );
      return;
    }
    const ok = await ask(
      t('Delete analyte "{name}" ({id})?\n\nUser-created — this is irreversible.', { name: e.pt_name, id: e.id }),
      { title: t('Delete analyte'), kind: 'warning' }
    );
    if (!ok) return;
    try {
      await deleteAnalyte(e.id);
      toasts.success(t('Analyte deleted'), e.pt_name);
      if (selectedId === e.id) { selectedId = null; selectedInfo = null; }
      await refresh();
    } catch (err) { toasts.error(err); }
  }

  async function refresh() {
    loading = true;
    try { entries = await listOntologyEntries(); }
    catch (e) { toasts.error(e); }
    finally { loading = false; }
  }
  onMount(refresh);

  async function reloadOntology() {
    // Same warning surface as Settings ▸ Library — re-install overwrites
    // every seed-source row's metadata. User-created analytes are safe.
    const ok = await ask(
      t('Reload the analyte Library from the bundled seed?\n\n• Re-installs every seed-bundled analyte\'s descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n• User-created analytes (source=user) and user-added aliases are preserved.\n• Existing parsed results are not touched. Run "Re-parse all" on Records afterwards if you want stored rows to pick up new Library fields.'),
      { title: t('Reload Library'), kind: 'warning' }
    );
    if (!ok) return;
    reloading = true;
    try {
      const r = await admin.reloadOntology();
      toasts.success(t('Library reloaded'), t('{count} analytes installed.', { count: r.analytes_installed }));
      await refresh();
      if (selectedId) await selectAnalyte(selectedId);
    } catch (e) { toasts.error(e); }
    finally { reloading = false; }
  }

  async function selectAnalyte(id: string) {
    selectedId = id;
    loadingDetail = true;
    selectedInfo = null;
    try { selectedInfo = await getInfo(id); }
    catch (e) { toasts.error(e); }
    finally { loadingDetail = false; }
  }

  const sections = $derived.by(() => {
    const set = new Set<string>();
    for (const e of entries) set.add(e.section);
    return [...set].sort();
  });

  const filtered = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    let rows = entries.filter((e) => {
      if (q) {
        const hit = e.pt_name.toLowerCase().includes(q) || e.id.toLowerCase().includes(q);
        if (!hit) return false;
      }
      if (sectionFilter && e.section !== sectionFilter) return false;
      // Panel-header gate — the dedicated `panel_headers` filter overrides
      // the toggle so the user can still see them on demand.
      if (dataFilter === 'panel_headers') {
        if (!e.is_panel_header) return false;
      } else if (!includeHeaders && e.is_panel_header) {
        return false;
      }
      switch (dataFilter) {
        case 'has_default_ref': if (!e.default_ref_json) return false; break;
        case 'no_default_ref':  if (e.default_ref_json) return false; break;
        case 'has_tiers':       if (!e.categorical_tiers_json) return false; break;
        case 'has_phases':      if (!e.cycle_phases_json) return false; break;
        case 'incomplete':
          if (e.has_description && e.has_high_means && e.has_low_means) return false;
          break;
        case 'unused':          if (e.result_count > 0) return false; break;
      }
      return true;
    });
    rows.sort((a, b) => {
      const dir = sortDir === 'asc' ? 1 : -1;
      switch (sortKey) {
        case 'pt_name': return a.pt_name.localeCompare(b.pt_name) * dir;
        case 'section':
          // primary by section, then subsection, then name
          return ((a.section + (a.subsection ?? '') + a.pt_name)
                    .localeCompare(b.section + (b.subsection ?? '') + b.pt_name)) * dir;
        case 'results': return (a.result_count - b.result_count) * dir;
      }
    });
    return rows;
  });

  const stats = $derived.by(() => ({
    total:        entries.length,
    sections:     sections.length,
    withDefault:  entries.filter((e) => e.default_ref_json).length,
    withTiers:    entries.filter((e) => e.categorical_tiers_json).length,
    withPhases:   entries.filter((e) => e.cycle_phases_json).length,
    sexKeyed:     entries.filter((e) => e.sex_dependent).length,
    // Panel headers don't need high/low descriptions — exclude them from
    // the "incomplete" count so the warning isn't a false positive.
    incomplete:   entries.filter((e) => !e.is_panel_header && !(e.has_description && e.has_high_means && e.has_low_means)).length,
    unused:       entries.filter((e) => e.result_count === 0).length
  }));

  function toggleSort(k: typeof sortKey) {
    if (sortKey === k) sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    else { sortKey = k; sortDir = 'asc'; }
  }

  function describeDefaultRef(json: string | null): string {
    if (!json) return '—';
    try {
      const o = JSON.parse(json) as Record<string, [number | null, number | null]>;
      const fmt = (k: string) => {
        const v = o[k];
        if (!v) return null;
        const lo = v[0] ?? null, hi = v[1] ?? null;
        if (lo != null && hi != null) return `${k}: ${lo}–${hi}`;
        if (hi != null)               return `${k}: <${hi}`;
        if (lo != null)               return `${k}: ≥${lo}`;
        return null;
      };
      return ['m', 'f', 'all'].map(fmt).filter(Boolean).join(' · ') || '—';
    } catch { return '—'; }
  }
</script>

<div class="space-y-4">
  <!-- ─── Header ─── -->
  <div class="flex items-end justify-between gap-3 flex-wrap">
    <div>
      <h1 class="text-xl font-semibold">{t('Library')}</h1>
      <p class="text-xs text-fg2">
        {t('The analyte Library powering every flag derivation, reference band, and chart context card.')} {t('{count} entries across {sections} sections — sourced from the bundled seed and refreshed in-place by', { count: stats.total, sections: stats.sections })} <em>{t('Reload')}</em>.
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button class="btn" disabled={reloading} onclick={reloadOntology}>
         {reloading ? t('Reloading…') : t('Reload from seed')}
      </button>
      <button class="btn-accent" onclick={openCreate}>
        <Icon name="plus" size={14} />
         {t('New analyte')}
      </button>
    </div>
  </div>

  <!-- ─── KPI strip ─── -->
  <section class="kpi-grid">
    <div class="kpi"><span class="kpi__label">{t('Analytes')}</span><span class="kpi__value">{stats.total}</span></div>
    <div class="kpi"><span class="kpi__label">{t('Sex-keyed refs')}</span><span class="kpi__value">{stats.sexKeyed}</span></div>
    <div class="kpi"><span class="kpi__label">{t('With default_ref')}</span><span class="kpi__value">{stats.withDefault}</span></div>
    <div class="kpi"><span class="kpi__label">{t('Categorical tiers')}</span><span class="kpi__value">{stats.withTiers}</span></div>
    <div class="kpi"><span class="kpi__label">{t('Cycle phases')}</span><span class="kpi__value">{stats.withPhases}</span></div>
    <div class="kpi {stats.incomplete > 0 ? 'kpi--warn' : ''}"><span class="kpi__label">{t('Incomplete')}</span><span class="kpi__value">{stats.incomplete}</span></div>
    <div class="kpi"><span class="kpi__label">{t('Never used')}</span><span class="kpi__value">{stats.unused}</span></div>
  </section>

  <!-- ─── Filters ─── -->
  <section class="filter-row">
    <input
      type="search"
      placeholder={t('Filter by name or id…')}
      class="ftr ftr--grow"
      bind:value={filter}
    />
    <select class="ftr" bind:value={sectionFilter}>
      <option value="">{t('Any section')}</option>
      {#each sections as s}<option value={s}>{s}</option>{/each}
    </select>
    <select class="ftr" bind:value={dataFilter}>
      <option value="">{t('Any data state')}</option>
      <option value="has_default_ref">{t('Has default_ref')}</option>
      <option value="no_default_ref">{t('Missing default_ref')}</option>
      <option value="has_tiers">{t('Has categorical tiers')}</option>
      <option value="has_phases">{t('Has cycle phases')}</option>
      <option value="incomplete">{t('Incomplete descriptions')}</option>
      <option value="unused">{t('Never seen in any report')}</option>
      <option value="panel_headers">{t('Section headers only')}</option>
    </select>
    <label class="text-xs text-fg2 flex items-center gap-1.5 cursor-pointer">
      <input type="checkbox" bind:checked={includeHeaders} />
       <span>{t('Include section headers')}</span>
    </label>
    <span class="text-xs text-fg2">{t('Showing {shown} of {total}', { shown: filtered.length, total: entries.length })}</span>
  </section>

  {#if loading}
     <div class="card p-6 text-sm text-fg2">{t('Loading…')}</div>
  {:else}
    <!-- ─── Two-pane layout: list on left, detail on right ─── -->
    <section class="ontology-layout grid grid-cols-1 lg:grid-cols-[1.1fr_1fr] gap-4">
      <!-- LIST -->
      <div class="card overflow-x-auto">
        <table class="w-full text-sm">
          <thead class="text-fg2 text-xs uppercase tracking-wide">
            <tr class="border-b border-line">
              <th class="px-3 py-2 text-left cursor-pointer" onclick={() => toggleSort('section')}>
                 {t('Section')} {#if sortKey === 'section'}<Icon name={sortDir === 'asc' ? 'arrow-up' : 'arrow-down'} size={12} />{/if}
              </th>
              <th class="px-3 py-2 text-left cursor-pointer" onclick={() => toggleSort('pt_name')}>
                 {t('Analyte')} {#if sortKey === 'pt_name'}<Icon name={sortDir === 'asc' ? 'arrow-up' : 'arrow-down'} size={12} />{/if}
              </th>
               <th class="px-3 py-2 text-left">{t('Default ref')}</th>
               <th class="px-3 py-2 text-left">{t('Flags')}</th>
              <th class="px-3 py-2 text-right cursor-pointer" onclick={() => toggleSort('results')}>
                 {t('Used')} {#if sortKey === 'results'}<Icon name={sortDir === 'asc' ? 'arrow-up' : 'arrow-down'} size={12} />{/if}
              </th>
               <th class="px-3 py-2 text-left">{t('Source')}</th>
              <th class="px-3 py-2 text-left w-20"></th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as e (e.id)}
              <tr class="ont-row {selectedId === e.id ? 'ont-row--active' : ''}"
                  onclick={() => selectAnalyte(e.id)}>
                <td class="px-3 py-1.5 text-xs text-fg2">
                  {e.section}{#if e.subsection}<div class="text-[10px] text-fg3">{e.subsection}</div>{/if}
                </td>
                <td class="px-3 py-1.5">
                  <div class="text-sm font-medium">{e.pt_name}</div>
                  <div class="text-[10px] text-fg3 font-mono">{e.id}</div>
                </td>
                <td class="px-3 py-1.5 text-xs text-fg2 max-w-[18rem] truncate" title={describeDefaultRef(e.default_ref_json)}>
                  {describeDefaultRef(e.default_ref_json)}
                </td>
                <td class="px-3 py-1.5 text-xs">
                  <div class="flex flex-wrap gap-1">
                    {#if e.is_panel_header}<span class="ont-tag ont-tag--accent" title={t('Section heading on the PDF — not a measurable test')}>{t('section header')}</span>{/if}
                    {#if e.sex_dependent}<span class="ont-tag ont-tag--accent">{t('sex')}</span>{/if}
                    {#if e.cycle_dependent}<span class="ont-tag ont-tag--accent">{t('cycle')}</span>{/if}
                    {#if e.age_dependent}<span class="ont-tag ont-tag--accent">{t('age')}</span>{/if}
                    {#if e.is_qualitative}<span class="ont-tag">{t('qual')}</span>{/if}
                    {#if e.is_derived}<span class="ont-tag">{t('derived')}</span>{/if}
                    {#if e.categorical_tiers_json}<span class="ont-tag ont-tag--ok">{t('tiers')}</span>{/if}
                    {#if e.cycle_phases_json}<span class="ont-tag ont-tag--ok">{t('phases')}</span>{/if}
                    {#if !(e.has_description && e.has_high_means && e.has_low_means)}
                       <span class="ont-tag ont-tag--warn" title={t('Missing one of: description / high_means / low_means')}>{t('incomplete')}</span>
                    {/if}
                  </div>
                </td>
                <td class="px-3 py-1.5 text-right tabular-nums {e.result_count === 0 ? 'text-fg3' : ''}">
                  {e.result_count}
                </td>
                <td class="px-3 py-1.5 text-xs">
                  <span class="ont-tag {e.source === 'user' ? 'ont-tag--ok' : ''}"
                         title={e.source === 'user' ? t('Created via the UI — survives reload_from_seed.') : t('Bundled with the app — `Reload from seed` refreshes this row from the JSON.')}>
                    {e.source}
                  </span>
                </td>
                <td class="px-3 py-1.5">
                  <div class="flex items-center gap-1"
                       role="toolbar"
                       tabindex="-1"
                        aria-label={t('Row actions')}
                       onclick={(ev) => ev.stopPropagation()}
                       onkeydown={(ev) => ev.stopPropagation()}>
                    <button type="button" class="ont-icon"
                            title={e.source === 'user'
                               ? t('Edit analyte')
                               : t('Edit bundled (seed) analyte — your changes will be overwritten by next Reload')}
                             aria-label={t('Edit')}
                            onclick={() => openEdit(e)}>
                      <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                           stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <path d="M4 16 L4 13 L13 4 L16 7 L7 16 z"/>
                        <path d="M11 6 L14 9"/>
                      </svg>
                    </button>
                    {#if e.source === 'user'}
                      <button type="button" class="ont-icon ont-icon--danger"
                              title={t('Delete analyte')}
                              aria-label={t('Delete')}
                              onclick={() => onDelete(e)}>
                        <svg viewBox="0 0 20 20" width="13" height="13" fill="none" stroke="currentColor"
                             stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                          <path d="M5 6 H15 M8 6 V4 H12 V6 M6 6 L7 16 a1 1 0 0 0 1 1 H12 a1 1 0 0 0 1 -1 L14 6"/>
                          <path d="M9 9 V14 M11 9 V14"/>
                        </svg>
                      </button>
                    {/if}
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- DETAIL PANEL -->
      <aside class="ontology-detail card p-4 space-y-3" aria-label={t('Selected analyte details')}>
        {#if !selectedId}
          <p class="text-sm text-fg2">{t('Select an analyte from the list to inspect its full Library entry.')}</p>
        {:else if loadingDetail || !selectedInfo}
          <p class="text-sm text-fg2">{t('Loading…')}</p>
        {:else}
          {@const info = selectedInfo}
          {@const tiers = info.categorical_tiers_json ? parseTiers(info.categorical_tiers_json) : []}
          <header class="space-y-1">
            <div class="flex items-baseline justify-between gap-2 flex-wrap">
              <h2 class="text-lg font-semibold">{info.pt_name}</h2>
              <a class="text-xs text-accent hover:underline inline-flex items-center gap-1" href={`/analyte/${info.id}`}>{t('Open analyte page')} <Icon name="arrow-right" size={12} /></a>
            </div>
            <div class="text-xs text-fg3 flex items-center gap-1.5 flex-wrap">
              <span class="font-mono">{info.id}</span>
              {#if info.section}<span>·</span><span>{info.section}</span>{/if}
              {#if info.subsection}<span>·</span><span>{info.subsection}</span>{/if}
              {#if info.panel}<span>·</span><span class="ont-tag">{info.panel}</span>{/if}
              {#if info.loinc}
                <span>·</span>
                <button
                  type="button"
                  class="cursor-pointer font-mono text-accent hover:underline"
                  onclick={() => openLoinc(info.loinc!)}
                  title={t('Open this code in the default browser')}
                >LOINC {info.loinc} <Icon name="external" size={12} /></button>
              {/if}
            </div>
            {#if info.method_annotation}
              <p class="text-[11px] text-fg3 font-mono">{info.method_annotation}</p>
            {/if}
            <div class="flex flex-wrap gap-1 pt-1">
              {#if info.sex_dependent}<span class="ont-tag ont-tag--accent">{t('sex-dependent')}</span>{/if}
              {#if info.cycle_dependent}<span class="ont-tag ont-tag--accent">{t('cycle-dependent')}</span>{/if}
              {#if info.age_dependent}<span class="ont-tag ont-tag--accent">{t('age-dependent')}</span>{/if}
              {#if info.is_qualitative}<span class="ont-tag">{t('qualitative')}</span>{/if}
              {#if info.is_derived}<span class="ont-tag">{t('derived')}</span>{/if}
            </div>
          </header>

          {#if info.expected_units.length > 0}
            <section>
              <h3 class="ont-h3">{t('Expected units')}</h3>
              <div class="flex flex-wrap gap-1">
                {#each info.expected_units as u}<span class="ont-tag ont-tag--mono">{u}</span>{/each}
              </div>
            </section>
          {/if}

          <!-- Compact "applicable reference" card. Shows both sex-keyed
               variants when the analyte is sex-dependent so reviewers
               can compare the male / female ranges side-by-side. -->
          <ReferenceCard
            info={info}
            patientSex={info.sex_dependent ? 'f' : 'all'}
            unit={info.expected_units[0] ?? ''}
          />

          {#if info.default_ref_json}
            <section>
              <h3 class="ont-h3">{t('Default reference range')}</h3>
              <pre class="ont-pre">{JSON.stringify(JSON.parse(info.default_ref_json), null, 2)}</pre>
            </section>
          {/if}

          {#if tiers.length > 0}
            <section>
              <h3 class="ont-h3">{t('Categorical tiers')}</h3>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-1">
                {#each tiers as t}
                  <div class="ont-tier"><span class="ont-tier__label">{t.label}</span><span class="ont-tier__range">{formatTierRange(t)}</span></div>
                {/each}
              </div>
            </section>
          {/if}

          {#if info.cycle_phases_json}
            <section>
              <h3 class="ont-h3">{t('Cycle-phase reference')}</h3>
              <pre class="ont-pre">{JSON.stringify(JSON.parse(info.cycle_phases_json), null, 2)}</pre>
            </section>
          {/if}

          {#if info.description}
            <section><h3 class="ont-h3">{t('Description')}</h3><p class="text-sm leading-relaxed">{localizeAnalyteText(info.id, 'description', info.description)}</p></section>
          {/if}
          {#if info.high_means}
            <section><h3 class="ont-h3 text-crit">{t('When elevated')}</h3><p class="text-sm leading-relaxed">{localizeAnalyteText(info.id, 'high_means', info.high_means)}</p></section>
          {/if}
          {#if info.low_means}
            <section><h3 class="ont-h3 text-warn">{t('When reduced')}</h3><p class="text-sm leading-relaxed">{localizeAnalyteText(info.id, 'low_means', info.low_means)}</p></section>
          {/if}
          {#if info.unit_notes}
            <section><h3 class="ont-h3 text-accent">{t('Unit notes')}</h3><p class="text-sm leading-relaxed">{localizeAnalyteText(info.id, 'unit_notes', info.unit_notes)}</p></section>
          {/if}

          {#if info.aliases.length > 0}
            <section>
              <h3 class="ont-h3">{t('Aliases ({count})', { count: info.aliases.length })}</h3>
              <div class="flex flex-wrap gap-1">
                {#each info.aliases as a}<span class="ont-tag ont-tag--mono">{a}</span>{/each}
              </div>
            </section>
          {/if}

          {#if info.is_panel_header}
            <section class="rounded-md border border-accent/40 bg-accent/10 p-3 text-xs text-accent">
              <strong>{t('Section header')}</strong> — {t('present in the Library so the parser can recognise this line on the PDF and skip it. It does not carry its own value; the analyte page renders an info-only card pointing to the panel’s measurable members.')}
            </section>
          {:else if !info.description && !info.high_means && !info.low_means && !info.unit_notes}
            <section class="rounded-md border border-warn/40 bg-warn/10 p-3 text-xs text-warn">
              {t('This entry has no description / high / low / unit-notes. Add them to the bundled seed')} (<span class="font-mono">ontology/analytes.seed.json</span>) {t('and click')} <em>{t('Reload from seed')}</em>.
            </section>
          {/if}
        {/if}
      </aside>
    </section>
  {/if}
</div>

<AnalyteEditorDialog
  bind:open={editorOpen}
  existing={editorTarget}
  knownSections={sections}
  onSaved={onSaved}
/>

<style>
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.4rem;
  }
  @media (min-width: 640px)  { .kpi-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); } }
  @media (min-width: 960px)  { .kpi-grid { grid-template-columns: repeat(7, minmax(0, 1fr)); } }
  .kpi {
    display: inline-flex;
    flex-direction: column;
    padding: 0.4rem 0.6rem;
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    min-width: 0;
  }
  .kpi--warn { border-color: rgb(var(--warn) / 0.5); background: rgb(var(--warn) / 0.06); }
  .kpi__label { font-size: 0.65rem; color: rgb(var(--fg-3)); }
  .kpi__value { font-size: 1rem; font-weight: 600; font-variant-numeric: tabular-nums; }

  .filter-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    padding: 0.4rem 0.7rem;
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
  }
  .ftr {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    padding: 0.3rem 0.55rem;
    font-size: 0.8rem;
    color: rgb(var(--fg-1));
  }
  .ftr:focus { outline: none; border-color: rgb(var(--accent)); box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18); }
  .ftr--grow { flex: 1 1 18rem; }

  .ont-row { border-bottom: 1px solid rgb(var(--line) / 0.5); cursor: pointer; transition: background 100ms ease; }
  .ont-row:hover { background: rgb(var(--bg-3) / 0.5); }
  .ont-row--active { background: rgb(var(--accent) / 0.10); }
  .ont-row--active:hover { background: rgb(var(--accent) / 0.14); }

  /* The list can be much taller than the viewport. Keep the selected entry
     beside it while the document scrolls, but give the detail pane its own
     bounded scroll when the entry itself is long. `overflow-x: clip` on the
     app main region keeps this sticky positioning relative to the document. */
  .ontology-layout { align-items: start; }
  .ontology-detail {
    position: sticky;
    top: 4rem;
    align-self: start;
    max-height: calc(100vh - 5rem);
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-gutter: stable;
  }
  @media (max-width: 1023px) {
    .ontology-detail { position: static; max-height: none; overflow-y: visible; }
  }

  .ont-tag {
    display: inline-flex;
    padding: 0.05rem 0.4rem;
    border-radius: 9999px;
    font-size: 0.6rem;
    font-weight: 500;
    color: rgb(var(--fg-2));
    background: rgb(var(--bg-3));
    border: 1px solid rgb(var(--line));
    letter-spacing: 0.02em;
  }
  .ont-tag--mono { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.65rem; }
  .ont-tag--accent { color: rgb(var(--accent)); background: rgb(var(--accent) / 0.10); border-color: rgb(var(--accent) / 0.4); }
  .ont-tag--ok    { color: rgb(var(--ok));    background: rgb(var(--ok)    / 0.10); border-color: rgb(var(--ok)    / 0.4); }
  .ont-tag--warn  { color: rgb(var(--warn));  background: rgb(var(--warn)  / 0.10); border-color: rgb(var(--warn)  / 0.4); }

  .ont-h3 {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    color: rgb(var(--fg-2));
    margin-bottom: 0.3rem;
  }
  .ont-pre {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    padding: 0.5rem 0.7rem;
    font-size: 0.75rem;
    overflow-x: auto;
  }
  .ont-tier {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 0.25rem 0.55rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.35rem;
    font-size: 0.75rem;
  }
  .ont-tier__label { color: rgb(var(--fg-1)); font-weight: 500; }
  .ont-tier__range { color: rgb(var(--fg-3)); font-variant-numeric: tabular-nums; }

  /* Compact action icon used in the row's last column. */
  .ont-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.6rem;
    height: 1.6rem;
    border: 1px solid transparent;
    border-radius: 0.35rem;
    background: transparent;
    color: rgb(var(--fg-3));
    cursor: pointer;
    transition: background 120ms ease, border-color 120ms ease, color 120ms ease;
  }
  .ont-icon:hover {
    background: rgb(var(--bg-2));
    color: rgb(var(--accent));
    border-color: rgb(var(--accent) / 0.4);
  }
  .ont-icon--danger:hover {
    color: rgb(var(--crit));
    border-color: rgb(var(--crit) / 0.4);
    background: rgb(var(--crit) / 0.08);
  }
</style>
