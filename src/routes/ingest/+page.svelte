<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    ingestPdf,
    pickPdfs,
    subscribeProgress,
    STAGE_LABEL,
    type IngestProgress,
    type BatchProgress,
    type IngestStage
  } from '$api/ingest';
  import * as samples from '$api/samples';
  import { AppError, type AppErrorPayload, ERROR_TITLES } from '$api/errors';
  import { toasts } from '../../lib/toasts/store.svelte';
  import { windowTitle } from '$lib/title.svelte';
  import { t } from '$lib/i18n/index.svelte';

  type FileState = {
    path: string;
    fileName: string;
    stage: IngestStage;
    progress: number;
    elapsedMs: number;
    message: string | null;
    pages: number | null;
    rowsParsed: number | null;
    rowsUnmatched: number | null;
    inlinePriors: number | null;
    docConfidence: number | null;
    ingestTier: number | null;
    alreadyIngested: boolean | null;
    error: AppErrorPayload | null;
  };

  let files = $state<Map<string, FileState>>(new Map());
  let order = $state<string[]>([]);
  let batch = $state<BatchProgress | null>(null);
  let dragging = $state(false);
  let busy = $state(false);
  let sampleCount = $state<number | null>(null);
  let unlisteners: UnlistenFn[] = [];

  const baseName = (p: string) => p.split(/[\\/]/).pop() ?? p;

  function errorTitle(kind: string): string {
    return t(ERROR_TITLES[kind as keyof typeof ERROR_TITLES] ?? kind);
  }

  // Surface batch progress in the OS window title — taskbar tooltip becomes
  // "[ingest: 3 of 12]" so users can monitor without keeping the window open.
  $effect(() => {
    if (batch && batch.total > 0 && batch.completed + batch.failed < batch.total) {
      windowTitle.set(t('ingest'), t('{completed} of {total}', { completed: batch.completed + batch.failed, total: batch.total }));
    } else {
      windowTitle.clear('ingest');
    }
  });
  onDestroy(() => windowTitle.clear('ingest'));

  function ensureFile(path: string): FileState {
    let f = files.get(path);
    if (!f) {
      f = {
        path,
        fileName: baseName(path),
        stage: 'started',
        progress: 0,
        elapsedMs: 0,
        message: null,
        pages: null,
        rowsParsed: null,
        rowsUnmatched: null,
        inlinePriors: null,
        docConfidence: null,
        ingestTier: null,
        alreadyIngested: null,
        error: null
      };
      files.set(path, f);
      order = [path, ...order];
    }
    return f;
  }

  function toErrorPayload(error: unknown): AppErrorPayload {
    const normalized = AppError.fromUnknown(error, 'ingest_pdfs');
    return {
      kind: normalized.kind,
      code: normalized.code,
      message: normalized.message,
      detail: normalized.detail,
      retryable: normalized.retryable,
      timestamp: normalized.timestamp,
      context: normalized.context
    };
  }

  function applyProgress(p: IngestProgress) {
    const f = ensureFile(p.path);
    f.stage = p.stage;
    f.progress = p.progress;
    f.elapsedMs = p.elapsed_ms;
    if (p.message != null) f.message = p.message;
    if (p.pages != null) f.pages = p.pages;
    if (p.rows_parsed != null) f.rowsParsed = p.rows_parsed;
    if (p.rows_unmatched != null) f.rowsUnmatched = p.rows_unmatched;
    if (p.inline_priors != null) f.inlinePriors = p.inline_priors;
    if (p.doc_confidence != null) f.docConfidence = p.doc_confidence;
    if (p.already_ingested != null) f.alreadyIngested = p.already_ingested;
    if (p.error) f.error = p.error;
    files = new Map(files);
  }

  /**
   * Progress events are deliberately best-effort UI telemetry. Reconcile the
   * final IPC response as the source of truth as well, so a dropped final
   * event can never leave a completed item displaying "Queued" forever.
   */
  function applyOutcomeToFile(f: FileState, outcome: import('$api/ingest').IngestOutcome) {
    const result = outcome.result;
    f.stage = outcome.ok
      ? result?.already_ingested ? 'duplicate' : 'completed'
      : 'error';
    f.progress = 1;
    f.error = outcome.error;
    if (result) {
      f.rowsParsed = result.rows_parsed;
      f.rowsUnmatched = result.rows_unmatched;
      f.inlinePriors = result.inline_priors_emitted;
      f.docConfidence = result.doc_confidence;
      f.ingestTier = result.ingest_tier;
      f.alreadyIngested = result.already_ingested;
      f.message = result.already_ingested
        ? t('Already ingested as {reportId}', { reportId: result.report_id })
        : t('Ingested as {reportId}', { reportId: result.report_id });
    }
  }

  function applyOutcomes(paths: string[], outcomes: import('$api/ingest').IngestOutcome[]) {
    const next = new Map(files);
    const seen = new Set<string>();
    for (const outcome of outcomes) {
      const f = next.get(outcome.path);
      if (f) {
        seen.add(outcome.path);
        applyOutcomeToFile(f, outcome);
      }
    }
    for (const path of paths) {
      if (seen.has(path)) continue;
      const f = next.get(path);
      if (!f) continue;
      f.stage = 'error';
      f.progress = 1;
      f.error = toErrorPayload(new Error(t('Ingestion returned no final result for this file')));
      f.message = t('No final result returned by the ingestion service');
    }
    files = next;
  }

  function markBatchError(paths: string[], error: unknown) {
    const payload = toErrorPayload(error);
    const next = new Map(files);
    let completed = 0;
    for (const path of paths) {
      const f = next.get(path);
      if (!f) continue;
      if (f.stage === 'completed' || f.stage === 'duplicate') {
        completed += 1;
        continue;
      }
      f.stage = 'error';
      f.progress = 1;
      f.error = payload;
      f.message = payload.message;
    }
    files = next;
    batch = {
      total: paths.length,
      completed,
      failed: paths.length - completed,
      current_path: null,
      elapsed_ms: batch?.elapsed_ms ?? 0
    };
  }

  async function runIngest(paths: string[]) {
    if (busy || paths.length === 0) return;
    busy = true;
    let completed = 0;
    let failed = 0;
    const batchStarted = Date.now();
    try {
      // Process one file per IPC call so each item receives its terminal
      // result immediately. A batch command only resolves after every PDF;
      // when one native extraction was slow, that made every selected item
      // look stuck in the initial "Hashing" state.
      for (const path of paths) {
        const f = ensureFile(path);
        f.stage = 'hashing';
        f.progress = 0.05;
        f.elapsedMs = 0;
        f.message = t('Preparing PDF…');
        f.error = null;
        files = new Map(files);
        batch = {
          total: paths.length,
          completed,
          failed,
          current_path: path,
          elapsed_ms: Date.now() - batchStarted
        };

        try {
          const result = await ingestPdf(path);
          applyOutcomes([path], [{ path, index: completed + failed, ok: true, error: null, result }]);
          completed += 1;
        } catch (error) {
          applyOutcomes([path], [{ path, index: completed + failed, ok: false, error: toErrorPayload(error), result: null }]);
          failed += 1;
        }

        batch = {
          total: paths.length,
          completed,
          failed,
          current_path: null,
          elapsed_ms: Date.now() - batchStarted
        };
      }
    } catch (e) {
      markBatchError(paths, e);
      toasts.error(e);
    } finally {
      busy = false;
    }
  }

  async function onPick() {
    const paths = await pickPdfs();
    if (paths.length) await runIngest(paths);
  }

  async function loadSamples() {
    try {
      const s = await samples.list();
      if (s.length === 0) {
        toasts.warn(t('No sample PDFs found'), t('Expected `<repo>/data test/*.pdf` in dev mode.'));
        return;
      }
      await runIngest(s.map((x) => x.path));
    } catch (e) {
      toasts.error(e);
    }
  }

  function clearCompleted() {
    const remaining: string[] = [];
    for (const p of order) {
      const f = files.get(p);
      if (f && (f.stage === 'completed' || f.stage === 'duplicate' || f.stage === 'error')) {
        files.delete(p);
      } else {
        remaining.push(p);
      }
    }
    order = remaining;
    files = new Map(files);
  }

  function clearAll() {
    files = new Map();
    order = [];
    batch = null;
  }

  onMount(async () => {
    const unsub = await subscribeProgress(
      (p) => applyProgress(p),
      (b) => (batch = b)
    );
    unlisteners.push(unsub);

    unlisteners.push(
      await listen<{ paths: string[] }>('tauri://drag-drop', async (e) => {
        dragging = false;
        const pdfs = (e.payload.paths || []).filter((p) => p.toLowerCase().endsWith('.pdf'));
        if (pdfs.length) await runIngest(pdfs);
      })
    );
    unlisteners.push(await listen('tauri://drag-enter', () => { dragging = true; }));
    unlisteners.push(await listen('tauri://drag-leave', () => { dragging = false; }));

    try {
      const s = await samples.list();
      sampleCount = s.length;
    } catch {
      sampleCount = 0;
    }
  });

  onDestroy(() => {
    for (const u of unlisteners) u();
    unlisteners = [];
  });

  function fmtElapsed(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }

  function stageColor(s: IngestStage): string {
    switch (s) {
      case 'completed': return 'pill-ok';
      case 'duplicate': return 'pill-muted';
      case 'error':     return 'pill-crit';
      default:          return 'pill-warn';
    }
  }

  function progressBarColor(s: IngestStage): string {
    switch (s) {
      case 'completed': return 'bg-ok';
      case 'duplicate': return 'bg-fg3';
      case 'error':     return 'bg-crit';
      default:          return 'bg-accent';
    }
  }

  const failedFiles = $derived(order.map((p) => files.get(p)).filter((f) => f && f.stage === 'error') as FileState[]);
</script>

<div class="space-y-4">
  <div class="flex items-end justify-between gap-2">
    <div>
      <h1 class="text-xl font-semibold">{t('Ingest')}</h1>
      <p class="text-sm text-fg2">{t('Drag-drop PDFs anywhere in the window, or pick files.')}</p>
    </div>
    <div class="flex items-center gap-2">
      {#if sampleCount && sampleCount > 0}
        <button class="btn" onclick={loadSamples} disabled={busy}>
          {t(sampleCount === 1 ? 'Load {count} sample' : 'Load {count} samples', { count: sampleCount })}
        </button>
      {/if}
      {#if order.length > 0}
        <button class="btn" onclick={clearCompleted} disabled={busy}>{t('Clear done')}</button>
        <button class="btn" onclick={clearAll} disabled={busy}>{t('Clear all')}</button>
      {/if}
      <button class="btn-accent" disabled={busy} onclick={onPick}>
        {busy ? t('Ingesting…') : t('Pick PDFs…')}
      </button>
    </div>
  </div>

  {#if batch && batch.total > 0}
    <section class="card p-3 space-y-2">
      <div class="flex items-center justify-between text-xs text-fg2">
        <span>
          {t('Batch')} <span class="text-fg1 font-medium">{batch.completed}</span> {t('done')}
          {#if batch.failed > 0}, <span class="text-crit font-medium">{batch.failed}</span> {t('failed')}{/if}
          / {batch.total}
        </span>
        <span class="tabular-nums">{fmtElapsed(batch.elapsed_ms)}</span>
      </div>
      <div class="h-1.5 bg-bg3 rounded overflow-hidden">
        <div
          class="h-full bg-accent transition-all duration-150"
          style="width: {((batch.completed + batch.failed) / batch.total) * 100}%"
        ></div>
      </div>
      {#if batch.current_path}
        <p class="text-xs text-fg3 truncate">→ {baseName(batch.current_path)}</p>
      {/if}

      {#if failedFiles.length > 0}
        <details open class="mt-2 border-t border-line pt-2">
          <summary class="text-xs font-medium text-crit cursor-pointer select-none">
            {t(failedFiles.length === 1 ? '{count} failure — expand to see why' : '{count} failures — expand to see why', { count: failedFiles.length })}
          </summary>
          <ul class="mt-2 space-y-2">
            {#each failedFiles as f}
              <li class="border-l-2 border-crit pl-2 text-xs">
                <div class="font-medium text-fg1 truncate" title={f.path}>{f.fileName}</div>
                {#if f.error}
                  <div class="text-crit">{errorTitle(f.error.kind)}: {f.error.message}</div>
                  <div class="text-fg3 font-mono text-[10px]">{f.error.code}</div>
                  {#if f.error.context?.stage}
                    <div class="text-fg3 text-[10px]">{t('stage:')} <span class="font-mono">{f.error.context.stage}</span></div>
                  {/if}
                  {#if f.error.context?.hints && f.error.context.hints.length > 0}
                    <ul class="mt-1 list-disc list-inside text-fg2 text-[11px] space-y-0.5">
                      {#each f.error.context.hints as h}
                        <li>{h}</li>
                      {/each}
                    </ul>
                  {/if}
                {/if}
              </li>
            {/each}
          </ul>
        </details>
      {/if}
    </section>
  {/if}

  <div
    class="card border-2 border-dashed p-8 text-center transition-colors
           {dragging ? 'bg-bg3 border-accent' : ''}"
  >
    <p class="text-sm text-fg2">{dragging ? t('Release to ingest') : t('Drop PDFs here')}</p>
    <p class="text-xs text-fg3 mt-1">{t('Tier 1 baseline: PDFium extraction → parser → encrypted SQLite')}</p>
  </div>

  {#if order.length > 0}
    <section class="card divide-y divide-line">
      {#each order as path (path)}
        {@const f = files.get(path)}
        {#if f}
          <div class="px-3 py-3 space-y-2">
            <div class="flex items-center gap-3">
              <span class="text-sm font-medium truncate flex-1" title={f.path}>{f.fileName}</span>
              <span class={stageColor(f.stage)}>
                {f.alreadyIngested ? t('Duplicate') : t(STAGE_LABEL[f.stage])}
              </span>
              <span class="text-xs text-fg3 tabular-nums w-12 text-right">{fmtElapsed(f.elapsedMs)}</span>
            </div>

            {#if f.stage !== 'completed' && f.stage !== 'duplicate' && f.stage !== 'error'}
              <div class="h-1.5 bg-bg3 rounded overflow-hidden">
                <div
                  class="h-full {progressBarColor(f.stage)} transition-all duration-150"
                  style="width: {Math.max(2, f.progress * 100)}%"
                ></div>
              </div>
            {/if}

            <div class="flex flex-wrap items-center gap-x-3 gap-y-1 text-[11px] text-fg3">
              {#if f.message}
                <span class="text-fg2 truncate max-w-md" title={f.message}>{f.message}</span>
              {/if}
               {#if f.pages != null}<span>{f.pages} {t('pages')}</span>{/if}
              {#if f.rowsParsed != null}
                <span>
                   {f.rowsParsed} {t('rows')}
                  {#if f.rowsUnmatched != null && f.rowsUnmatched > 0}
                     <span class="text-warn">({f.rowsUnmatched} {t('unmatched')})</span>
                  {/if}
                </span>
              {/if}
              {#if f.inlinePriors != null && f.inlinePriors > 0}
                 <span>{f.inlinePriors} {t('priors')}</span>
              {/if}
              {#if f.docConfidence != null}
                 <span>{t('conf {confidence}%', { confidence: (f.docConfidence * 100).toFixed(0) })}</span>
              {/if}
              {#if f.ingestTier != null}
                 <span class="pill-muted">{t('Tier {tier}', { tier: f.ingestTier })}</span>
              {/if}
            </div>

            {#if f.error}
              <div class="border-l-2 border-crit pl-2 mt-1 space-y-1">
                <div class="text-xs text-crit">
                   <span class="font-semibold">{errorTitle(f.error.kind)}:</span>
                  {f.error.message}
                </div>
                <div class="text-[10px] text-fg3 font-mono">
                  {f.error.code}{f.error.context?.stage ? ` · stage=${f.error.context.stage}` : ''}
                </div>
                {#if f.error.context?.hints && f.error.context.hints.length > 0}
                  <ul class="text-[11px] text-fg2 list-disc list-inside space-y-0.5">
                    {#each f.error.context.hints as h}
                      <li>{h}</li>
                    {/each}
                  </ul>
                {/if}
                {#if f.error.detail}
                  <details>
                     <summary class="text-[10px] text-fg3 cursor-pointer select-none">{t('Technical detail')}</summary>
                    <pre class="text-[10px] text-fg3 mt-1 whitespace-pre-wrap font-mono">{f.error.detail}</pre>
                  </details>
                {/if}
              </div>
            {/if}
          </div>
        {/if}
      {/each}
    </section>
  {/if}
</div>
