<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { quintOut, cubicOut } from 'svelte/easing';
  import { flip } from 'svelte/animate';
  import { toasts, type Toast } from '../toasts/store.svelte';
  import { appearance } from '$theme/appearance.svelte';
  import Icon from '$components/icon.svelte';

  function classFor(kind: Toast['kind']): string {
    switch (kind) {
      case 'success': return 'border-ok    text-ok';
      case 'warn':    return 'border-warn  text-warn';
      case 'error':   return 'border-crit  text-crit';
      default:        return 'border-line  text-fg2';
    }
  }

  // Honour the user's reduce-motion preference (and the OS-level hint).
  // Svelte transitions are JS-driven, so the CSS reduce-motion override
  // doesn't reach them — we manually flip durations to zero.
  function reducedMotion(): boolean {
    if (appearance.reduceMotion) return true;
    if (typeof window !== 'undefined') {
      try { return window.matchMedia('(prefers-reduced-motion: reduce)').matches; }
      catch { /* */ }
    }
    return false;
  }
  const flyIn  = $derived(reducedMotion() ? { duration: 0 } : { x: 32, y: 8, duration: 260, easing: quintOut });
  const fadeOut = $derived(reducedMotion() ? { duration: 0 } : { duration: 180, easing: cubicOut });
  const flipMs = $derived(reducedMotion() ? 0 : 220);
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-md pointer-events-none">
  {#each toasts.toasts as t (t.id)}
    <div class="card p-3 border-l-4 {classFor(t.kind)} shadow-lg pointer-events-auto bg-bg2"
         in:fly={flyIn}
         out:fade={fadeOut}
         animate:flip={{ duration: flipMs, easing: quintOut }}>
      <div class="flex items-start justify-between gap-3">
        <div class="flex-1 min-w-0">
          <div class="text-sm font-semibold text-fg1 truncate">{t.title}</div>
          {#if t.message}
            <div class="text-xs text-fg2 mt-0.5 break-words whitespace-pre-wrap">{t.message}</div>
          {/if}

          {#if t.context && (t.context.stage || t.context.path || t.context.section || t.context.line != null || t.context.patient)}
            <dl class="mt-2 grid grid-cols-[max-content_1fr] gap-x-2 gap-y-0.5 text-[11px] text-fg3">
              {#if t.context.stage}
                <dt class="font-medium">stage</dt>
                <dd class="font-mono">{t.context.stage}</dd>
              {/if}
              {#if t.context.path}
                <dt class="font-medium">file</dt>
                <dd class="font-mono truncate" title={t.context.path}>{t.context.path}</dd>
              {/if}
              {#if t.context.section}
                <dt class="font-medium">section</dt>
                <dd class="font-mono truncate">{t.context.section}</dd>
              {/if}
              {#if t.context.line != null}
                <dt class="font-medium">line</dt>
                <dd class="font-mono">{t.context.line}</dd>
              {/if}
              {#if t.context.patient}
                <dt class="font-medium">patient</dt>
                <dd class="truncate">{t.context.patient}</dd>
              {/if}
            </dl>
          {/if}

          {#if t.context?.hints && t.context.hints.length > 0}
            <ul class="mt-1.5 text-[11px] text-fg2 list-disc list-inside space-y-0.5">
              {#each t.context.hints as h}
                <li>{h}</li>
              {/each}
            </ul>
          {/if}

          {#if t.code || t.command}
            <div class="text-[10px] text-fg3 mt-1.5 font-mono">
              {t.command ?? ''}{t.command && t.code ? ' · ' : ''}{t.code ?? ''}
            </div>
          {/if}

          {#if t.kind === 'error' && t.detail}
            <details class="mt-1">
              <summary class="text-[10px] text-fg3 cursor-pointer select-none">Detail</summary>
              <pre class="text-[10px] text-fg3 mt-1 whitespace-pre-wrap font-mono">{t.detail}</pre>
            </details>
          {/if}

          {#if t.context?.breadcrumbs && t.context.breadcrumbs.length > 0}
            <details class="mt-1">
              <summary class="text-[10px] text-fg3 cursor-pointer select-none">Breadcrumbs</summary>
              <ol class="text-[10px] text-fg3 mt-1 list-decimal list-inside space-y-0.5">
                {#each t.context.breadcrumbs as b}
                  <li class="font-mono">{b}</li>
                {/each}
              </ol>
            </details>
          {/if}
        </div>
        <div class="flex items-center gap-1 shrink-0">
          {#if t.retry}
            <button
              class="text-xs text-accent hover:underline"
              onclick={() => { t.retry?.(); toasts.dismiss(t.id); }}
            >Retry</button>
          {/if}
          <button
            class="text-xs text-fg3 hover:text-fg1 px-1"
            onclick={() => toasts.dismiss(t.id)}
            aria-label="Dismiss"
          ><Icon name="x" size={14} /></button>
        </div>
      </div>
    </div>
  {/each}
</div>
