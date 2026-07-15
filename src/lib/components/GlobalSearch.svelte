<script lang="ts">
  import { goto } from '$app/navigation';
  import { globalSearch, type SearchResults } from '$api/search';

  let query = $state('');
  let results = $state<SearchResults | null>(null);
  let open = $state(false);
  let loading = $state(false);
  let lastFired = 0;

  // Flat list of hits in display order so ↑/↓ can move through them.
  const flatHits = $derived(
    results
      ? [...results.patients, ...results.analytes, ...results.reports]
      : []
  );
  let activeIdx = $state(-1);

  // 200ms debounce — typeahead while user types but not on every keystroke.
  let debounceHandle: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    const q = query.trim();
    if (debounceHandle) clearTimeout(debounceHandle);
    if (q.length === 0) {
      results = null;
      open = false;
      return;
    }
    const fired = ++lastFired;
    debounceHandle = setTimeout(async () => {
      loading = true;
      try {
        const r = await globalSearch(q);
        // Skip stale responses if the user kept typing.
        if (fired === lastFired) {
          results = r;
          open = true;
          activeIdx = -1;
        }
      } catch { /* swallow — search is non-critical */ }
      finally { loading = false; }
    }, 200);
  });

  function go(href: string) {
    goto(href);
    query = '';
    results = null;
    open = false;
  }

  function onKeydown(e: KeyboardEvent) {
    if (!open || flatHits.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIdx = (activeIdx + 1) % flatHits.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIdx = activeIdx <= 0 ? flatHits.length - 1 : activeIdx - 1;
    } else if (e.key === 'Enter') {
      const target = flatHits[activeIdx >= 0 ? activeIdx : 0];
      if (target) go(target.href);
    } else if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<div class="relative">
  <input
    type="search"
    placeholder="Search… (patients · analytes · reports)"
    class="search w-72"
    bind:value={query}
    onfocus={() => { if (results) open = true; }}
    onblur={() => setTimeout(() => (open = false), 150)}
    onkeydown={onKeydown}
  />
  {#if open && results}
    <div class="absolute right-0 mt-1 w-96 max-h-[70vh] overflow-y-auto card shadow-lg z-30">
      {#if results.patients.length === 0 && results.analytes.length === 0 && results.reports.length === 0}
        <div class="p-3 text-xs text-fg3">No matches{loading ? '…' : ''}</div>
      {/if}
      {#snippet section(title: string, hits: typeof flatHits, baseIdx: number)}
        {#if hits.length > 0}
          <div class="px-3 pt-2 pb-1 text-[10px] uppercase tracking-wide text-fg3">{title}</div>
          {#each hits as h, i}
            {@const idx = baseIdx + i}
            <button
              type="button"
              class="w-full text-left px-3 py-1.5 hover:bg-bg2 border-b border-line/30 {idx === activeIdx ? 'bg-bg2' : ''}"
              onmousedown={(e) => { e.preventDefault(); go(h.href); }}
            >
              <div class="text-sm">{h.label}</div>
              {#if h.sub}<div class="text-[11px] text-fg3">{h.sub}</div>{/if}
            </button>
          {/each}
        {/if}
      {/snippet}
      {@render section('Patients', results.patients, 0)}
      {@render section('Analytes', results.analytes, results.patients.length)}
      {@render section('Reports', results.reports, results.patients.length + results.analytes.length)}
    </div>
  {/if}
</div>
