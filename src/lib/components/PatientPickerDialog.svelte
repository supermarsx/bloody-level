<script lang="ts">
  import { onMount } from 'svelte';
  import { listPatients, type PatientSummary } from '$api/reports';
  import { toasts } from '../toasts/store.svelte';

  let {
    excludeId,
    title = 'Pick a patient',
    confirmLabel = 'Select',
    open = $bindable(false),
    onPick = (_: PatientSummary) => {}
  } = $props<{
    excludeId?: string;
    title?: string;
    confirmLabel?: string;
    open?: boolean;
    onPick?: (p: PatientSummary) => void;
  }>();

  let patients = $state<PatientSummary[]>([]);
  let query = $state('');
  let selected = $state<PatientSummary | null>(null);

  async function load() {
    try {
      const all = await listPatients();
      patients = excludeId ? all.filter((p) => p.id !== excludeId) : all;
    } catch (e) { toasts.error(e); }
  }

  onMount(load);

  const filtered = $derived.by(() => {
    if (!query.trim()) return patients;
    const q = query.toLowerCase();
    return patients.filter((p) =>
      p.display_name.toLowerCase().includes(q) || p.id.toLowerCase().includes(q)
    );
  });
</script>

{#if open}
  <div class="fixed inset-0 z-40 flex items-center justify-center p-4">
    <button
      type="button"
      class="absolute inset-0 bg-black/40"
      aria-label="Close"
      onclick={() => (open = false)}
    ></button>
    <div class="relative card w-full max-w-md p-4 space-y-3 bg-bg2" role="dialog" aria-modal="true">
      <h2 class="text-sm font-semibold">{title}</h2>

      <input
        type="text"
        placeholder="Search…"
        class="block w-full bg-bg1 border border-line rounded-md px-3 py-2 text-sm"
        bind:value={query}
      />

      <div class="card max-h-72 overflow-y-auto divide-y divide-line">
        {#each filtered as p}
          <button
            type="button"
            class="w-full text-left px-3 py-2 hover:bg-bg3 {selected?.id === p.id ? 'bg-bg3' : ''}"
            onclick={() => (selected = p)}
          >
            <div class="flex items-baseline justify-between gap-2">
              <span class="text-sm font-medium truncate">{p.display_name}</span>
              <span class="text-[10px] font-mono text-fg3">{p.sex}</span>
            </div>
            <div class="text-[11px] text-fg3">
              {p.report_count} report{p.report_count === 1 ? '' : 's'} · <span class="font-mono">{p.id}</span>
            </div>
          </button>
        {/each}
        {#if filtered.length === 0}
          <div class="px-3 py-3 text-xs text-fg3 text-center">No matches.</div>
        {/if}
      </div>

      <div class="flex items-center justify-end gap-2">
        <button class="btn" onclick={() => (open = false)}>Cancel</button>
        <button
          class="btn-accent"
          disabled={!selected}
          onclick={() => { if (selected) { onPick(selected); open = false; } }}
        >{confirmLabel}</button>
      </div>
    </div>
  </div>
{/if}
