<script lang="ts">
  let { flag } = $props<{ flag: string | null | undefined }>();

  const meta = $derived.by(() => {
    switch (flag) {
      case 'low':           return { cls: 'pill-warn', label: 'Low',          icon: '↓' };
      case 'high':          return { cls: 'pill-warn', label: 'High',         icon: '↑' };
      case 'critical_low':  return { cls: 'pill-crit', label: 'Critical Low', icon: '⚠' };
      case 'critical_high': return { cls: 'pill-crit', label: 'Critical High',icon: '⚠' };
      case 'abnormal_qual': return { cls: 'pill-warn', label: 'Abnormal',     icon: '!' };
      case 'normal':        return { cls: 'pill-ok',   label: 'Normal',       icon: '✓' };
      default:              return { cls: 'pill-muted',label: '—',            icon: '' };
    }
  });
</script>

<span class={meta.cls}>
  {#if meta.icon}<span class="mr-1">{meta.icon}</span>{/if}
  {meta.label}
</span>
