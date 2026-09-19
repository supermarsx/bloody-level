<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { t } from '$lib/i18n/index.svelte';

  onMount(() => {
    void goto('/library', { replaceState: true });
  });
</script>

<svelte:head>
  <title>{t('Library')}</title>
</svelte:head>

<div class="card p-6 text-sm text-fg2">
  {t('The Library has moved. Redirecting…')}
</div>
