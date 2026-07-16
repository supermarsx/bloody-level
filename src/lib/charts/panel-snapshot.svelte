<script lang="ts">
  import FlagPill from './flag-pill.svelte';
  import DeltaBadge from './delta-badge.svelte';
  import { formatNumber } from '$format/numbers';
  import { prettyUnit } from '$format/units';

  let {
    name,
    value,
    unit,
    flag,
    previous,
    direction = 'neutral',
    method = ''
  } = $props<{
    name: string;
    value: number | null;
    unit: string | null;
    flag: string | null;
    previous: number | null;
    direction?: 'higher-bad' | 'lower-bad' | 'neutral';
    method?: string;
  }>();
</script>

<div class="card p-3 flex flex-col gap-1.5">
  <div class="flex items-baseline justify-between gap-2">
    <span class="text-xs font-medium text-fg2 truncate">{name}</span>
    <FlagPill {flag} />
  </div>
  <div class="flex items-baseline gap-1.5">
    <span class="text-2xl font-semibold tabular-nums">{formatNumber(value)}</span>
    <span class="text-xs text-fg3">{prettyUnit(unit)}</span>
  </div>
  <div class="flex items-center justify-between text-[11px]">
    <DeltaBadge current={value} {previous} {direction} />
    {#if method}<span class="text-fg3 truncate" title={method}>{method}</span>{/if}
  </div>
</div>
