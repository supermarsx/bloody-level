<script lang="ts">
  // Modal editor for creating or updating an analyte. Mode is inferred
  // from the presence of `existing`. The editing of `categorical_tiers`
  // and `cycle_phases` JSON happens through a raw-text area for v1 —
  // these fields are complex enough that bespoke editors deserve their
  // own pass; the textarea validates as JSON before save.
  import * as analyteApi from '$api/analyte-info';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { toasts } from '../toasts/store.svelte';

  let {
    open = $bindable(false),
    /** When set the dialog is in "edit" mode and the form is seeded from this. */
    existing = null as analyteApi.AnalyteOntologyEntry | null,
    /** Suggested section list for the dropdown. */
    knownSections = [] as string[],
    onSaved
  } = $props<{
    open: boolean;
    existing?: analyteApi.AnalyteOntologyEntry | null;
    knownSections?: string[];
    onSaved?: (id: string) => void;
  }>();

  type Form = {
    id: string;
    pt_name: string;
    section: string;
    subsection: string;
    panel: string;
    loinc: string;
    method_annotation: string;
    expected_units: string;       // comma-separated for the form
    default_ref_json: string;
    categorical_tiers_json: string;
    cycle_phases_json: string;
    sex_dependent: boolean;
    age_dependent: boolean;
    cycle_dependent: boolean;
    is_qualitative: boolean;
    is_derived: boolean;
    is_panel_header: boolean;
    paired_value: boolean;
    description: string;
    high_means: string;
    low_means: string;
    unit_notes: string;
    aliases: string;              // comma-separated
  };

  let form = $state<Form>(emptyForm());
  let saving = $state(false);
  let err = $state<string | null>(null);
  let infoLoaded = $state(false);

  function emptyForm(): Form {
    return {
      id: '', pt_name: '', section: '', subsection: '', panel: '',
      loinc: '', method_annotation: '', expected_units: '',
      default_ref_json: '', categorical_tiers_json: '', cycle_phases_json: '',
      sex_dependent: false, age_dependent: false, cycle_dependent: false,
      is_qualitative: false, is_derived: false, is_panel_header: false,
      paired_value: false,
      description: '', high_means: '', low_means: '', unit_notes: '',
      aliases: ''
    };
  }

  // When the dialog opens, populate the form from `existing` (edit) or
  // reset to blank (create).
  $effect(() => {
    if (!open) { infoLoaded = false; err = null; return; }
    if (!existing) {
      form = emptyForm();
      infoLoaded = true;
      return;
    }
    // For an edit we need the full info (description / high_means / etc.) —
    // the overview row only carries booleans.
    (async () => {
      try {
        const info = await analyteApi.get(existing!.id);
        form = {
          id: info.id,
          pt_name: info.pt_name,
          section: info.section,
          subsection: info.subsection ?? '',
          panel: info.panel ?? '',
          loinc: info.loinc ?? '',
          method_annotation: info.method_annotation ?? '',
          expected_units: info.expected_units.join(', '),
          default_ref_json: info.default_ref_json ?? '',
          categorical_tiers_json: info.categorical_tiers_json ?? '',
          cycle_phases_json: info.cycle_phases_json ?? '',
          sex_dependent: !!existing!.sex_dependent,
          age_dependent: !!existing!.age_dependent,
          cycle_dependent: !!existing!.cycle_dependent,
          is_qualitative: info.is_qualitative,
          is_derived: info.is_derived,
          is_panel_header: !!existing!.is_panel_header,
          paired_value: !!existing!.paired_value,
          description: info.description ?? '',
          high_means: info.high_means ?? '',
          low_means: info.low_means ?? '',
          unit_notes: info.unit_notes ?? '',
          aliases: info.aliases.join(', ')
        };
        infoLoaded = true;
      } catch (e) {
        err = (e as Error)?.message ?? String(e);
        infoLoaded = true;
      }
    })();
  });

  function trim<T extends string | null>(s: T): T {
    if (s == null) return s;
    const t = (s as string).trim();
    return (t.length === 0 ? null : t) as T;
  }

  function splitList(s: string): string[] {
    return s.split(',').map((x) => x.trim()).filter(Boolean);
  }

  function tryParseJson(s: string, label: string): string | null {
    const t = s.trim();
    if (t.length === 0) return null;
    try { JSON.parse(t); }
    catch (err) { throw new Error(`${label} is not valid JSON: ${(err as Error).message}`); }
    return t;
  }

  async function save() {
    err = null;
    let default_ref_json: string | null;
    let categorical_tiers_json: string | null;
    let cycle_phases_json: string | null;
    try {
      default_ref_json       = tryParseJson(form.default_ref_json, 'Default ref');
      categorical_tiers_json = tryParseJson(form.categorical_tiers_json, 'Categorical tiers');
      cycle_phases_json      = tryParseJson(form.cycle_phases_json, 'Cycle phases');
    } catch (e) {
      err = (e as Error).message;
      return;
    }

    const args: analyteApi.AnalyteWriteArgs = {
      id: form.id.trim(),
      pt_name: form.pt_name.trim(),
      section: form.section.trim() || 'PATOLOGIA QUÍMICA',
      subsection:        trim(form.subsection),
      panel:             trim(form.panel),
      loinc:             trim(form.loinc),
      method_annotation: trim(form.method_annotation),
      expected_units: splitList(form.expected_units),
      default_ref_json,
      categorical_tiers_json,
      cycle_phases_json,
      sex_dependent: form.sex_dependent,
      age_dependent: form.age_dependent,
      cycle_dependent: form.cycle_dependent,
      is_qualitative: form.is_qualitative,
      is_derived: form.is_derived,
      is_panel_header: form.is_panel_header,
      paired_value: form.paired_value,
      description: trim(form.description),
      high_means: trim(form.high_means),
      low_means: trim(form.low_means),
      unit_notes: trim(form.unit_notes),
      aliases: splitList(form.aliases)
    };

    saving = true;
    try {
      if (existing) {
        if (existing.source === 'seed') {
          const ok = await ask(
            'You are editing a bundled (seed) analyte. The next "Reload from seed" will overwrite your edits.\n\nUser-created analytes are immune to reloads. Continue saving the edit anyway?',
            { title: 'Editing seed analyte', kind: 'warning' }
          );
          if (!ok) { saving = false; return; }
        }
        await analyteApi.updateAnalyte(args);
        toasts.success('Analyte updated', args.pt_name);
      } else {
        await analyteApi.createAnalyte(args);
        toasts.success('Analyte created', args.pt_name);
      }
      onSaved?.(args.id);
      open = false;
    } catch (e) {
      err = (e as Error)?.message ?? String(e);
    } finally {
      saving = false;
    }
  }

  function close() { open = false; }
</script>

{#if open}
  <div class="fixed inset-0 z-30 bg-black/55 grid place-items-center p-4 overflow-y-auto">
    <button type="button" class="absolute inset-0 cursor-default"
            aria-label="Close dialog" onclick={close}></button>

    <div class="relative card p-5 w-full max-w-3xl space-y-4 max-h-[92vh] overflow-y-auto"
         role="dialog" aria-modal="true" tabindex="-1">
      <header class="flex items-baseline justify-between gap-3">
        <h2 class="text-base font-semibold flex items-center gap-2">
          <span class="ed-pill" aria-hidden="true">
            {existing ? 'EDITING' : 'NEW'}
          </span>
          {existing ? `Edit ${existing.pt_name}` : 'Create analyte'}
        </h2>
        {#if existing && existing.source === 'seed'}
          <span class="text-[11px] text-warn">Bundled — edits will be lost on next reload</span>
        {:else if existing}
          <span class="text-[11px] text-fg3">User-created · safe across reloads</span>
        {/if}
      </header>

      {#if !infoLoaded}
        <p class="text-sm text-fg2">Loading…</p>
      {:else}
        <!-- ── Identity ── -->
        <fieldset class="space-y-3">
          <legend class="ed-legend">Identity</legend>
          <div class="ed-grid">
            <label class="edit-field">
              <span class="edit-field__label">ID <span class="edit-field__opt">— a-z, 0-9, _</span></span>
              <input class="edit-field__input"
                     placeholder="e.g. amilase_pancreatica"
                     bind:value={form.id}
                     disabled={!!existing} />
              <span class="edit-field__hint">Slug used for results.analyte_id, /analyte/&lt;id&gt; URLs, alias keys. Immutable after creation.</span>
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Display name (pt_name)</span>
              <input class="edit-field__input"
                     placeholder="e.g. Amílase Pancreática"
                     bind:value={form.pt_name} />
              <span class="edit-field__hint">Auto-registered as an alias too.</span>
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Section</span>
              <input class="edit-field__input" list="ed-sections"
                     placeholder="e.g. PATOLOGIA QUÍMICA"
                     bind:value={form.section} />
              <datalist id="ed-sections">
                {#each knownSections as s}<option value={s}></option>{/each}
              </datalist>
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Subsection <span class="edit-field__opt">— optional</span></span>
              <input class="edit-field__input" placeholder="e.g. ENZIMOLOGIA"
                     bind:value={form.subsection} />
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Panel <span class="edit-field__opt">— optional</span></span>
              <input class="edit-field__input" placeholder="e.g. hepatic"
                     bind:value={form.panel} />
            </label>
            <label class="edit-field">
              <span class="edit-field__label">LOINC <span class="edit-field__opt">— optional</span></span>
              <input class="edit-field__input" placeholder="e.g. 1798-8" bind:value={form.loinc} />
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Method annotation <span class="edit-field__opt">— optional</span></span>
              <input class="edit-field__input"
                     placeholder="e.g. Quimioluminescência" bind:value={form.method_annotation} />
            </label>
            <label class="edit-field">
              <span class="edit-field__label">Expected units <span class="edit-field__opt">— comma-separated</span></span>
              <input class="edit-field__input" placeholder="g/dl, mg/dL" bind:value={form.expected_units} />
            </label>
          </div>
        </fieldset>

        <!-- ── Reference data ── -->
        <fieldset class="space-y-3">
          <legend class="ed-legend">Reference data</legend>
          <label class="edit-field">
            <span class="edit-field__label">Default ref (JSON)</span>
            <textarea class="edit-field__input font-mono text-xs" rows="3"
                      placeholder={'{"m":[13.0,17.0],"f":[12.0,15.0]}\nor\n{"all":[null,500]}'}
                      bind:value={form.default_ref_json}></textarea>
            <span class="edit-field__hint">Sex-keyed `{`m`}`/`{`f`}` or universal `{`all`}`. Numbers or `null` for open-ended.</span>
          </label>
          <label class="edit-field">
            <span class="edit-field__label">Categorical tiers (JSON) <span class="edit-field__opt">— optional</span></span>
            <textarea class="edit-field__input font-mono text-xs" rows="3"
                      placeholder={'[{"label":"Deficiência","max":10},{"label":"Suficiência","min":30,"max":100}]'}
                      bind:value={form.categorical_tiers_json}></textarea>
          </label>
          <label class="edit-field">
            <span class="edit-field__label">Cycle phases (JSON) <span class="edit-field__opt">— optional</span></span>
            <textarea class="edit-field__input font-mono text-xs" rows="3"
                      placeholder={'{"follicular":[0.2,1.5],"luteal":[1.7,27]}'}
                      bind:value={form.cycle_phases_json}></textarea>
          </label>
        </fieldset>

        <!-- ── Flags ── -->
        <fieldset class="space-y-1">
          <legend class="ed-legend">Flags</legend>
          <div class="ed-flag-grid">
            <label class="ed-check"><input type="checkbox" bind:checked={form.sex_dependent}> sex-dependent</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.age_dependent}> age-dependent</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.cycle_dependent}> cycle-dependent</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.is_qualitative}> qualitative</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.is_derived}> derived</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.is_panel_header}> panel header</label>
            <label class="ed-check"><input type="checkbox" bind:checked={form.paired_value}> paired value</label>
          </div>
        </fieldset>

        <!-- ── Clinical context ── -->
        <fieldset class="space-y-3">
          <legend class="ed-legend">Clinical context</legend>
          <label class="edit-field">
            <span class="edit-field__label">Description</span>
            <textarea class="edit-field__input" rows="3" bind:value={form.description}></textarea>
          </label>
          <label class="edit-field">
            <span class="edit-field__label">When elevated</span>
            <textarea class="edit-field__input" rows="3" bind:value={form.high_means}></textarea>
          </label>
          <label class="edit-field">
            <span class="edit-field__label">When reduced</span>
            <textarea class="edit-field__input" rows="3" bind:value={form.low_means}></textarea>
          </label>
          <label class="edit-field">
            <span class="edit-field__label">Unit notes</span>
            <textarea class="edit-field__input" rows="2" bind:value={form.unit_notes}></textarea>
          </label>
        </fieldset>

        <!-- ── Aliases ── -->
        <fieldset>
          <legend class="ed-legend">Aliases <span class="edit-field__opt">— comma-separated</span></legend>
          <input class="edit-field__input mt-1"
                 placeholder="e.g. AMY, Amylase, Alpha-Amylase"
                 bind:value={form.aliases}>
          <span class="edit-field__hint">
            New aliases are tagged user-source so they survive `Reload from seed`. The display name is auto-aliased.
          </span>
        </fieldset>

        {#if err}
          <div class="rounded-md border border-crit/40 bg-crit/10 p-3 text-xs text-crit">{err}</div>
        {/if}

        <footer class="flex justify-end gap-2 pt-1 border-t border-line">
          <button class="btn" onclick={close}>Cancel</button>
          <button class="btn-accent" disabled={saving || !form.id || !form.pt_name} onclick={save}>
            {saving ? 'Saving…' : (existing ? 'Save changes' : 'Create analyte')}
          </button>
        </footer>
      {/if}
    </div>
  </div>
{/if}

<style>
  .ed-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.1rem 0.45rem;
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-size: 0.6rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    border-radius: 0.3rem;
  }
  .ed-legend {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 600;
    color: rgb(var(--fg-3));
  }
  .ed-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.6rem 1rem;
  }
  @media (min-width: 600px) {
    .ed-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  }
  .ed-flag-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.3rem 1rem;
  }
  @media (min-width: 600px) {
    .ed-flag-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  }
  .ed-check {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    color: rgb(var(--fg-2));
    cursor: pointer;
  }
  /* Field primitives shared with the patient edit panel — exported via
     :global so they're reachable inside this component too. */
  :global(.edit-field) { display: flex; flex-direction: column; gap: 0.3rem; min-width: 0; }
  :global(.edit-field__label) { font-size: 0.7rem; font-weight: 500; color: rgb(var(--fg-2)); }
  :global(.edit-field__opt)   { color: rgb(var(--fg-3)); font-style: italic; font-weight: 400; }
  :global(.edit-field__input) {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
    color: rgb(var(--fg-1));
    transition: border-color 120ms ease, box-shadow 120ms ease;
    width: 100%;
  }
  :global(.edit-field__input:hover) { border-color: rgb(var(--fg-3)); }
  :global(.edit-field__input:focus) {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18);
  }
  :global(.edit-field__hint) { font-size: 0.65rem; color: rgb(var(--fg-3)); line-height: 1.3; }
</style>
