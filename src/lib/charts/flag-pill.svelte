<script lang="ts">
  import Icon, { type IconName } from '$components/icon.svelte';
  let { flag } = $props<{ flag: string | null | undefined }>();

  const meta = $derived.by(() => {
    switch (flag) {
      case 'low':           return { cls: 'pill-warn', label: 'Low',           icon: 'arrow-down' as IconName };
      case 'high':          return { cls: 'pill-warn', label: 'High',          icon: 'arrow-up' as IconName };
      case 'critical_low':  return { cls: 'pill-crit', label: 'Critical Low',  icon: 'warning' as IconName };
      case 'critical_high': return { cls: 'pill-crit', label: 'Critical High', icon: 'warning' as IconName };
      case 'abnormal_qual': return { cls: 'pill-warn', label: 'Abnormal',      icon: 'warning' as IconName };
      case 'normal':        return { cls: 'pill-ok',   label: 'Normal',        icon: 'check' as IconName };
      default:              return { cls: 'pill-muted',label: '—',             icon: null };
    }
  });
</script>

<span class={meta.cls}>
  {#if meta.icon}<Icon name={meta.icon} size={12} />{/if}
  {meta.label}
</span>
