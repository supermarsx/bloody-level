<script lang="ts">
  export type IconName =
    | 'arrow-down'
    | 'arrow-left'
    | 'arrow-right'
    | 'arrow-up'
    | 'ban'
    | 'book'
    | 'chart'
    | 'check'
    | 'chevron-down'
    | 'chevron-up'
    | 'copy'
    | 'dashboard'
    | 'database'
    | 'dot'
    | 'download'
    | 'external'
    | 'eye'
    | 'eye-off'
    | 'female'
    | 'file'
    | 'filter'
    | 'flask'
    | 'gender'
    | 'grip'
    | 'info'
    | 'key'
    | 'layers'
    | 'library'
    | 'lock'
    | 'male'
    | 'minmax'
    | 'monitor'
    | 'mouse'
    | 'moon'
    | 'palette'
    | 'pin'
    | 'plus'
    | 'receipt'
    | 'save'
    | 'scale'
    | 'settings'
    | 'shield'
    | 'sun'
    | 'tag'
    | 'target'
    | 'trash'
    | 'trend-up'
    | 'unlock'
    | 'warning'
    | 'x';

  let {
    name,
    size = 16,
    strokeWidth = 1.8,
    title = null
  } = $props<{
    name: IconName;
    size?: number;
    strokeWidth?: number;
    title?: string | null;
  }>();
</script>

<svg
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  role={title ? 'img' : undefined}
  aria-hidden={title ? undefined : 'true'}
>
  {#if title}<title>{title}</title>{/if}
  {#if name === 'arrow-down'}
    <path d="M12 4v15m-5-5 5 5 5-5" />
  {:else if name === 'arrow-left'}
    <path d="M20 12H5m5-5-5 5 5 5" />
  {:else if name === 'arrow-right'}
    <path d="M4 12h15m-5-5 5 5-5 5" />
  {:else if name === 'arrow-up'}
    <path d="M12 20V5m-5 5 5-5 5 5" />
  {:else if name === 'ban'}
    <circle cx="12" cy="12" r="8.5" />
    <path d="m6 6 12 12" />
  {:else if name === 'book' || name === 'library'}
    <path d="M5 4h6a2 2 0 0 1 2 2v14H7a2 2 0 0 1-2-2V4Z" />
    <path d="M19 4h-6a2 2 0 0 0-2 2v14h6a2 2 0 0 0 2-2V4Z" />
    <path d="M8 8h3m-3 3h3m5-3h-3m3 3h-3" />
  {:else if name === 'chart' || name === 'trend-up'}
    <path d="M4 19V5m0 14h16" />
    <path d="m6 15 4-4 3 2 5-6" />
    <path d="M15 7h3v3" />
  {:else if name === 'check'}
    <path d="m5 12 4 4L19 6" />
  {:else if name === 'chevron-down'}
    <path d="m6 9 6 6 6-6" />
  {:else if name === 'chevron-up'}
    <path d="m6 15 6-6 6 6" />
  {:else if name === 'copy'}
    <rect x="8" y="8" width="11" height="12" rx="1.5" />
    <path d="M16 8V5.5A1.5 1.5 0 0 0 14.5 4h-9A1.5 1.5 0 0 0 4 5.5v10A1.5 1.5 0 0 0 5.5 17H8" />
  {:else if name === 'dashboard'}
    <rect x="4" y="4" width="6" height="6" rx="1" />
    <rect x="14" y="4" width="6" height="6" rx="1" />
    <rect x="4" y="14" width="6" height="6" rx="1" />
    <rect x="14" y="14" width="6" height="6" rx="1" />
  {:else if name === 'database'}
    <ellipse cx="12" cy="6" rx="7" ry="3" />
    <path d="M5 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3V6m-14 6v6c0 1.7 3.1 3 7 3s7-1.3 7-3v-6" />
  {:else if name === 'dot'}
    <circle cx="12" cy="12" r="3" fill="currentColor" stroke="none" />
  {:else if name === 'download'}
    <path d="M12 4v11m-4-4 4 4 4-4M5 20h14" />
  {:else if name === 'external'}
    <path d="M14 5h5v5m0-5-8 8" />
    <path d="M18 13v4a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4" />
  {:else if name === 'eye'}
    <path d="M3 12s3.2-5 9-5 9 5 9 5-3.2 5-9 5-9-5-9-5Z" />
    <circle cx="12" cy="12" r="2.2" />
  {:else if name === 'eye-off'}
    <path d="m4 4 16 16M10.6 6.9A10 10 0 0 1 12 7c5.8 0 9 5 9 5a16 16 0 0 1-3.1 3.3M6.2 7.7C4.1 9.1 3 12 3 12s3.2 5 9 5a9 9 0 0 0 2.3-.3" />
    <path d="M9.9 9.9a3 3 0 0 0 4.2 4.2" />
  {:else if name === 'female'}
    <circle cx="12" cy="9" r="4" />
    <path d="M12 13v7m-3-3h6" />
  {:else if name === 'file' || name === 'receipt'}
    <path d="M6 3h8l4 4v14H6z" />
    <path d="M14 3v5h4M9 12h6m-6 4h6" />
  {:else if name === 'filter'}
    <path d="M4 5h16l-6 7v5l-4 2v-7L4 5Z" />
  {:else if name === 'flask'}
    <path d="M9 3h6m-5 0v6l-5.5 9.2A1.8 1.8 0 0 0 6 21h12a1.8 1.8 0 0 0 1.5-2.8L14 9V3" />
    <path d="M7.1 16h9.8" />
  {:else if name === 'gender'}
    <circle cx="10" cy="14" r="4" />
    <path d="m13 11 6-6m-4 0h4v4M10 18v3m-3-3h6" />
  {:else if name === 'grip'}
    <path d="M8 6h.1M16 6h.1M8 12h.1M16 12h.1M8 18h.1M16 18h.1" stroke-width="3" />
  {:else if name === 'info'}
    <circle cx="12" cy="12" r="8.5" />
    <path d="M12 11v5m0-8v.1" />
  {:else if name === 'key'}
    <circle cx="8" cy="15" r="3" />
    <path d="m10.3 12.7 7.7-7.7m-2 2 2 2m-4 0 2 2" />
  {:else if name === 'layers'}
    <path d="m12 4 8 4-8 4-8-4 8-4Z" />
    <path d="m4 12 8 4 8-4m-16 4 8 4 8-4" />
  {:else if name === 'lock'}
    <rect x="5" y="10" width="14" height="10" rx="1.8" />
    <path d="M8 10V7a4 4 0 0 1 8 0v3" />
  {:else if name === 'male'}
    <circle cx="10" cy="14" r="4" />
    <path d="m13 11 6-6m-4 0h4v4" />
  {:else if name === 'minmax'}
    <path d="M8 4v16m-3-3 3 3 3-3M5 7l3-3 3 3M16 20V4m-3 3 3-3 3 3m-6 10 3 3 3-3" />
  {:else if name === 'monitor'}
    <rect x="3.5" y="4.5" width="17" height="12" rx="1.5" />
    <path d="M9 20h6m-3-3.5V20" />
  {:else if name === 'mouse'}
    <rect x="7" y="3" width="10" height="18" rx="5" />
    <path d="M12 3v5m0-3h.1" />
  {:else if name === 'moon'}
    <path d="M19 15.5A8 8 0 0 1 8.5 5 8.5 8.5 0 1 0 19 15.5Z" fill="currentColor" stroke="none" />
  {:else if name === 'palette'}
    <path d="M12 4a8 8 0 0 0 0 16h1.2a1.8 1.8 0 0 0 1.2-3.1 1.8 1.8 0 0 1 1.2-3.1H18a2 2 0 0 0 2-2A8 8 0 0 0 12 4Z" />
    <path d="M7.5 10h.1m2-3h.1m5 1h.1m-8 6h.1" />
  {:else if name === 'pin'}
    <path d="m15 4 5 5-3 1-4 4 1 3-1 1-4-4 1-1 3 1 4-4 1-3-5-5Z" />
    <path d="m9 15-5 5" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'save'}
    <path d="M5 4h12l2 2v14H5z" />
    <path d="M8 4v5h7V4m-6 10h6v6H9z" />
  {:else if name === 'scale'}
    <path d="M12 4v16m-5 0h10M7 8h10M4 8l-3 6a3 3 0 0 0 6 0L4 8Zm16 0-3 6a3 3 0 0 0 6 0l-3-6Z" />
  {:else if name === 'settings'}
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v3m0 14v3M2 12h3m14 0h3M4.9 4.9 7 7m10 10 2.1 2.1M19.1 4.9 17 7M7 17l-2.1 2.1" />
  {:else if name === 'shield'}
    <path d="M12 3 19 6v5c0 4.7-3 8.2-7 10-4-1.8-7-5.3-7-10V6l7-3Z" />
    <path d="m9 12 2 2 4-4" />
  {:else if name === 'sun'}
    <circle cx="12" cy="12" r="3.5" />
    <path d="M12 2v2m0 16v2M2 12h2m16 0h2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" />
  {:else if name === 'tag'}
    <path d="M4 5v6l9 9 7-7-9-9H5a1 1 0 0 0-1 1Z" />
    <circle cx="8" cy="8" r="1" />
  {:else if name === 'target'}
    <circle cx="12" cy="12" r="8" />
    <circle cx="12" cy="12" r="4" />
    <circle cx="12" cy="12" r="1" fill="currentColor" />
  {:else if name === 'trash'}
    <path d="M5 7h14m-9 0V4h4v3m-8 0 1 13h10l1-13M10 11v5m4-5v5" />
  {:else if name === 'warning'}
    <path d="m12 3 9 17H3L12 3Z" />
    <path d="M12 9v5m0 3v.1" />
  {:else if name === 'unlock'}
    <rect x="5" y="10" width="14" height="10" rx="1.8" />
    <path d="M8 10V7a4 4 0 0 1 7.2-2.4" />
  {:else if name === 'x'}
    <path d="m6 6 12 12M18 6 6 18" />
  {/if}
</svg>
