<script lang="ts">
  import { onMount } from 'svelte';
  import * as admin from '$api/records-admin';
  import { toasts } from '../toasts/store.svelte';

  let {
    rawText,
    open = $bindable(false),
    onLinked = (_: number) => {}
  } = $props<{
    rawText: string;
    open?: boolean;
    onLinked?: (rowsRelinked: number) => void;
  }>();

  let analytes = $state<admin.AnalyteOption[]>([]);
  let query = $state('');
  let busy = $state(false);
  let selectedId = $state<string | null>(null);

  const filtered = $derived.by(() => {
    if (!query.trim()) return analytes.slice(0, 50);
    const q = query.toLowerCase();
    return analytes
      .filter((a) =>
        a.pt_name.toLowerCase().includes(q) ||
        a.id.toLowerCase().includes(q) ||
        (a.panel ?? '').toLowerCase().includes(q) ||
        (a.subsection ?? '').toLowerCase().includes(q)
      )
      .slice(0, 50);
  });

  async function load() {
    try {
      analytes = await admin.listAnalytes();
    } catch (e) {
      toasts.error(e);
    }
  }

  onMount(load);

  async function submit() {
    if (!selectedId) return;
    busy = true;
    try {
      const res = await admin.linkUnmatchedAnalyte({ raw_text: rawText, analyte_id: selectedId });
      toasts.success(
        'Alias added',
        `Linked "${rawText}" → ${selectedId}. ${res.rows_relinked} existing row${res.rows_relinked === 1 ? '' : 's'} updated.`
      );
      onLinked(res.rows_relinked);
      open = false;
    } catch (e) {
      toasts.error(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-40 flex items-center justify-center p-4">
    <!-- Backdrop is a real button for keyboard + a11y. -->
    <button
      type="button"
      class="absolute inset-0 bg-black/40"
      aria-label="Close dialog"
      onclick={() => (open = false)}
    ></button>
    <div
      class="relative card w-full max-w-xl p-4 space-y-3 bg-bg2"
      role="dialog"
      aria-modal="true"
    >
      <div>
        <h2 class="text-sm font-semibold">Link unmatched analyte</h2>
        <p class="text-xs text-fg2 mt-0.5">
          Map this raw name to an ontology entry. The mapping persists as a user alias and
          re-links every existing row that matched the same raw text.
        </p>
      </div>

      <div class="card-tight bg-bg1">
        <div class="text-[10px] uppercase tracking-wide text-fg3">Raw text</div>
        <div class="text-sm font-mono break-all">{rawText}</div>
      </div>

      <input
        type="text"
        placeholder="Search analytes (name, id, panel)…"
        class="block w-full bg-bg1 border border-line rounded-md px-3 py-2 text-sm"
        bind:value={query}
      />

      <div class="card max-h-72 overflow-y-auto divide-y divide-line">
        {#each filtered as a}
          <button
            type="button"
            class="w-full text-left px-3 py-2 hover:bg-bg3 {selectedId === a.id ? 'bg-bg3' : ''}"
            onclick={() => (selectedId = a.id)}
          >
            <div class="flex items-baseline justify-between gap-2">
              <span class="text-sm font-medium truncate">{a.pt_name}</span>
              <span class="text-[10px] font-mono text-fg3">{a.id}</span>
            </div>
            <div class="text-[11px] text-fg3">
              {a.section}{a.subsection ? ` · ${a.subsection}` : ''}{a.panel ? ` · ${a.panel}` : ''}
            </div>
          </button>
        {/each}
        {#if filtered.length === 0}
          <div class="px-3 py-3 text-xs text-fg3 text-center">No matches.</div>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-2">
        <button class="btn" onclick={() => (open = false)} disabled={busy}>Cancel</button>
        <button class="btn-accent" onclick={submit} disabled={!selectedId || busy}>
          {busy ? 'Linking…' : selectedId ? `Link to ${selectedId}` : 'Pick an analyte'}
        </button>
      </div>
    </div>
  </div>
{/if}
