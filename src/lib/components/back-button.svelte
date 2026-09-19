<script lang="ts">
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';

  // Reusable browser-style "back" affordance. Walks the navigation history
  // when there's something to walk back to, otherwise falls through to a
  // sensible fallback route (Dashboard by default). The fallback matters
  // when the user opened the app on a deep link — there's no history to
  // pop, so a literal `history.back()` would no-op.
  let {
    fallback = '/',
    label = 'Back'
  } = $props<{
    fallback?: string;
    label?: string;
  }>();

  function go() {
    if (typeof window !== 'undefined' && window.history.length > 1) {
      window.history.back();
    } else {
      goto(fallback);
    }
  }
</script>

<button
  type="button"
  class="back-btn"
  onclick={go}
  title="{t(label)} (Alt+←)"
  aria-label={t(label)}
>
  <svg viewBox="0 0 20 20" width="14" height="14" fill="none" stroke="currentColor"
       stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M11.5 4.5 L6 10 L11.5 15.5"/>
    <path d="M6 10 H16"/>
  </svg>
  <span>{t(label)}</span>
</button>

<style>
  .back-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.2rem 0.55rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 1px solid transparent;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
    cursor: pointer;
  }
  .back-btn:hover {
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--line));
  }
  .back-btn:focus-visible {
    outline: 2px solid rgb(var(--accent));
    outline-offset: 1px;
  }
</style>
