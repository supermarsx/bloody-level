<script lang="ts">
  // One-click patient-sex selector. Persists immediately to `patients.sex`
  // via update_patient. The value is the *global* per-patient sex used by
  // every flag derivation, every chart band, and every default_ref lookup
  // — never the per-PDF inferred value.
  import * as admin from '$api/records-admin';
  import { toasts } from '../toasts/store.svelte';

  let {
    patientId,
    displayName,
    currentSex,
    dobIso = null,
    onChanged
  } = $props<{
    patientId: string;
    displayName: string;
    currentSex: string;
    dobIso?: string | null;
    onChanged?: (next: admin.PatientSex) => void;
  }>();

  let busy = $state(false);

  const options: { value: admin.PatientSex; label: string; icon: string }[] = [
    { value: 'm', label: 'Male',    icon: '♂' },
    { value: 'f', label: 'Female',  icon: '♀' },
    { value: 'x', label: 'Other',   icon: '⚧' },
    { value: '?', label: 'Unknown', icon: '?' }
  ];

  async function setSex(next: admin.PatientSex) {
    if (next === currentSex || busy) return;
    busy = true;
    try {
      await admin.updatePatient({
        id: patientId,
        display_name: displayName,
        sex: next,
        dob_iso: dobIso
      });
      toasts.success('Sex updated', `${displayName}: ${next}`);
      onChanged?.(next);
    } catch (e) { toasts.error(e); }
    finally { busy = false; }
  }
</script>

<div class="sex-toggle" role="radiogroup" aria-label="Patient sex">
  {#each options as opt}
    {@const active = currentSex === opt.value}
    <button
      type="button"
      role="radio"
      aria-checked={active}
      class="sex-toggle__opt {active ? 'sex-toggle__opt--active' : ''}"
      disabled={busy}
      onclick={() => setSex(opt.value)}
      title="Set sex to {opt.label}"
    >
      <span aria-hidden="true" class="sex-toggle__icon">{opt.icon}</span>
      <span>{opt.label}</span>
    </button>
  {/each}
</div>

<style>
  .sex-toggle {
    display: inline-flex;
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    overflow: hidden;
    background: rgb(var(--bg-2));
  }
  .sex-toggle__opt {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.7rem;
    font-size: 0.75rem;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 0;
    border-right: 1px solid rgb(var(--line));
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease;
  }
  .sex-toggle__opt:last-child { border-right: 0; }
  .sex-toggle__opt:hover:not(:disabled) {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
  }
  .sex-toggle__opt--active {
    background: rgb(var(--accent) / 0.15);
    color: rgb(var(--accent));
    font-weight: 600;
  }
  .sex-toggle__opt:disabled {
    opacity: 0.55;
    cursor: wait;
  }
  .sex-toggle__icon {
    font-size: 0.95rem;
    line-height: 1;
  }
</style>
