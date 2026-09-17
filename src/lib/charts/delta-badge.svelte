<script lang="ts">
  import { formatNumber, formatPercent } from '$format/numbers';
  import Icon from '$components/icon.svelte';

  let {
    current,
    previous,
    direction = 'neutral'
  } = $props<{
    current: number | null;
    previous: number | null;
    direction?: 'higher-bad' | 'lower-bad' | 'neutral';
  }>();

  const delta = $derived.by(() => {
    if (current == null || previous == null) return null;
    const abs = current - previous;
    const pct = previous === 0 ? 0 : (abs / previous) * 100;
    return { abs, pct, dir: abs > 0 ? 'up' : abs < 0 ? 'down' : 'flat' };
  });

  const colorClass = $derived.by(() => {
    if (!delta || delta.dir === 'flat') return 'delta-none';
    if (direction === 'neutral') return 'text-fg2';
    if (direction === 'higher-bad') return delta.dir === 'up' ? 'delta-up' : 'delta-down';
    return delta.dir === 'down' ? 'delta-up' : 'delta-down';
  });
</script>

{#if delta}
  <span class="inline-flex items-center gap-1 text-xs font-medium {colorClass}">
    {#if delta.dir === 'up'}<Icon name="arrow-up" size={12} />{:else if delta.dir === 'down'}<Icon name="arrow-down" size={12} />{:else}<Icon name="arrow-right" size={12} />{/if}
    <span>{formatNumber(delta.abs)}</span>
    <span class="text-fg3">({formatPercent(delta.pct)})</span>
  </span>
{:else}
  <span class="text-xs text-fg3">—</span>
{/if}
