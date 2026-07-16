<script lang="ts">
  import { page } from '$app/stores';
  import { theme } from '$theme/store.svelte';
  import * as auth from '$api/auth';
  import GlobalSearch from './GlobalSearch.svelte';

  // Inline SVG paths so the navbar has no asset dependencies. Each icon is
  // 20×20, single-stroke, 1.6px stroke-width — readable at our 12-line bar.
  const links = [
    { href: '/',         label: 'Dashboard', icon: 'dashboard' },
    { href: '/ingest',   label: 'Ingest',    icon: 'ingest' },
    { href: '/patients', label: 'Patients',  icon: 'patients' },
    { href: '/records',  label: 'Records',   icon: 'records' },
    { href: '/audit',    label: 'Audit',     icon: 'audit' },
    { href: '/compare',  label: 'Compare',   icon: 'compare' },
    { href: '/ontology', label: 'Ontology',  icon: 'ontology' },
    { href: '/settings', label: 'Settings',  icon: 'settings' }
  ] as const;

  async function lock() {
    await auth.lock();
    location.href = '/';
  }
</script>

<header class="border-b border-line bg-bg1 sticky top-0 z-20">
  <div class="flex items-center justify-between px-4 h-12">
    <div class="flex items-center gap-4">
      <!-- App icon only — no wordmark to keep the bar tight. The icon is
           a stripped-down version of the same waveform used as the OS app
           icon; clicking it routes home. -->
      <a href="/" class="shrink-0 inline-flex items-center" aria-label="blevel-tracker — home" title="blevel-tracker">
        <svg viewBox="0 0 96 96" width="22" height="22" fill="none" aria-hidden="true">
          <defs>
            <linearGradient id="navGrad" x1="0" y1="0" x2="1" y2="1">
              <stop offset="0%" stop-color="#5b8bff"/>
              <stop offset="100%" stop-color="#8e5bff"/>
            </linearGradient>
          </defs>
          <circle cx="48" cy="48" r="44" stroke="url(#navGrad)" stroke-width="3" opacity="0.30"/>
          <path d="M14 56 L26 48 L36 60 L48 32 L58 50 L70 38 L82 44"
                stroke="url(#navGrad)" stroke-width="5"
                stroke-linecap="round" stroke-linejoin="round"/>
          <circle cx="48" cy="32" r="4" fill="url(#navGrad)"/>
        </svg>
      </a>

      <nav class="flex items-center gap-1">
        {#each links as l}
          {@const active = $page.url.pathname === l.href || ($page.url.pathname.startsWith(l.href) && l.href !== '/')}
          <a
            href={l.href}
            class="nav-tab {active ? 'nav-tab--active' : ''}"
            title={l.label}
            aria-label={l.label}
          >
            <svg viewBox="0 0 20 20" width="16" height="16" fill="none" stroke="currentColor"
                 stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              {#if l.icon === 'dashboard'}
                <!-- 4 tiles -->
                <rect x="3" y="3" width="6" height="6" rx="1.2"/>
                <rect x="11" y="3" width="6" height="6" rx="1.2"/>
                <rect x="3" y="11" width="6" height="6" rx="1.2"/>
                <rect x="11" y="11" width="6" height="6" rx="1.2"/>
              {:else if l.icon === 'ingest'}
                <!-- Document with downward arrow -->
                <path d="M5 3 h7 l3 3 v11 a1 1 0 0 1 -1 1 H5 a1 1 0 0 1 -1 -1 V4 a1 1 0 0 1 1 -1 z"/>
                <path d="M10 9 v5 m-2.5 -2 l2.5 2.5 l2.5 -2.5"/>
              {:else if l.icon === 'patients'}
                <!-- Two figures -->
                <circle cx="7.5" cy="7" r="2.6"/>
                <path d="M3 16 c 0 -2.5 2 -4 4.5 -4 s 4.5 1.5 4.5 4"/>
                <circle cx="14" cy="8" r="2"/>
                <path d="M11.5 17 c 0 -1.8 1.4 -3 3 -3 s 3 1.2 3 3"/>
              {:else if l.icon === 'records'}
                <!-- Stacked rows -->
                <rect x="3" y="4" width="14" height="3" rx="0.6"/>
                <rect x="3" y="9" width="14" height="3" rx="0.6"/>
                <rect x="3" y="14" width="14" height="3" rx="0.6"/>
              {:else if l.icon === 'audit'}
                <!-- Magnifier over lines -->
                <circle cx="9" cy="9" r="5"/>
                <path d="M13 13 l3 3"/>
                <path d="M7.5 9 h3 m-3 -1.5 h3"/>
              {:else if l.icon === 'compare'}
                <!-- Two intersecting trend lines -->
                <path d="M3 14 L7 10 L11 12 L17 5"/>
                <path d="M3 7 L8 12 L13 8 L17 14"/>
              {:else if l.icon === 'ontology'}
                <!-- Stylised book / atlas pages -->
                <path d="M4 4 H9 a2 2 0 0 1 2 2 V17 H6 a2 2 0 0 1 -2 -2 z"/>
                <path d="M16 4 H11 a2 2 0 0 0 -2 2 V17 H14 a2 2 0 0 0 2 -2 z"/>
                <path d="M5.5 8 H8.5 M5.5 11 H8.5 M11.5 8 H14.5 M11.5 11 H14.5"/>
              {:else if l.icon === 'settings'}
                <!-- Gear -->
                <circle cx="10" cy="10" r="2.5"/>
                <path d="M10 2 v2 M10 16 v2 M2 10 h2 M16 10 h2
                         M4.2 4.2 l1.4 1.4 M14.4 14.4 l1.4 1.4
                         M4.2 15.8 l1.4 -1.4 M14.4 5.6 l1.4 -1.4"/>
              {/if}
            </svg>
            <span class="nav-tab__label">{l.label}</span>
          </a>
        {/each}
      </nav>
    </div>

    <div class="flex items-center gap-2">
      <GlobalSearch />
      <!-- Icon-only theme cycler. The full mode label moved to the title
           tooltip so the navbar stays compact; the emoji alone signals
           current state at a glance. -->
      <button
        class="icon-btn"
        onclick={() => theme.cycle()}
        title="Theme: {theme.mode} · click to cycle (light → dark → system)"
        aria-label="Cycle theme (current: {theme.mode})"
      >
        {#if theme.mode === 'system'}
          <!-- Half-filled circle for "follow system" — reads as both
               sun + moon rolled together. -->
          <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
            <circle cx="10" cy="10" r="6" fill="none" stroke="currentColor" stroke-width="1.6"/>
            <path d="M10 4 a6 6 0 0 1 0 12 z" fill="currentColor"/>
          </svg>
        {:else if theme.resolved === 'dark'}
          <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
            <path d="M14 11.5 A6 6 0 1 1 8.5 6 a5 5 0 0 0 5.5 5.5z"
                  fill="currentColor"/>
          </svg>
        {:else}
          <svg viewBox="0 0 20 20" width="16" height="16" fill="none"
               stroke="currentColor" stroke-width="1.6" stroke-linecap="round" aria-hidden="true">
            <circle cx="10" cy="10" r="3.5"/>
            <path d="M10 2 v2 M10 16 v2 M2 10 h2 M16 10 h2
                     M4.2 4.2 l1.4 1.4 M14.4 14.4 l1.4 1.4
                     M4.2 15.8 l1.4 -1.4 M14.4 5.6 l1.4 -1.4"/>
          </svg>
        {/if}
      </button>
      <!-- Icon-only lock (closed padlock). Click re-locks the DB and the
           layout's auth gate kicks the user back to the unlock screen. -->
      <button class="icon-btn" onclick={lock} title="Lock the database" aria-label="Lock">
        <svg viewBox="0 0 20 20" width="16" height="16" fill="none"
             stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="4" y="9" width="12" height="8" rx="1.5"/>
          <path d="M7 9 V6 a3 3 0 0 1 6 0 V9"/>
        </svg>
      </button>
    </div>
  </div>
</header>

<style>
  .nav-tab {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.6rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    color: rgb(var(--fg-2));
    transition: background 160ms ease, color 160ms ease, padding 220ms cubic-bezier(0.4, 0, 0.2, 1);
  }
  .nav-tab:hover {
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-1));
  }
  .nav-tab--active {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
  }
  .nav-tab--active :global(svg) {
    color: rgb(var(--accent));
  }
  /* Label slides + fades when collapsing to icon-only mode. The active
     tab keeps its label even at narrow widths so the user always knows
     where they are; everything else gets a hover tooltip via `title`. */
  .nav-tab__label {
    display: inline-block;
    overflow: hidden;
    max-width: 8rem;
    opacity: 1;
    transform: translateX(0);
    transition: max-width 220ms cubic-bezier(0.4, 0, 0.2, 1),
                opacity   180ms ease,
                transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
    white-space: nowrap;
  }
  /* Collapse threshold — below 1024px the inactive tabs go icon-only.
     The active tab keeps its label by overriding inside the @media block. */
  @media (max-width: 1023px) {
    .nav-tab:not(.nav-tab--active) .nav-tab__label {
      max-width: 0;
      opacity: 0;
      transform: translateX(-4px);
      margin-left: -0.4rem; /* swallow the gap when collapsed */
    }
    .nav-tab:not(.nav-tab--active) {
      padding-left: 0.45rem;
      padding-right: 0.45rem;
    }
  }

  /* Square icon-only action button — used for the theme cycler and the
     lock toggle on the right of the navbar. Sized to match `.nav-tab`
     line height so the navbar reads as a single horizontal band. */
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.75rem;
    height: 1.75rem;
    border: 1px solid rgb(var(--line));
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-2));
    border-radius: 0.375rem;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .icon-btn:hover {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--accent) / 0.5);
  }
  .icon-btn:focus-visible {
    outline: none;
    border-color: rgb(var(--accent));
    box-shadow: 0 0 0 2px rgb(var(--accent) / 0.3);
  }
</style>
