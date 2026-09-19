<script lang="ts">
  import '../app.css';
  import { onMount, onDestroy } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { theme } from '$theme/store.svelte';
  import { ensureThemesRegistered } from '$theme/echarts-themes';
  import { chartPrefs } from '$charts/prefs.svelte';
  import { comparePresets } from '$charts/compare-presets.svelte';
  import { appearance } from '$theme/appearance.svelte';
  import * as auth from '$api/auth';
  import { authEvents } from '$api/auth-events.svelte';
  import { toasts } from '../lib/toasts/store.svelte';
  import { AppError } from '$api/errors';
  import UnlockGate from '$components/unlock-gate.svelte';
  import NavBar from '$components/nav-bar.svelte';
  import Toaster from '$components/toaster.svelte';
  import Splash from '$components/splash.svelte';
  import { isTauri } from '$api/index';
  import { windowTitle } from '$lib/title.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { page } from '$app/stores';

  let { children } = $props();

  let unlocked = $state(false);
  let checking = $state(true);
  let splashVisible = $state(true);
  let splashMessage = $state(t('Initializing…'));
  let unsub: (() => void) | null = null;

  async function checkAuth() {
    try {
      const s = await auth.status();
      unlocked = s.unlocked;
    } catch (e) {
      unlocked = false;
      if (e instanceof AppError && e.kind !== 'locked' && e.kind !== 'not_initialized') {
        toasts.error(e);
      }
    }
    checking = false;
  }

  /**
   * Reveal the OS window after the splash has rendered. The window starts
   * hidden in tauri.conf.json so the user never sees a white-flash blank
   * frame; this call is the moment the splash is on-screen and we're ready
   * for them to look at it.
   */
  async function revealWindow() {
    if (!isTauri()) return;
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const w = getCurrentWindow();
      await w.show();
      await w.setFocus();
    } catch (e) {
      // Non-critical — log only. If show() fails the user just sees their
      // window appear via Tauri's default behavior on the next render.
      // eslint-disable-next-line no-console
      console.warn('[splash] revealWindow failed', e);
    }
  }

  function fadeOutSplash() {
    splashVisible = false;
  }

  async function hydratePreferences() {
    // These settings live inside the encrypted vault. A first attempt can
    // legitimately happen while the unlock gate is showing, so retry after
    // the user unlocks instead of permanently keeping process defaults.
    await Promise.all([chartPrefs.load(), appearance.load(), comparePresets.load()]);
  }

  function setupGlobalErrorHandlers() {
    if (typeof window === 'undefined') return;
    window.addEventListener('unhandledrejection', (e) => {
      toasts.error(e.reason);
    });
    window.addEventListener('error', (e) => {
      toasts.error(e.error ?? new Error(e.message));
    });
  }

  onMount(async () => {
    theme.apply();
    ensureThemesRegistered();
    setupGlobalErrorHandlers();

    // The public demo is a real app route rendered with synthetic data. It
    // must be viewable from the README/docs without requiring a vault or
    // making an auth IPC call, while every normal route remains auth-gated.
    if ($page.url.pathname.startsWith('/demo/')) {
      unlocked = true;
      checking = false;
      splashVisible = false;
      return;
    }

    // When any command fails with `locked`, drop straight back to the gate.
    unsub = authEvents.on((evt) => {
      if (evt.type === 'locked') {
        if (unlocked) {
          toasts.warn(t('Session locked'), t('Re-enter your password to continue.'));
        }
        unlocked = false;
      }
    });

    // Show the splash, then reveal the window — this ordering means the very
    // first frame the user sees is already painted with the splash, never an
    // empty white WebView frame.
    splashMessage = t('Connecting to the vault…');
    await revealWindow();

    splashMessage = t('Checking unlock state…');
    await checkAuth();
    // Hydrate chart + appearance preferences from the settings table so
    // both render correctly on first paint after unlock.
    if (unlocked) await hydratePreferences();

    // Hold the splash a beat longer so the fade reads as intentional rather
    // than a flicker. 220ms matches the CSS opacity transition duration.
    splashMessage = t('Ready');
    setTimeout(fadeOutSplash, 220);
  });

  onDestroy(() => unsub?.());

  // Alt+← / Alt+→ — global history navigation, matches every browser /
  // file-explorer convention. Suppressed inside text inputs so editing
  // doesn't fight with shortcut handling.
  function onGlobalKeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement | null;
    if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
    if (!e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
    if (e.key === 'ArrowLeft' && window.history.length > 1) {
      e.preventDefault();
      window.history.back();
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      window.history.forward();
    }
  }

  // Reflect lock state and a default per-route subject derived from the URL.
  // Specific routes override `subject` via setPageTitle() in their own files
  // (e.g. report/[id] sets the patient + nickname/date).
  $effect(() => { windowTitle.setLocked(!unlocked); });

  // Page-transition animation parameters. Honour the user's reduce-motion
  // preference (both the appearance setting and the OS-level hint) by
  // collapsing every transition to instant. Svelte's transition functions
  // are JS-driven, so the CSS reduce-motion override doesn't reach them.
  function reducedMotion(): boolean {
    if (appearance.reduceMotion) return true;
    if (typeof window !== 'undefined') {
      try { return window.matchMedia('(prefers-reduced-motion: reduce)').matches; }
      catch { /* */ }
    }
    return false;
  }
  // We key page transitions on the route's logical "section" rather than
  // the full pathname, so navigating between e.g. /report/abc and
  // /report/def doesn't re-mount the whole tree (and lose the scroll
  // position on the patient list). Top-level segments only.
  const routeKey = $derived(
    (() => {
      const seg = $page.url.pathname.split('/').filter(Boolean)[0] ?? '';
      return seg;
    })()
  );
  const pageInTransition  = $derived(reducedMotion() ? { duration: 0 } : { y: 8, duration: 220, easing: cubicOut, delay: 60 });
  const pageOutTransition = $derived(reducedMotion() ? { duration: 0 } : { duration: 120, easing: cubicOut });

  $effect(() => {
    const path = $page.url.pathname;
    // Only apply a generic subject for static routes — dynamic detail pages
    // set their own richer subjects, so we leave their value alone.
    const generic: Record<string, string> = {
      '/':         'Dashboard',
      '/ingest':   'Ingest PDFs',
      '/patients': 'Patients',
      '/records':  'Records',
      '/audit':    'Audit log',
      '/compare':  'Compare',
      '/library': 'Library',
      '/settings': 'Settings'
    };
    if (path in generic) {
      windowTitle.setSubject(t(generic[path]));
    }
  });
</script>

<!-- The splash overlays the rest. We render the unlock gate / main app
     immediately *behind* the splash so by the time the splash fades out
     the underlying UI is already laid out — no second flash. -->
{#if !checking}
  {#if !unlocked}
    <UnlockGate onUnlocked={() => { unlocked = true; hydratePreferences(); }} />
  {:else}
    <!-- Plain document scroll — `overscroll-behavior: none` on body kills
         the rubber-band bounce without trapping the document, so child
         re-layouts (modals, charts, tables) can never freeze the scroll.
         The NavBar is already `sticky top-0`, so it pins regardless. -->
    <div class="min-h-screen flex flex-col bg-bg1 text-fg1">
      <NavBar />
      <!-- Keyed by top-level route segment so cross-section navigation
           (Dashboard → Patients → Audit, etc.) gets a fade-up transition,
           but inner navigation that stays inside one section (e.g. one
           report to another) doesn't re-mount the page and lose scroll. -->
      <main class="flex-1 px-4 py-6 overflow-x-clip">
        {#key routeKey}
          <div in:fly={pageInTransition} out:fade={pageOutTransition}>
            {@render children()}
          </div>
        {/key}
      </main>
    </div>
  {/if}
{/if}

<svelte:window onkeydown={onGlobalKeydown} />

<Splash visible={splashVisible} message={splashMessage} />

<Toaster />
