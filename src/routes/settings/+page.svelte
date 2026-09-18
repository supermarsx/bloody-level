<script lang="ts">
  import { onMount } from 'svelte';
  import * as settings from '$api/settings';
  import * as tiers from '$api/tiers';
  import * as security from '$api/security';
  import * as appInfo from '$api/app-info';
  import * as admin from '$api/records-admin';
  import { theme } from '$theme/store.svelte';
  import { chartPrefs } from '$charts/prefs.svelte';
  import { comparePresets, EMPTY_FILTERS, type ComparePreset, type ComparePresetFilters, type EditablePreset } from '$charts/compare-presets.svelte';
  import * as analyteApi from '$api/analyte-info';
  import { appearance, ACCENT_PRESETS, type AccentName, type Density, type FontScale, type FontFamily } from '$theme/appearance.svelte';
  import { openFileExternal, openUrl } from '$api/shell';
  import Icon, { type IconName } from '$components/icon.svelte';
  import { dashboardPrefs, DASHBOARD_SECTIONS } from '$lib/dashboard/prefs.svelte';
  import { discardPending } from '$api/debounced-settings';

  // ── Library credits — typed once so each row is a real link ───────────
  type Credit = { name: string; note: string; url: string };
  type CreditGroup = { head: string; items: Credit[] };
  const credits: CreditGroup[] = [
    { head: 'App shell', items: [
      { name: 'Tauri 2',                 note: 'tauri.app',         url: 'https://tauri.app/' },
      { name: 'Svelte 5',                note: 'svelte.dev',        url: 'https://svelte.dev/' },
      { name: 'SvelteKit 2',             note: 'kit.svelte.dev',    url: 'https://kit.svelte.dev/' },
      { name: 'Tailwind CSS 4',          note: 'tailwindcss.com',   url: 'https://tailwindcss.com/' },
      { name: 'Vite 8',                  note: 'vitejs.dev',        url: 'https://vitejs.dev/' }
    ]},
    { head: 'PDF / parsing', items: [
      { name: 'pdfium-render',           note: 'crates.io/pdfium-render', url: 'https://crates.io/crates/pdfium-render' },
      { name: 'Google Pdfium',           note: 'pdfium.googlesource.com', url: 'https://pdfium.googlesource.com/pdfium/' },
      { name: 'bblanchon/pdfium-binaries', note: 'prebuilt binaries',     url: 'https://github.com/bblanchon/pdfium-binaries' },
      { name: 'regex',                   note: 'rust-lang/regex',         url: 'https://github.com/rust-lang/regex' },
      { name: 'strsim',                  note: 'fuzzy matching',          url: 'https://crates.io/crates/strsim' },
      { name: 'unicode-normalization',   note: 'diacritic folding',       url: 'https://crates.io/crates/unicode-normalization' }
    ]},
    { head: 'Storage / crypto', items: [
      { name: 'rusqlite',                note: 'crates.io/rusqlite',      url: 'https://crates.io/crates/rusqlite' },
      { name: 'SQLCipher',               note: 'zetetic.net/sqlcipher',   url: 'https://www.zetetic.net/sqlcipher/' },
      { name: 'argon2 (RustCrypto)',     note: 'password KDF',            url: 'https://crates.io/crates/argon2' },
      { name: 'chacha20poly1305',        note: 'DMK wrap',                url: 'https://crates.io/crates/chacha20poly1305' },
      { name: 'hkdf',                    note: 'sub-key derivation',      url: 'https://crates.io/crates/hkdf' },
      { name: 'secrecy',                 note: 'memory hygiene',          url: 'https://crates.io/crates/secrecy' },
      { name: 'zeroize',                 note: 'secure-erase utilities',  url: 'https://crates.io/crates/zeroize' }
    ]},
    { head: 'Charts & UI', items: [
      { name: 'Apache ECharts 6',        note: 'echarts.apache.org',      url: 'https://echarts.apache.org/' },
      { name: 'date-fns',                note: 'date math helpers',       url: 'https://date-fns.org/' },
      { name: 'sharp',                   note: 'icon rasterisation',      url: 'https://sharp.pixelplumbing.com/' }
    ]},
    { head: 'Optional model tiers', items: [
      { name: 'Microsoft Phi-4-mini-reasoning', note: 'repair tier (opt-in)', url: 'https://huggingface.co/microsoft/Phi-4-mini-reasoning' },
      { name: 'AI2 olmOCR-2',            note: 'vision OCR (opt-in)',     url: 'https://huggingface.co/allenai/olmOCR-7B-0225-preview' },
      { name: 'Tesseract',               note: 'classical OCR (opt-in)',  url: 'https://github.com/tesseract-ocr/tesseract' },
      { name: 'llama.cpp',               note: 'GGUF runtime',            url: 'https://github.com/ggml-org/llama.cpp' },
      { name: 'llama-cpp-2 (Rust)',      note: 'Rust bindings',           url: 'https://crates.io/crates/llama-cpp-2' }
    ]},
    { head: 'Reference data', items: [
      { name: 'LOINC',                   note: 'analyte identifiers',     url: 'https://loinc.org/' },
      { name: 'CUF Saúde',               note: 'lab format calibration',  url: 'https://www.cuf.pt/' },
      { name: 'Germano de Sousa',        note: 'lab format calibration',  url: 'https://www.germanodesousa.com/' },
      { name: 'ADA',                     note: 'diabetes guidelines',     url: 'https://diabetes.org/' },
      { name: 'NICE',                    note: 'UK clinical guidance',    url: 'https://www.nice.org.uk/' },
      { name: 'Mayo Clinic Laboratories', note: 'reference ranges',       url: 'https://www.mayocliniclabs.com/' },
      { name: 'LabCorp',                 note: 'reference ranges',        url: 'https://www.labcorp.com/' }
    ]}
  ];

  async function visit(url: string) {
    try { await openUrl(url); }
    catch (e) { toasts.error(e); }
  }
  import { toasts } from '../../lib/toasts/store.svelte';
  import { goto, replaceState } from '$app/navigation';
  import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
  import * as auth from '$api/auth';
  import * as lifecycle from '$api/lifecycle';

  // ── Loaded data ────────────────────────────────────────────────────────
  let tess   = $state<tiers.TierStatus | null>(null);
  let llm    = $state<tiers.ModelTierStatus | null>(null);
  let olm    = $state<tiers.ModelTierStatus | null>(null);
  let pdfium = $state<tiers.PdfiumStatus | null>(null);
  let securityStatus = $state<security.SecurityStatus | null>(null);
  let info   = $state<appInfo.AppInfo | null>(null);
  let raw    = $state<Record<string, unknown>>({});
  let err    = $state<string | null>(null);

  async function refresh() {
    err = null;
    try {
      [tess, llm, olm, pdfium, info, raw, securityStatus] = await Promise.all([
        tiers.tesseract(), tiers.llm(), tiers.olmocr(), tiers.pdfium(),
        appInfo.get(), settings.getAll(), security.status()
      ]);
    } catch (e) {
      err = String(e);
      toasts.error(e);
    }
  }

  async function toggleTier(key: 'tesseract' | 'llm' | 'olmocr', enabled: boolean) {
    const current = (raw[key] as Record<string, unknown> | undefined) ?? {};
    await settings.set(key, { ...current, enabled });
    await refresh();
  }

  type ModelTierKey = 'llm' | 'olmocr';
  let tierAction = $state<ModelTierKey | null>(null);
  let tierDownload = $state<'tesseract' | ModelTierKey | null>(null);

  function configuredModelPath(key: ModelTierKey, status?: tiers.TierStatus | null): string {
    const value = (raw[key] as Record<string, unknown> | undefined)?.model_path;
    if (typeof value === 'string' && value.trim()) return value.trim();
    const configured = 'configured_model_path' in (status ?? {})
      ? (status as tiers.ModelTierStatus).configured_model_path
      : null;
    return configured?.trim() ?? '';
  }

  function modelPathHint(key: ModelTierKey, status: tiers.TierStatus): string {
    const configured = configuredModelPath(key, status);
    if (configured) return configured;
    if (status.loaded_model_path) return status.loaded_model_path;
    return info?.models_dir ? `${info.models_dir} (set model_path in this tier)` : 'No model_path configured';
  }

  async function runTierAction(key: ModelTierKey, action: 'load' | 'unload', modelPath = '') {
    if (tierAction) return;
    tierAction = key;
    try {
      if (key === 'llm') {
        if (action === 'load') await tiers.loadLlm(modelPath);
        else await tiers.unloadLlm();
      } else {
        if (action === 'load') await tiers.loadOlmocr(modelPath);
        else await tiers.unloadOlmocr();
      }
      await refresh();
    } catch (e) {
      toasts.error(e);
      await refresh();
    } finally {
      tierAction = null;
    }
  }

  async function downloadModel(key: ModelTierKey) {
    if (tierDownload) return;
    const details = key === 'llm'
      ? 'Download the recommended Phi-4 Q4_K_M model (about 2.5 GB) into the encrypted app data folder?'
      : 'Download the complete olmOCR-2 7B model snapshot (about 16 GB) into the app data folder?';
    if (!await ask(details, { title: `Download ${key === 'llm' ? 'Phi-4' : 'olmOCR-2'} model`, kind: 'warning' })) return;
    tierDownload = key;
    try {
      if (key === 'llm') await tiers.downloadLlm();
      else await tiers.downloadOlmocr();
      toasts.success('Model download complete', `${key === 'llm' ? 'Phi-4' : 'olmOCR-2'} is configured for this instance.`);
      await refresh();
    } catch (e) { toasts.error(e); await refresh(); }
    finally { tierDownload = null; }
  }

  async function downloadTesseractLanguage(language: string) {
    if (tierDownload) return;
    tierDownload = 'tesseract';
    try {
      await tiers.downloadTesseractLanguage(language);
      toasts.success('Language data downloaded', `Tesseract ${language} data is ready in the managed models folder.`);
      await refresh();
    } catch (e) { toasts.error(e); await refresh(); }
    finally { tierDownload = null; }
  }

  async function chooseModelPath(key: ModelTierKey) {
    const selected = await openDialog({
      directory: false,
      multiple: false,
      title: key === 'llm' ? 'Choose Phi-4 model file' : 'Choose olmOCR model file'
    }) as string | null;
    if (!selected) return;
    const current = (raw[key] as Record<string, unknown> | undefined) ?? {};
    await settings.set(key, { ...current, model_path: selected });
    await refresh();
  }

  async function enableOsVault() {
    try {
      securityStatus = await security.enableOsVault();
      toasts.success('OS vault enabled', `The vault DMK is now protected by ${securityStatus.os_vault_platform}.`);
    } catch (e) { toasts.error(e); }
  }

  async function disableOsVault() {
    const confirmed = await ask(
      'Disable native OS vault unlock? The encrypted database and PDF cache will remain protected, but this device will no longer offer OS-vault unlock.',
      { title: 'Disable OS vault', kind: 'warning' }
    );
    if (!confirmed) return;
    try {
      securityStatus = await security.disableOsVault();
      toasts.success('OS vault disabled', 'Password and passkey unlock remain available.');
    } catch (e) { toasts.error(e); }
  }

  async function toggleAutoUnlock(enabled: boolean) {
    try { securityStatus = await security.setAutoUnlock(enabled); }
    catch (e) { toasts.error(e); }
  }

  // ── Storage tab — export / import vault ───────────────────────────────
  let loadingStorage = $state(false);
  let exporting     = $state(false);
  let importing     = $state(false);
  let lastExport    = $state<appInfo.ExportVaultResult | null>(null);
  let lastImport    = $state<appInfo.ImportVaultResult | null>(null);

  async function onExportVault() {
    if (exporting) return;
    let dest: string | null = null;
    try {
      // `open` with `directory: true` returns the chosen folder path, or
      // null when the user cancels.
      dest = await openDialog({
        directory: true,
        multiple: false,
        title: 'Choose a destination folder for the vault export'
      }) as string | null;
    } catch (e) {
      toasts.error(e);
      return;
    }
    if (!dest) return;
    exporting = true;
    try {
      const r = await appInfo.exportVault(dest);
      lastExport = r;
      toasts.success(
        'Vault exported',
        `${r.files_copied} files · ${appInfo.formatBytes(r.bytes_copied)} written to ${r.destination}`
      );
      await refresh();
    } catch (e) {
      toasts.error(e);
    } finally {
      exporting = false;
    }
  }

  async function onOpenDataFolder() {
    if (!info?.data_dir) return;
    try {
      await openFileExternal(info.data_dir);
    } catch (e) {
      toasts.error(e);
    }
  }

  async function onImportVault() {
    if (importing) return;
    // Hard confirm + walk the user through the lock requirement. We do the
    // ask FIRST so we don't lock the vault if they cancel the picker.
    const proceed = await ask(
      'Replace the current vault with a previous export?\n\n' +
      '• The current vault will be renamed to data.backup-<timestamp> alongside the data dir.\n' +
      '• The vault will be locked first; you will need the password from the source export to unlock it.\n' +
      '• Restart the app after import for the new vault to take effect cleanly.\n\n' +
      'This cannot be undone via the UI — manual rollback only.',
      { title: 'Import vault', kind: 'warning' }
    );
    if (!proceed) return;
    let src: string | null = null;
    try {
      src = await openDialog({
        directory: true,
        multiple: false,
        title: 'Select the exported vault directory to restore'
      }) as string | null;
    } catch (e) {
      toasts.error(e);
      return;
    }
    if (!src) return;
    importing = true;
    try {
      // Lock the vault before swapping files — the open SQLite handle
      // would otherwise pin the old DB on Windows.
      try { await auth.lock(); } catch { /* already locked is fine */ }
      const r = await appInfo.importVault(src);
      lastImport = r;
      toasts.success(
        'Vault imported',
        `${r.files_copied} files · ${appInfo.formatBytes(r.bytes_copied)} restored. Restart the app to continue.`
      );
    } catch (e) {
      toasts.error(e);
    } finally {
      importing = false;
    }
  }

  // ── Compare presets editor ────────────────────────────────────────────
  let allAnalytes = $state<analyteApi.AnalyteOntologyEntry[]>([]);
  let presetEditingId = $state<string | null>(null);
  // The form mirrors EditablePreset so save() just hands it through.
  let editorForm = $state<{ name: string; ids: string[]; analyteSearch: string;
                            useFilters: boolean; filters: ComparePresetFilters }>({
    name: '',
    ids: [],
    analyteSearch: '',
    useFilters: false,
    filters: { ...EMPTY_FILTERS }
  });
  let creating = $state(false);

  async function ensureAnalytesLoaded() {
    if (allAnalytes.length > 0) return;
    try { allAnalytes = await analyteApi.listOntologyEntries(); }
    catch (e) { toasts.error(e); }
  }

  function startEditPreset(p: ComparePreset) {
    presetEditingId = p.id;
    creating = false;
    const f = p.filters;
    editorForm = {
      name: p.name,
      // Dynamic presets don't have an editable analyte list.
      ids: p.kind === 'static' ? [...p.ids] : [],
      analyteSearch: '',
      useFilters: f !== null && f !== undefined,
      filters: { ...EMPTY_FILTERS, ...f },
    };
    void ensureAnalytesLoaded();
  }

  function startCreatePreset() {
    presetEditingId = '__new__';
    creating = true;
    editorForm = {
      name: '',
      ids: [],
      analyteSearch: '',
      useFilters: false,
      filters: { ...EMPTY_FILTERS },
    };
    void ensureAnalytesLoaded();
  }

  function cancelEditPreset() {
    presetEditingId = null;
    creating = false;
  }

  /** Build a Partial<ComparePresetFilters> with only the fields that
   *  diverge from EMPTY_FILTERS so we don't store a noisy "every key
   *  carrying a default value" object. */
  function diffFilters(f: ComparePresetFilters): Partial<ComparePresetFilters> {
    const out: Partial<ComparePresetFilters> = {};
    for (const k of Object.keys(EMPTY_FILTERS) as (keyof ComparePresetFilters)[]) {
      const def = EMPTY_FILTERS[k] as unknown;
      const cur = f[k] as unknown;
      if (def !== cur) (out as Record<string, unknown>)[k] = cur;
    }
    return out;
  }

  function savePreset() {
    if (presetEditingId == null) return;
    const filters = editorForm.useFilters ? diffFilters(editorForm.filters) : null;
    const editable: EditablePreset = {
      name: editorForm.name.trim() || 'Untitled',
      ids: editorForm.ids,
      filters,
    };
    if (creating) {
      const created = comparePresets.addUserPreset({
        name: editable.name,
        ids: editable.ids,
        filters: editable.filters,
      });
      toasts.success('Preset created', `“${created.name}” will appear in Compare's preset bar.`);
    } else {
      // For bundled presets we write to bundledOverrides so reset-defaults
      // works; for user presets we update the user list directly.
      const target = comparePresets.presets.find((p) => p.id === presetEditingId);
      if (target?.source === 'user') {
        comparePresets.updateUserPreset(presetEditingId, editable);
      } else {
        comparePresets.setBundledOverride(presetEditingId, editable);
      }
      toasts.success('Preset saved', `“${editable.name}” updated.`);
    }
    cancelEditPreset();
  }

  async function deletePreset(p: ComparePreset) {
    if (p.source !== 'user') return;
    const ok = await ask(
      `Delete preset “${p.name}”? This cannot be undone.`,
      { title: 'Delete preset', kind: 'warning' }
    );
    if (!ok) return;
    comparePresets.removeUserPreset(p.id);
    if (presetEditingId === p.id) cancelEditPreset();
    toasts.success('Preset deleted', `“${p.name}” removed.`);
  }

  async function resetPreset(p: ComparePreset) {
    if (!comparePresets.isBundledOverridden(p.id)) return;
    const ok = await ask(
      `Reset “${p.name}” to its bundled defaults? Your customisations will be lost.`,
      { title: 'Reset preset', kind: 'warning' }
    );
    if (!ok) return;
    comparePresets.clearBundledOverride(p.id);
    if (presetEditingId === p.id) cancelEditPreset();
    toasts.success('Preset reset', `“${p.name}” restored to bundled defaults.`);
  }

  async function resetAllPresets() {
    const ok = await ask(
      'Restore every bundled preset to its bundled defaults?\n\n' +
      '• Your edits to bundled presets (Iron panel, Lipids, Hematology, Abnormal, Subclinical, etc.) will be discarded.\n' +
      '• User-created presets are preserved.',
      { title: 'Reset bundled presets', kind: 'warning' }
    );
    if (!ok) return;
    comparePresets.resetAllBundled();
    cancelEditPreset();
    toasts.success('Bundled presets reset');
  }

  function toggleEditorAnalyte(id: string) {
    editorForm.ids = editorForm.ids.includes(id)
      ? editorForm.ids.filter((x) => x !== id)
      : [...editorForm.ids, id];
  }

  function moveEditorAnalyte(idx: number, delta: number) {
    const target = idx + delta;
    if (target < 0 || target >= editorForm.ids.length || target === idx) return;
    const next = [...editorForm.ids];
    const [m] = next.splice(idx, 1);
    next.splice(target, 0, m);
    editorForm.ids = next;
  }

  let reloading = $state(false);
  async function onReloadOntology() {
    // Re-install OVERWRITES every seed-source row's metadata (descriptions,
    // refs, tiers, aliases) from the bundled JSON. User-created analytes
    // and user-source aliases are left untouched. Worth a confirm so a
    // misclick on a populated vault doesn't surprise anyone.
    const ok = await ask(
      'Reload analyte ontology from the bundled seed?\n\n' +
      '• Re-installs every seed-bundled analyte\'s descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n' +
      '• User-created analytes (source=user) and user-added aliases are preserved.\n' +
      '• Existing parsed results are not touched. Run "Re-parse all" on Records afterwards if you want stored rows to pick up new ontology fields.',
      { title: 'Reload ontology', kind: 'warning' }
    );
    if (!ok) return;
    reloading = true;
    try {
      const r = await admin.reloadOntology();
      toasts.success('Ontology reloaded', `${r.analytes_installed} analytes installed.`);
      await refresh();
    } catch (e) { toasts.error(e); }
    finally { reloading = false; }
  }

  // ── App lifecycle ─────────────────────────────────────────────────────
  type LifecycleAction = 'frontend' | 'app' | 'reset' | 'defaults';
  let lifecycleAction = $state<LifecycleAction | null>(null);

  function onRestartFrontend() {
    if (lifecycleAction) return;
    lifecycleAction = 'frontend';
    toasts.info('Restarting frontend', 'Reloading the current window…', 1500);
    // Let the toast paint before replacing the webview document.
    setTimeout(() => lifecycle.restartFrontend(), 80);
  }

  async function onRestartApp() {
    if (lifecycleAction) return;
    const ok = await ask(
      'Restart bloody-level now?\n\n' +
      'Any unsaved changes in the current view will be discarded. The local vault will remain intact.',
      { title: 'Restart app', kind: 'warning' }
    );
    if (!ok) return;
    lifecycleAction = 'app';
    try {
      await lifecycle.restartApp();
    } catch (e) {
      lifecycleAction = null;
      toasts.error(e);
    }
  }

  async function onResetApp() {
    if (lifecycleAction) return;
    const ok = await ask(
      'Reset this local bloody-level instance?\n\n' +
      'This permanently removes the encrypted vault, password, passkeys, imported reports, PDFs, models, and settings from this device. It cannot be undone. Export a backup first if you may need this data later.',
      { title: 'Reset local app', kind: 'warning' }
    );
    if (!ok) return;
    lifecycleAction = 'reset';
    try {
      // The backend refuses to remove an open vault. Locking first also
      // releases the SQLite handle on Windows before deleting its directory.
      await auth.lock();
      await auth.resetInstance();
      toasts.success('Local app reset', 'Starting the welcome screen…', 2500);
      setTimeout(() => {
        if (typeof window !== 'undefined') window.location.assign('/');
      }, 80);
    } catch (e) {
      lifecycleAction = null;
      toasts.error(e);
    }
  }

  async function onResetDefaults() {
    if (lifecycleAction) return;
    const ok = await ask(
      'Reset all bloody-level preferences to their bundled defaults?\n\n' +
      'This clears appearance, dashboard, chart, compare, OCR, and other settings. Your vault, reports, PDFs, models, and authentication remain untouched.',
      { title: 'Reset all preferences', kind: 'warning' }
    );
    if (!ok) return;
    lifecycleAction = 'defaults';
    try {
      discardPending();
      await settings.resetAll();
      toasts.success('Preferences reset', 'Reloading bloody-level with the bundled defaults…');
      setTimeout(() => lifecycle.restartFrontend(), 80);
    } catch (e) {
      lifecycleAction = null;
      toasts.error(e);
    }
  }

  onMount(refresh);
  onMount(() => dashboardPrefs.load());

  // ── Tab navigation ─────────────────────────────────────────────────────
  // Tab id is reflected in the URL hash so deep-linking works (e.g.
  // `/settings#charts` opens directly to the chart pane). Falls back to
  // the first tab when the hash is unknown / missing.
  type TabId = 'appearance' | 'dashboard' | 'charts' | 'comparison' | 'security' | 'ingestion' | 'ontology' | 'storage' | 'about' | 'advanced';
  const tabs: { id: TabId; label: string; hint: string; icon: IconName }[] = [
    { id: 'appearance', label: 'Appearance', hint: 'Theme & visual',          icon: 'palette' },
    { id: 'dashboard',  label: 'Dashboard',  hint: 'Home layout',              icon: 'dashboard' },
    { id: 'charts',     label: 'Charts',     hint: 'Zoom, sliders, labels',   icon: 'chart' },
    { id: 'comparison', label: 'Comparison', hint: 'Presets & filter sets',   icon: 'flask' },
    { id: 'security',   label: 'Security',   hint: 'OS vault & unlock',        icon: 'shield' },
    { id: 'ingestion',  label: 'Ingestion',  hint: 'PDF / OCR / LLM tiers',   icon: 'download' },
    { id: 'ontology',   label: 'Ontology',   hint: 'Analyte registry',        icon: 'book' },
    { id: 'storage',    label: 'Storage',    hint: 'Paths & sizes',           icon: 'database' },
    { id: 'about',      label: 'About',      hint: 'Build & environment',     icon: 'info' },
    { id: 'advanced',   label: 'Advanced',   hint: 'Raw settings JSON',       icon: 'settings' }
  ];

  let activeTab = $state<TabId>('appearance');
  onMount(() => {
    if (typeof window !== 'undefined') {
      const fromHash = window.location.hash.replace('#', '') as TabId;
      if (tabs.some((t) => t.id === fromHash)) activeTab = fromHash;
      window.addEventListener('hashchange', () => {
        const next = window.location.hash.replace('#', '') as TabId;
        if (tabs.some((t) => t.id === next)) activeTab = next;
      });
    }
  });
  function selectTab(id: TabId) {
    activeTab = id;
    if (typeof window !== 'undefined') {
      replaceState(`${window.location.pathname}#${id}`, {});
    }
  }
</script>

<div class="settings">
  <header class="settings__head">
    <h1 class="text-xl font-semibold">Settings</h1>
    <p class="text-xs text-fg2">Local-only preferences. Nothing leaves this device.</p>
  </header>

  {#if err}<div class="card p-3 text-sm text-crit">{err}</div>{/if}

  <div class="settings__shell">
    <!-- ─── Tab nav ────────────────────────────────────────────────── -->
    <nav class="settings__nav" aria-label="Settings sections">
      {#each tabs as t}
        <button type="button"
                class="settings__tab {activeTab === t.id ? 'settings__tab--active' : ''}"
                onclick={() => selectTab(t.id)}>
            <span class="settings__tab-icon"><Icon name={t.icon} size={16} /></span>
          <span class="settings__tab-body">
            <span class="settings__tab-label">{t.label}</span>
            <span class="settings__tab-hint">{t.hint}</span>
          </span>
        </button>
      {/each}
    </nav>

    <!-- ─── Content pane ───────────────────────────────────────────── -->
    <div class="settings__pane">
      {#if activeTab === 'appearance'}
        <!-- ─── Theme mode ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Theme</h2>
            <p class="text-xs text-fg2">Light, dark, or follow your OS preference.</p>
          </div>
          <div class="seg">
            {#each ['system', 'light', 'dark'] as m}
              <button type="button"
                      class="seg__opt {theme.mode === m ? 'seg__opt--on' : ''}"
                      onclick={() => theme.set(m as 'system' | 'light' | 'dark')}>
                {#if m === 'system'}<Icon name="monitor" size={14} /> System{:else if m === 'light'}<Icon name="sun" size={14} /> Light{:else}<Icon name="moon" size={14} /> Dark{/if}
              </button>
            {/each}
          </div>
          <p class="text-xs text-fg3">Resolved: <strong>{theme.resolved}</strong></p>
        </section>

        <!-- ─── Accent color ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Accent color</h2>
            <p class="text-xs text-fg2">
              Drives every interactive surface — buttons, links, focus rings, the primary chart series, the splash mark.
              Picks light + dark variants of the same hue automatically.
            </p>
          </div>
          <div class="swatches">
            {#each ACCENT_PRESETS as preset}
              <button type="button"
                      class="swatch {appearance.accent === preset.id ? 'swatch--on' : ''}"
                      style="--swatch-color: {preset.swatch};"
                      onclick={() => appearance.setAccent(preset.id as AccentName)}
                      title={preset.label}
                      aria-label="Accent: {preset.label}"
                      aria-pressed={appearance.accent === preset.id}>
                <span class="swatch__dot"></span>
                <span class="swatch__name">{preset.label}</span>
              </button>
            {/each}
          </div>
        </section>

        <!-- ─── Density / spacing ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Density</h2>
            <p class="text-xs text-fg2">
              Adjusts global card padding and line-height. Compact suits dense tables; comfortable
              gives every reading room to breathe.
            </p>
          </div>
          <div class="seg">
            {#each [
              { id: 'comfortable', label: '↕ Comfortable' },
              { id: 'compact',     label: '↕ Compact' }
            ] as opt}
              <button type="button"
                      class="seg__opt {appearance.density === opt.id ? 'seg__opt--on' : ''}"
                      onclick={() => appearance.setDensity(opt.id as Density)}>
                {opt.label}
              </button>
            {/each}
          </div>
        </section>

        <!-- ─── Font scale ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Font size</h2>
            <p class="text-xs text-fg2">
              Scales the entire UI by adjusting the root font size. Tailwind text classes derive from
              this, so cards and tables scale together.
            </p>
          </div>
          <div class="seg">
            {#each [
              { id: 'tiny', label: 'Aa Tiny  · 12 px' },
              { id: 'sm', label: 'Aa Small  · 14 px' },
              { id: 'md', label: 'Aa Normal · 15 px' },
              { id: 'lg', label: 'Aa Large  · 16 px' },
              { id: 'gigantic', label: 'Aa Gigantic · 20 px' }
            ] as opt}
              <button type="button"
                      class="seg__opt {appearance.fontScale === opt.id ? 'seg__opt--on' : ''}"
                      onclick={() => appearance.setFontScale(opt.id as FontScale)}>
                {opt.label}
              </button>
            {/each}
          </div>
        </section>

        <!-- ─── Font family ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Body font</h2>
            <p class="text-xs text-fg2">
              The typeface for body copy. Numeric columns and chart axes always use a monospace
              regardless, so this only affects prose.
            </p>
          </div>
          <div class="seg">
            {#each [
              { id: 'sans',  label: 'Sans (default)' },
              { id: 'serif', label: 'Serif' },
              { id: 'mono',  label: 'Mono' }
            ] as opt}
              <button type="button"
                      class="seg__opt {appearance.fontFamily === opt.id ? 'seg__opt--on' : ''}"
                      onclick={() => appearance.setFontFamily(opt.id as FontFamily)}>
                {opt.label}
              </button>
            {/each}
          </div>
        </section>

        <!-- ─── Motion ─── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Motion</h2>
          </div>
          <label class="row" style="border-top: 0; padding-top: 0;">
            <div class="row__body">
              <div class="row__title">Reduce motion</div>
              <div class="row__hint">
                Disables splash animations, gradient blobs, and chart transitions. Honoured in addition
                to the system <code class="font-mono text-[10px]">prefers-reduced-motion</code> hint.
              </div>
            </div>
            <input type="checkbox" checked={appearance.reduceMotion}
                   onchange={(e) => appearance.setReduceMotion((e.target as HTMLInputElement).checked)} />
          </label>
        </section>

        <!-- ─── Reset ─── -->
        <section class="card p-5 space-y-2">
          <div class="flex items-baseline justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">Reset appearance</h2>
              <p class="text-xs text-fg2">Restores the violet / comfortable / normal / sans defaults. Theme mode is unchanged.</p>
            </div>
            <button class="btn" onclick={() => appearance.reset()}>Reset</button>
          </div>
        </section>
      {/if}

      {#if activeTab === 'dashboard'}
        <section class="card p-5 space-y-4">
          <div>
            <h2 class="text-sm font-semibold">Dashboard layout</h2>
            <p class="text-xs text-fg2">
              Choose which sections appear on the home dashboard and use the arrows to set their order.
              Preferences are stored locally with the rest of your encrypted app settings.
            </p>
          </div>

          <div class="dashboard-sections">
            {#each dashboardPrefs.order as section, index (section)}
              {@const meta = DASHBOARD_SECTIONS.find((item) => item.id === section)!}
              <div class="dashboard-section-row">
                <label class="row dashboard-section-row__toggle">
                  <input type="checkbox" checked={dashboardPrefs.visible[section]} onchange={() => dashboardPrefs.toggle(section)} />
                  <span class="row__body">
                    <span class="row__title">{meta.label}</span>
                    <span class="row__hint">{meta.hint}</span>
                  </span>
                </label>
                <div class="dashboard-section-row__actions" aria-label="Reorder {meta.label}">
                  <button type="button" class="mini-btn" disabled={index === 0} onclick={() => dashboardPrefs.move(section, -1)} title="Move up" aria-label="Move {meta.label} up"><Icon name="chevron-up" size={14} /></button>
                  <button type="button" class="mini-btn" disabled={index === dashboardPrefs.order.length - 1} onclick={() => dashboardPrefs.move(section, 1)} title="Move down" aria-label="Move {meta.label} down"><Icon name="chevron-down" size={14} /></button>
                </div>
              </div>
            {/each}
          </div>
        </section>

        <section class="card p-5 space-y-4">
          <div>
            <h2 class="text-sm font-semibold">Dashboard item limits</h2>
            <p class="text-xs text-fg2">These limits only change how many cards or rows are shown; the underlying records stay untouched.</p>
          </div>
          <label class="row">
            <span class="row__body"><span class="row__title">Recent abnormal flags</span><span class="row__hint">Maximum patient cards in the spotlight.</span></span>
            <input class="settings-number" type="number" min="1" max="50" value={dashboardPrefs.spotlightLimit} onchange={(e) => dashboardPrefs.setLimit('spotlight', Number((e.currentTarget as HTMLInputElement).value))} />
          </label>
          <label class="row">
            <span class="row__body"><span class="row__title">Recent reports</span><span class="row__hint">Newest imported reports listed on the dashboard.</span></span>
            <input class="settings-number" type="number" min="1" max="50" value={dashboardPrefs.reportLimit} onchange={(e) => dashboardPrefs.setLimit('reports', Number((e.currentTarget as HTMLInputElement).value))} />
          </label>
          <label class="row">
            <span class="row__body"><span class="row__title">Recent activity</span><span class="row__hint">Latest audit events requested for the dashboard.</span></span>
            <input class="settings-number" type="number" min="1" max="50" value={dashboardPrefs.activityLimit} onchange={(e) => dashboardPrefs.setLimit('activity', Number((e.currentTarget as HTMLInputElement).value))} />
          </label>
          <label class="row">
            <span class="row__body"><span class="row__title">Patients</span><span class="row__hint">Patient cards shown after sorting by most recent activity.</span></span>
            <select class="settings-number" value={dashboardPrefs.patientLimit} onchange={(e) => dashboardPrefs.setLimit('patients', Number((e.currentTarget as HTMLSelectElement).value))}>
              <option value="0">All patients</option>
              <option value="6">6 patients</option>
              <option value="12">12 patients</option>
              <option value="24">24 patients</option>
              <option value="50">50 patients</option>
              <option value="100">100 patients</option>
            </select>
          </label>
          <div class="flex justify-end pt-1">
            <button type="button" class="btn" onclick={() => dashboardPrefs.reset()}>Reset dashboard</button>
          </div>
        </section>
      {/if}

      {#if activeTab === 'charts'}
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Chart interaction</h2>
            <p class="text-xs text-fg2">
              Auxiliary scrollers and wheel-zoom are off by default. Drag-to-pan inside the
              plot area always works, and the toolbar above each chart exposes the same toggles
              for one-off use.
            </p>
          </div>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Timeline range slider</div>
              <div class="row__hint">Adds a draggable horizontal stretcher under the chart for X-axis windowing.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showXSlider}
                   onchange={(e) => chartPrefs.setX((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Vertical units slider</div>
              <div class="row__hint">Adds a vertical stretcher on the right edge for Y-axis windowing.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showYSlider}
                   onchange={(e) => chartPrefs.setY((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Mouse-wheel zoom inside chart</div>
              <div class="row__hint">
                Off by default — the wheel scrolls the page. Turn on to zoom the chart with the wheel
                (and Shift+wheel for the Y axis).
              </div>
            </div>
            <input type="checkbox" checked={chartPrefs.scrollZoom}
                   onchange={(e) => chartPrefs.setScrollZoom((e.target as HTMLInputElement).checked)} />
          </label>
        </section>

        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Reference range source</h2>
            <p class="text-xs text-fg2">
              Which range drives every flag pill, chart band, and reference card.
              Affects all analytes globally; the parser still captures every printed range
              regardless so you can switch back at any time without re-ingesting.
            </p>
          </div>
          <div class="seg">
            {#each [
              { id: 'auto',    label: 'Auto',       icon: 'scale'   as IconName },
              { id: 'library', label: 'Library',    icon: 'library' as IconName },
              { id: 'printed', label: 'Per-report', icon: 'receipt' as IconName }
            ] as opt}
              <button type="button"
                      class="seg__opt {chartPrefs.referenceSource === opt.id ? 'seg__opt--on' : ''}"
                      onclick={() => chartPrefs.setReferenceSource(opt.id as 'auto' | 'library' | 'printed')}>
                <Icon name={opt.icon} size={14} /> {opt.label}
              </button>
            {/each}
          </div>
          <ul class="text-xs text-fg2 space-y-1 pt-1">
            <li><strong>Auto</strong> (default): library wins when the ontology has any usable reference (sex/cycle/tier/universal); falls back to the lab-printed range otherwise.</li>
            <li><strong>Library</strong>: always derive from the analyte ontology — sex- and cycle-aware where applicable. Printed ranges from the PDF are ignored for flag computation.</li>
            <li><strong>Per-report</strong>: trust whatever range the lab printed on each individual report, even when the ontology has a more specific default. Useful when your lab uses non-standard cutoffs you want to honour exactly.</li>
          </ul>
        </section>

        <!-- ───── Default chart appearance ───── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Default chart appearance</h2>
            <p class="text-xs text-fg2">
              Every toolbar toggle on a chart writes through to the corresponding default
              here, so what you set per-chart sticks app-wide. Reset to the bundled defaults
              with the button at the bottom.
            </p>
          </div>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Show value labels by default</div>
              <div class="row__hint">Pinned numeric label at every point. Useful at low cadence; gets noisy with many readings.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showValues}
                   onchange={(e) => chartPrefs.setShowValues((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Smoothed line by default</div>
              <div class="row__hint">Bezier-interpolated segments. Off by default — straight segments make stepwise changes obvious.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.smooth}
                   onchange={(e) => chartPrefs.setSmooth((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Reference bands by default</div>
              <div class="row__hint">Coloured horizontal stripe(s) marking the normal / borderline / critical ranges.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showBands}
                   onchange={(e) => chartPrefs.setShowBands((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Symbols at each reading</div>
              <div class="row__hint">Off → just the line. Useful for very dense series where every-point markers crowd the canvas.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showSymbols}
                   onchange={(e) => chartPrefs.setShowSymbols((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Use report nicknames on the X axis</div>
              <div class="row__hint">When the source report has a nickname, replace the date with it. Falls back to the date for unlabelled reports.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.useNicknames}
                   onchange={(e) => chartPrefs.setUseNicknames((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Trend line</div>
              <div class="row__hint">Overlay a dashed linear-regression line over the visible points.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showTrendLine}
                   onchange={(e) => chartPrefs.setShowTrendLine((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Today reference line</div>
              <div class="row__hint">Vertical dashed marker at today's date.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showTodayLine}
                   onchange={(e) => chartPrefs.setShowTodayLine((e.target as HTMLInputElement).checked)} />
          </label>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Y-axis scale</div>
              <div class="row__hint">Logarithmic clamps to <code>linear</code> automatically when any value is ≤ 0.</div>
            </div>
            <div class="seg">
              {#each [
                { id: 'linear', label: 'Linear' },
                { id: 'log',    label: 'Log'    }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.scale === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setScale(opt.id as 'linear' | 'log')}>{opt.label}</button>
              {/each}
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Date format</div>
              <div class="row__hint">How dates appear on the X-axis when nicknames aren't being used.</div>
            </div>
            <div class="seg seg--nowrap">
              {#each [
                { id: 'iso',       label: '2024-09-07' },
                { id: 'short',     label: 'Sep 07'     },
                { id: 'monthYear', label: "Sep '24"    },
                { id: 'numeric',   label: '09/07/24'   }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.dateFormat === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setDateFormat(opt.id as 'iso' | 'short' | 'monthYear' | 'numeric')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Grid density</div>
              <div class="row__hint">Off hides every gridline. Detailed adds minor ticks for fine-grained reading.</div>
            </div>
            <div class="seg">
              {#each [
                { id: 'off',       label: 'Off'      },
                { id: 'standard',  label: 'Standard' },
                { id: 'detailed',  label: 'Detailed' }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.gridDensity === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setGridDensity(opt.id as 'off' | 'standard' | 'detailed')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Line width</div>
              <div class="row__hint">Stroke thickness in pixels.</div>
            </div>
            <input class="input w-20 text-right" type="number" min="0.5" max="6" step="0.1"
                   value={chartPrefs.lineWidth}
                   oninput={(e) => {
                     const v = parseFloat((e.target as HTMLInputElement).value);
                     if (Number.isFinite(v) && v > 0) chartPrefs.setLineWidth(v);
                   }} />
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Symbol size</div>
              <div class="row__hint">Diameter of the dot at each reading, in pixels.</div>
            </div>
            <input class="input w-20 text-right" type="number" min="2" max="20" step="1"
                   value={chartPrefs.symbolSize}
                   oninput={(e) => {
                     const v = parseFloat((e.target as HTMLInputElement).value);
                     if (Number.isFinite(v) && v > 0) chartPrefs.setSymbolSize(v);
                   }} />
          </div>
        </section>

        <!-- ───── Series rendering ───── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Series rendering</h2>
            <p class="text-xs text-fg2">
              How the data line is drawn. Bar mode disables smoothing and step interpolation
              (those only apply to line / area).
            </p>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Series type</div>
              <div class="row__hint">Line keeps the chart light; area fills the region beneath the line; bar shows discrete vertical bars.</div>
            </div>
            <div class="seg">
              {#each [
                { id: 'line', label: 'Line' },
                { id: 'area', label: 'Area' },
                { id: 'bar',  label: 'Bar'  }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.seriesType === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setSeriesType(opt.id as 'line' | 'area' | 'bar')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Step interpolation</div>
              <div class="row__hint">
                None = straight (or smoothed) segments. Start / Middle / End render staircase
                steps anchored at the indicated edge of each interval — useful when readings
                represent values that hold steady between samples.
              </div>
            </div>
            <div class="seg seg--nowrap">
              {#each [
                { id: 'none',   label: 'None'   },
                { id: 'start',  label: 'Start'  },
                { id: 'middle', label: 'Middle' },
                { id: 'end',    label: 'End'    }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.step === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setStep(opt.id as 'none' | 'start' | 'middle' | 'end')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Colour points by flag</div>
              <div class="row__hint">Tints each reading marker by its low / normal / high / critical flag, on top of the line colour.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.colorByFlag}
                   onchange={(e) => chartPrefs.setColorByFlag((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Connect across gaps</div>
              <div class="row__hint">When off, missing readings break the line into segments; when on, the line bridges any null/undefined value.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.connectNulls}
                   onchange={(e) => chartPrefs.setConnectNulls((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Mean line</div>
              <div class="row__hint">Horizontal dashed line at the mean of the visible readings.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showMeanLine}
                   onchange={(e) => chartPrefs.setShowMeanLine((e.target as HTMLInputElement).checked)} />
          </label>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Min &amp; max markers</div>
              <div class="row__hint">Pin labels at the highest and lowest readings.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showMinMaxMarkers}
                   onchange={(e) => chartPrefs.setShowMinMaxMarkers((e.target as HTMLInputElement).checked)} />
          </label>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Reference band opacity</div>
              <div class="row__hint">100% = bundled token alpha. 0% hides the fill entirely; 200% doubles its boldness.</div>
            </div>
            <div class="flex items-center gap-2">
              <input class="input w-20 text-right" type="number" min="0" max="200" step="10"
                     value={chartPrefs.bandOpacityPct}
                     oninput={(e) => {
                       const v = parseFloat((e.target as HTMLInputElement).value);
                       if (Number.isFinite(v) && v >= 0) chartPrefs.setBandOpacityPct(v);
                     }} />
              <span class="text-xs text-fg3">%</span>
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Animation speed</div>
              <div class="row__hint">Off disables every chart transition for instant rendering.</div>
            </div>
            <div class="seg">
              {#each [
                { id: 'off',    label: 'Off'    },
                { id: 'fast',   label: 'Fast'   },
                { id: 'normal', label: 'Normal' },
                { id: 'slow',   label: 'Slow'   }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.animation === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setAnimation(opt.id as 'off' | 'fast' | 'normal' | 'slow')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>
        </section>

        <!-- ───── Tooltip & chrome ───── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Tooltip &amp; chrome</h2>
          </div>
          <div class="row">
            <div class="row__body">
              <div class="row__title">Tooltip mode</div>
              <div class="row__hint">
                Axis = crosshair pointer with the nearest reading; Item = hover only the
                exact point underneath the cursor.
              </div>
            </div>
            <div class="seg">
              {#each [
                { id: 'axis', label: 'Axis (crosshair)' },
                { id: 'item', label: 'Item (point)'     }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.tooltipMode === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setTooltipMode(opt.id as 'axis' | 'item')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>

          <label class="row">
            <div class="row__body">
              <div class="row__title">Show chart titles</div>
              <div class="row__hint">When off, the chart's `title` prop is ignored — useful when the page already provides the heading.</div>
            </div>
            <input type="checkbox" checked={chartPrefs.showTitle}
                   onchange={(e) => chartPrefs.setShowTitle((e.target as HTMLInputElement).checked)} />
          </label>

          <div class="row">
            <div class="row__body">
              <div class="row__title">X-axis label overflow</div>
              <div class="row__hint">
                What to do when too many dates would collide on the time axis.
                <strong>Auto</strong> hides overlapping labels (date still on hover).
                <strong>Rotate</strong> tilts each label 35° so more fit before clipping.
                <strong>Hide</strong> never draws axis labels — cleanest visual; rely on hover for the exact date.
              </div>
            </div>
            <div class="seg seg--nowrap">
              {#each [
                { id: 'auto',   label: 'Auto'   },
                { id: 'rotate', label: 'Rotate' },
                { id: 'hide',   label: 'Hide'   }
              ] as opt}
                <button type="button"
                        class="seg__opt {chartPrefs.xLabelMode === opt.id ? 'seg__opt--on' : ''}"
                        onclick={() => chartPrefs.setXLabelMode(opt.id as 'auto' | 'rotate' | 'hide')}>
                  {opt.label}
                </button>
              {/each}
            </div>
          </div>
        </section>

        <!-- ───── Default time window ───── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Default time window</h2>
            <p class="text-xs text-fg2">
              At chart open, narrow the visible range to the most recent N. Older readings
              are still on file and reachable via slider/wheel zoom-out — this just controls
              the first paint.
            </p>
          </div>
          <div class="seg seg--wrap">
            {#each [
              { id: 'all', label: 'All'        },
              { id: '30d', label: '30 days'    },
              { id: '90d', label: '90 days'    },
              { id: '6m',  label: '6 months'   },
              { id: '1y',  label: '1 year'     },
              { id: '2y',  label: '2 years'    },
              { id: '5y',  label: '5 years'    }
            ] as opt}
              <button type="button"
                      class="seg__opt {chartPrefs.timeWindow === opt.id ? 'seg__opt--on' : ''}"
                      onclick={() => chartPrefs.setTimeWindow(opt.id as 'all' | '30d' | '90d' | '6m' | '1y' | '2y' | '5y')}>
                {opt.label}
              </button>
            {/each}
          </div>
        </section>

        <!-- ───── Default zoom & axis padding ───── -->
        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Default zoom &amp; axis padding</h2>
            <p class="text-xs text-fg2">
              How much breathing room the axes leave around the data. The X axis is single-stage
              — the chart always opens flush against the data. The Y axis is two-stage: the
              "axis padding" is the maximum zoom-out, and the "default visible padding" is how
              snug it opens.
            </p>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">X-axis padding</div>
              <div class="row__hint">Tight margin past the first/last point on the time axis. 0% pins the line edge-to-edge.</div>
            </div>
            <div class="flex items-center gap-2">
              <input class="input w-20 text-right" type="number" min="0" max="50" step="1"
                     value={chartPrefs.xAxisPadPct}
                     oninput={(e) => {
                       const v = parseFloat((e.target as HTMLInputElement).value);
                       if (Number.isFinite(v) && v >= 0) chartPrefs.setXAxisPadPct(v);
                     }} />
              <span class="text-xs text-fg3">%</span>
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Y-axis maximum padding</div>
              <div class="row__hint">How far past the data the user can zoom OUT vertically. Larger = more headroom for reference bands that extend beyond the data.</div>
            </div>
            <div class="flex items-center gap-2">
              <input class="input w-20 text-right" type="number" min="0" max="200" step="5"
                     value={chartPrefs.yAxisPadPct}
                     oninput={(e) => {
                       const v = parseFloat((e.target as HTMLInputElement).value);
                       if (Number.isFinite(v) && v >= 0) chartPrefs.setYAxisPadPct(v);
                     }} />
              <span class="text-xs text-fg3">%</span>
            </div>
          </div>

          <div class="row">
            <div class="row__body">
              <div class="row__title">Y-axis default visible padding</div>
              <div class="row__hint">How snug the chart opens vertically. Lower = data fills the plot at first paint.</div>
            </div>
            <div class="flex items-center gap-2">
              <input class="input w-20 text-right" type="number" min="0" max="50" step="1"
                     value={chartPrefs.yVisiblePadPct}
                     oninput={(e) => {
                       const v = parseFloat((e.target as HTMLInputElement).value);
                       if (Number.isFinite(v) && v >= 0) chartPrefs.setYVisiblePadPct(v);
                     }} />
              <span class="text-xs text-fg3">%</span>
            </div>
          </div>
        </section>

        <!-- ───── Reset chart defaults ───── -->
        <section class="card p-4 flex items-center justify-between gap-3">
          <div>
            <h2 class="text-sm font-semibold">Reset chart defaults</h2>
            <p class="text-xs text-fg2">Restore every chart preference on this page (and the toolbar toggles) to bundled defaults.</p>
          </div>
          <button class="btn" onclick={() => chartPrefs.resetAll()}>Reset</button>
        </section>
      {/if}

      {#if activeTab === 'comparison'}
        <!-- ───── Compare presets editor ───── -->
        <section class="card p-5 space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <div>
              <h2 class="text-sm font-semibold">Comparison presets</h2>
              <p class="text-xs text-fg2">
                Manage the preset buttons that appear above the analyte picker on the Compare page.
                Bundled presets can be customised; your edits live as overrides so "reset to defaults"
                always brings them back. User-created presets are saved alongside.
                Presets can also carry filter overrides (date range, flag filter, HRT anchor, etc.)
                so e.g. "Iron panel · last 12 months" is one click.
              </p>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn text-xs" onclick={startCreatePreset}><Icon name="plus" size={13} /> New preset</button>
              <button class="btn text-xs"
                      onclick={resetAllPresets}
                      disabled={Object.keys(comparePresets.bundledOverrides).length === 0}
                      title="Restore every bundled preset to its original definition. User presets are kept.">
                <Icon name="refresh" size={13} /> Reset bundled
              </button>
            </div>
          </div>

          <!-- Preset list -->
          <ul class="divide-y divide-line border border-line rounded">
            {#each comparePresets.presets as p (p.id)}
              {@const editing = presetEditingId === p.id}
              {@const overridden = p.source === 'bundled' && comparePresets.isBundledOverridden(p.id)}
              {@const hasFilters = !!p.filters && Object.keys(p.filters).length > 0}
              <li class="px-3 py-2 text-xs">
                <div class="flex items-center justify-between gap-3 flex-wrap">
                  <div class="flex flex-col min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-fg1">{p.name}</span>
                      {#if p.kind === 'dynamic'}
                        <span class="pill-muted text-[10px]" title="Patient-aware: analyte list is filled at runtime.">dynamic</span>
                      {/if}
                      {#if p.source === 'user'}
                        <span class="pill-warn text-[10px]">user</span>
                      {:else if overridden}
                        <span class="pill-warn text-[10px]">customised</span>
                      {:else}
                        <span class="pill-muted text-[10px]">bundled</span>
                      {/if}
                      {#if hasFilters}
                        <span class="pill-muted text-[10px] text-warn inline-flex items-center gap-1" title="Sets filter overrides on click"><Icon name="filter" size={11} /> filters</span>
                      {/if}
                    </div>
                    <div class="text-fg3 text-[11px] mt-0.5">
                      {#if p.kind === 'static'}
                        {p.ids.length} analyte{p.ids.length === 1 ? '' : 's'}
                        {#if p.ids.length > 0}
                          · <span class="font-mono">{p.ids.slice(0, 6).join(', ')}{p.ids.length > 6 ? '…' : ''}</span>
                        {/if}
                      {:else}
                        Resolved at runtime · <span class="font-mono">{p.dataSource}</span>
                      {/if}
                    </div>
                  </div>
                  <div class="flex items-center gap-1">
                    <button class="btn text-[11px] px-2 py-1" onclick={() => editing ? cancelEditPreset() : startEditPreset(p)}>
                      {editing ? 'Cancel' : 'Edit'}
                    </button>
                    {#if overridden}
                      <button class="btn text-[11px] px-2 py-1" onclick={() => resetPreset(p)} title="Restore bundled defaults"><Icon name="refresh" size={14} /></button>
                    {/if}
                    {#if p.source === 'user'}
                      <button class="btn text-[11px] px-2 py-1 text-crit border-crit/40 hover:bg-crit/10"
                              onclick={() => deletePreset(p)} title="Delete this preset"><Icon name="trash" size={14} /></button>
                    {/if}
                  </div>
                </div>

                <!-- Inline editor -->
                {#if editing}
                  <div class="mt-3 pt-3 border-t border-line space-y-3">
                    <label class="block">
                      <span class="text-[11px] text-fg3 block mb-1">Display name</span>
                      <input class="input w-full" type="text" bind:value={editorForm.name} placeholder="e.g. Iron panel" />
                    </label>

                    {#if p.kind === 'static'}
                      <div>
                        <span class="text-[11px] text-fg3 block mb-1">Analytes ({editorForm.ids.length})</span>
                        <!-- Picked list with reorder + remove -->
                        {#if editorForm.ids.length > 0}
                          <ul class="border border-line rounded mb-2 divide-y divide-line">
                            {#each editorForm.ids as aid, idx (aid)}
                              {@const meta = allAnalytes.find((x) => x.id === aid)}
                              <li class="flex items-center justify-between px-2 py-1 text-[11px]">
                                <span class="truncate">
                                  <span class="font-medium">{meta?.pt_name ?? aid}</span>
                                  <span class="text-fg3 ml-1 font-mono">{aid}</span>
                                </span>
                                <span class="flex items-center gap-1 shrink-0">
                                  <button class="reorder-btn-mini" onclick={() => moveEditorAnalyte(idx, -1)} disabled={idx === 0} title="Move up"><Icon name="arrow-up" size={12} /></button>
                                  <button class="reorder-btn-mini" onclick={() => moveEditorAnalyte(idx, +1)} disabled={idx === editorForm.ids.length - 1} title="Move down"><Icon name="arrow-down" size={12} /></button>
                                  <button class="reorder-btn-mini text-crit" onclick={() => toggleEditorAnalyte(aid)} title="Remove"><Icon name="x" size={12} /></button>
                                </span>
                              </li>
                            {/each}
                          </ul>
                        {/if}
                        <input class="input w-full" type="text" placeholder="Search analytes to add…" bind:value={editorForm.analyteSearch} />
                        <div class="border border-line rounded mt-1 max-h-40 overflow-y-auto">
                          {#each allAnalytes
                            .filter((a) => !a.is_panel_header)
                            .filter((a) => !editorForm.ids.includes(a.id))
                            .filter((a) => {
                              const q = editorForm.analyteSearch.trim().toLowerCase();
                              return !q || a.pt_name.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
                            })
                            .slice(0, 40) as a (a.id)}
                            <button type="button"
                                    class="w-full flex items-center justify-between gap-2 px-2 py-1 text-left text-[11px] hover:bg-bg2 transition-colors"
                                    onclick={() => toggleEditorAnalyte(a.id)}>
                              <span class="truncate">
                                <span class="font-medium">{a.pt_name}</span>
                                <span class="text-fg3 ml-1 font-mono">{a.id}</span>
                              </span>
                              <span class="text-accent text-[10px]">＋ add</span>
                            </button>
                          {:else}
                            <div class="px-2 py-2 text-[11px] text-fg3 text-center">No analytes match.</div>
                          {/each}
                        </div>
                      </div>
                    {:else}
                      <p class="text-[11px] text-fg3 italic">
                        Dynamic preset — the analyte list is resolved per-patient at runtime
                        ({p.dataSource}). You can rename it and attach filters here.
                      </p>
                    {/if}

                    <!-- Filter overrides -->
                    <div>
                      <label class="flex items-center gap-2 cursor-pointer text-xs text-fg2 mb-2">
                        <input type="checkbox" bind:checked={editorForm.useFilters} />
                        <span>
                          Set filters when this preset is applied
                          <span class="text-fg3 block text-[10px]">When off, applying the preset only changes the picked analytes; current filters stay as-is.</span>
                        </span>
                      </label>
                      {#if editorForm.useFilters}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-x-3 gap-y-2 pl-6">
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Date from</span>
                            <input class="input" type="date" bind:value={editorForm.filters.dateFromIso} />
                          </label>
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Date until</span>
                            <input class="input" type="date" bind:value={editorForm.filters.dateUntilIso} />
                          </label>
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Value min</span>
                            <input class="input" type="number" inputmode="decimal" bind:value={editorForm.filters.valueMin} />
                          </label>
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Value max</span>
                            <input class="input" type="number" inputmode="decimal" bind:value={editorForm.filters.valueMax} />
                          </label>
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Last N per analyte (0 = all)</span>
                            <input class="input" type="number" min="0" bind:value={editorForm.filters.lastNPerAnalyte} />
                          </label>
                          <label class="flex flex-col gap-1 text-[11px] text-fg3">
                            <span>HRT anchor</span>
                            <select class="select" bind:value={editorForm.filters.hrtFilter}>
                              <option value="all">All readings</option>
                              <option value="pre">Pre-HRT</option>
                              <option value="post">Post-HRT</option>
                            </select>
                          </label>
                          <label class="md:col-span-2 flex flex-col gap-1 text-[11px] text-fg3">
                            <span>Exclude report IDs (comma/space-separated)</span>
                            <input class="input font-mono" type="text" bind:value={editorForm.filters.excludeReportIds} />
                          </label>
                          <div class="md:col-span-2 grid grid-cols-2 gap-x-3 gap-y-1 text-[11px]">
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.includeInlinePriors} /><span>Include inline-prior</span></label>
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.intersectOnly} /><span>Intersection only</span></label>
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyNormal} /><span>Normal only</span></label>
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyAbnormal} /><span class="text-warn">Abnormal</span></label>
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyCritical} /><span class="text-crit">Critical</span></label>
                            <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyUnflagged} /><span class="text-fg3">Unflagged</span></label>
                          </div>
                        </div>
                      {/if}
                    </div>

                    <div class="flex justify-end gap-2 pt-1">
                      <button class="btn" onclick={cancelEditPreset}>Cancel</button>
                      <button class="btn-accent" onclick={savePreset}
                              disabled={!editorForm.name.trim() || (p.kind === 'static' && editorForm.ids.length === 0 && !editorForm.useFilters && creating)}>
                        {creating ? 'Create preset' : 'Save changes'}
                      </button>
                    </div>
                  </div>
                {/if}
              </li>
            {/each}

            <!-- New-preset row when creating with no existing match. -->
            {#if creating && presetEditingId === '__new__'}
              <li class="px-3 py-2 text-xs bg-bg2/50">
                <div class="flex items-center justify-between gap-3 flex-wrap">
                  <div>
                    <span class="font-medium text-fg1">New preset</span>
                    <span class="pill-warn text-[10px] ml-2">user · draft</span>
                  </div>
                  <div class="flex items-center gap-1">
                    <button class="btn text-[11px] px-2 py-1" onclick={cancelEditPreset}>Cancel</button>
                  </div>
                </div>
                <div class="mt-3 pt-3 border-t border-line space-y-3">
                  <label class="block">
                    <span class="text-[11px] text-fg3 block mb-1">Display name</span>
                    <!-- svelte-ignore a11y_autofocus — landing focus on the
                         Name field is the desired UX for "create preset". -->
                    <input class="input w-full" type="text" bind:value={editorForm.name} placeholder="e.g. Liver — last year only" autofocus />
                  </label>
                  <div>
                    <span class="text-[11px] text-fg3 block mb-1">Analytes ({editorForm.ids.length})</span>
                    {#if editorForm.ids.length > 0}
                      <ul class="border border-line rounded mb-2 divide-y divide-line">
                        {#each editorForm.ids as aid, idx (aid)}
                          {@const meta = allAnalytes.find((x) => x.id === aid)}
                          <li class="flex items-center justify-between px-2 py-1 text-[11px]">
                            <span class="truncate">
                              <span class="font-medium">{meta?.pt_name ?? aid}</span>
                              <span class="text-fg3 ml-1 font-mono">{aid}</span>
                            </span>
                            <span class="flex items-center gap-1 shrink-0">
                              <button class="reorder-btn-mini" onclick={() => moveEditorAnalyte(idx, -1)} disabled={idx === 0}><Icon name="arrow-up" size={12} /></button>
                              <button class="reorder-btn-mini" onclick={() => moveEditorAnalyte(idx, +1)} disabled={idx === editorForm.ids.length - 1}><Icon name="arrow-down" size={12} /></button>
                              <button class="reorder-btn-mini text-crit" onclick={() => toggleEditorAnalyte(aid)}><Icon name="x" size={12} /></button>
                            </span>
                          </li>
                        {/each}
                      </ul>
                    {/if}
                    <input class="input w-full" type="text" placeholder="Search analytes to add…" bind:value={editorForm.analyteSearch} />
                    <div class="border border-line rounded mt-1 max-h-40 overflow-y-auto">
                      {#each allAnalytes
                        .filter((a) => !a.is_panel_header)
                        .filter((a) => !editorForm.ids.includes(a.id))
                        .filter((a) => {
                          const q = editorForm.analyteSearch.trim().toLowerCase();
                          return !q || a.pt_name.toLowerCase().includes(q) || a.id.toLowerCase().includes(q);
                        })
                        .slice(0, 40) as a (a.id)}
                        <button type="button"
                                class="w-full flex items-center justify-between gap-2 px-2 py-1 text-left text-[11px] hover:bg-bg2 transition-colors"
                                onclick={() => toggleEditorAnalyte(a.id)}>
                          <span class="truncate">
                            <span class="font-medium">{a.pt_name}</span>
                            <span class="text-fg3 ml-1 font-mono">{a.id}</span>
                          </span>
                          <span class="text-accent text-[10px]">＋ add</span>
                        </button>
                      {:else}
                        <div class="px-2 py-2 text-[11px] text-fg3 text-center">No analytes match.</div>
                      {/each}
                    </div>
                  </div>

                  <div>
                    <label class="flex items-center gap-2 cursor-pointer text-xs text-fg2 mb-2">
                      <input type="checkbox" bind:checked={editorForm.useFilters} />
                      <span>
                        Set filters when this preset is applied
                        <span class="text-fg3 block text-[10px]">When off, applying the preset only changes the picked analytes; current filters stay as-is.</span>
                      </span>
                    </label>
                    {#if editorForm.useFilters}
                      <div class="grid grid-cols-1 md:grid-cols-2 gap-x-3 gap-y-2 pl-6">
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Date from</span>
                          <input class="input" type="date" bind:value={editorForm.filters.dateFromIso} />
                        </label>
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Date until</span>
                          <input class="input" type="date" bind:value={editorForm.filters.dateUntilIso} />
                        </label>
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Value min</span>
                          <input class="input" type="number" inputmode="decimal" bind:value={editorForm.filters.valueMin} />
                        </label>
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Value max</span>
                          <input class="input" type="number" inputmode="decimal" bind:value={editorForm.filters.valueMax} />
                        </label>
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Last N per analyte (0 = all)</span>
                          <input class="input" type="number" min="0" bind:value={editorForm.filters.lastNPerAnalyte} />
                        </label>
                        <label class="flex flex-col gap-1 text-[11px] text-fg3">
                          <span>HRT anchor</span>
                          <select class="select" bind:value={editorForm.filters.hrtFilter}>
                            <option value="all">All readings</option>
                            <option value="pre">Pre-HRT</option>
                            <option value="post">Post-HRT</option>
                          </select>
                        </label>
                        <label class="md:col-span-2 flex flex-col gap-1 text-[11px] text-fg3">
                          <span>Exclude report IDs (comma/space-separated)</span>
                          <input class="input font-mono" type="text" bind:value={editorForm.filters.excludeReportIds} />
                        </label>
                        <div class="md:col-span-2 grid grid-cols-2 gap-x-3 gap-y-1 text-[11px]">
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.includeInlinePriors} /><span>Include inline-prior</span></label>
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.intersectOnly} /><span>Intersection only</span></label>
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyNormal} /><span>Normal only</span></label>
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyAbnormal} /><span class="text-warn">Abnormal</span></label>
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyCritical} /><span class="text-crit">Critical</span></label>
                          <label class="flex items-center gap-2"><input type="checkbox" bind:checked={editorForm.filters.onlyUnflagged} /><span class="text-fg3">Unflagged</span></label>
                        </div>
                      </div>
                    {/if}
                  </div>

                  <div class="flex justify-end gap-2 pt-1">
                    <button class="btn" onclick={cancelEditPreset}>Cancel</button>
                    <button class="btn-accent" onclick={savePreset}
                            disabled={!editorForm.name.trim() || (editorForm.ids.length === 0 && !editorForm.useFilters)}>
                      Create preset
                    </button>
                  </div>
                </div>
              </li>
            {/if}
          </ul>
        </section>
      {/if}

      {#if activeTab === 'security'}
        <section class="card p-5 space-y-4">
          <div class="flex items-start gap-3">
            <span class="section-icon"><Icon name="shield" size={18} /></span>
            <div>
              <h2 class="text-sm font-semibold">Device security</h2>
              <p class="text-xs text-fg2 mt-1">Manage the protection layers used by this local instance. The encrypted SQLCipher database and managed PDF cache remain protected whether or not OS-vault unlock is enabled.</p>
            </div>
          </div>

          {#if securityStatus}
            <div class="security-status-grid">
              <div class="security-status-card">
                <span class="text-[11px] text-fg3">Native OS vault</span>
                <strong class={securityStatus.os_vault_enabled && securityStatus.os_vault_credential_present ? 'text-ok' : 'text-fg2'}>
                  {securityStatus.os_vault_enabled && securityStatus.os_vault_credential_present ? 'Enabled' : 'Not enabled'}
                </strong>
                <span class="text-[11px] text-fg3">{securityStatus.os_vault_platform}</span>
              </div>
              <div class="security-status-card">
                <span class="text-[11px] text-fg3">Session</span>
                <strong class={securityStatus.session_unlocked ? 'text-ok' : 'text-warn'}>{securityStatus.session_unlocked ? 'Unlocked' : 'Locked'}</strong>
                <span class="text-[11px] text-fg3">DMK held only in native memory</span>
              </div>
            </div>

            {#if securityStatus.last_error}
              <p class="row__hint row__hint--error">{securityStatus.last_error}</p>
            {/if}

            <div class="security-actions">
              {#if securityStatus.os_vault_enabled && securityStatus.os_vault_credential_present}
                <button class="btn" type="button" onclick={disableOsVault}>
                  <Icon name="lock" size={14} /> Disable OS vault
                </button>
                <label class="security-toggle">
                  <input type="checkbox" checked={securityStatus.os_vault_auto_unlock} onchange={(event) => toggleAutoUnlock((event.currentTarget as HTMLInputElement).checked)} />
                  <span><strong>Automatic unlock</strong><small>Unlock at launch when the OS credential store permits it.</small></span>
                </label>
              {:else}
                <button class="btn-accent" type="button" disabled={!securityStatus.os_vault_supported} onclick={enableOsVault}>
                  <Icon name="shield" size={14} /> Enable native OS vault
                </button>
                <span class="text-[11px] text-fg3">Enable this while unlocked to place a device-bound DMK copy in {securityStatus.os_vault_platform}.</span>
              {/if}
            </div>
          {:else}
            <p class="text-xs text-fg3">Loading security status…</p>
          {/if}

          <div class="security-notes">
            <p><strong>Protection model.</strong> Passwords and passkeys continue to work as recovery methods. The OS vault is an additional device-local wrapper, not a replacement for the encrypted database.</p>
            <p><strong>Shared-device warning.</strong> Anyone who can unlock this operating-system account may be able to use the optional automatic unlock setting. Keep it off on shared or unattended machines.</p>
          </div>
        </section>

        <section class="card p-5 space-y-2">
          <h2 class="text-sm font-semibold">Security checklist</h2>
          <ul class="text-xs text-fg2 space-y-1 list-disc pl-4">
            <li>Use a strong vault password and keep at least one recovery method available.</li>
            <li>Lock the app when stepping away from an unlocked session.</li>
            <li>Export backups to a separately protected location; exported PDFs and the database remain encrypted.</li>
          </ul>
        </section>
      {/if}

      {#if activeTab === 'ingestion'}
        <section class="card p-5 space-y-4">
          <div>
            <h2 class="text-sm font-semibold">PDF extraction (tier 1)</h2>
            <p class="text-xs text-fg2">Always-on baseline. Other tiers below are opt-in.</p>
          </div>
          {#if pdfium}
            <div class="row">
              <div class="row__body">
                <div class="row__title">pdfium</div>
                <div class="row__hint">
                  {pdfium.available ? 'Library bound. Ingestion ready.' : 'Library missing — ingestion will fail.'}
                </div>
                {#if pdfium.error}<div class="text-[11px] text-warn break-words mt-1">{pdfium.error}</div>{/if}
              </div>
              <span class={pdfium.available ? 'pill-ok' : 'pill-crit'}>{pdfium.available ? 'OK' : 'Missing'}</span>
            </div>
          {/if}
        </section>

        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">OCR / LLM tiers</h2>
            <p class="text-xs text-fg3">
              Each tier loads only when used. Toggle off to disable entirely. Compile with the
              relevant cargo feature to make a tier available.
            </p>
          </div>

          {#if tess}
            <label class="row">
              <div class="row__body">
                <div class="row__title">Tesseract OCR <span class="row__sub">no LLM</span></div>
                <div class="row__hint">
                  compiled: {tess.compiled ? 'yes' : 'no — rebuild with --features tesseract-ocr'} ·
                  languages: {tess.model_present ? 'present' : 'missing'} ·
                  runtime: {tess.loaded ? 'ready' : 'unavailable'}
                </div>
                {#if tess.last_error}
                  <div class="row__hint row__hint--error">{tess.last_error}</div>
                {/if}
                <div class="tier-resource-help">
                  <button class="mini-btn" disabled={tierDownload !== null} onclick={() => downloadTesseractLanguage('eng')}>
                    {tierDownload === 'tesseract' ? 'Downloading…' : 'Download eng data'}
                  </button>
                  <button class="mini-btn" disabled={tierDownload !== null} onclick={() => downloadTesseractLanguage('por')}>
                    {tierDownload === 'tesseract' ? 'Downloading…' : 'Download por data'}
                  </button>
                  <button class="mini-btn" onclick={() => visit('https://github.com/tesseract-ocr/tessdoc/blob/main/Downloads.md')}>Native binary guide</button>
                </div>
              </div>
              <input type="checkbox"
                bind:checked={tess.enabled_in_settings}
                disabled={!tess.compiled}
                onchange={(e) => toggleTier('tesseract', e.currentTarget.checked)} />
            </label>
          {/if}

          {#if llm}
            <div class="row">
              <div class="row__body">
                <div class="row__title">Phi-4-mini-reasoning <span class="row__sub">repair tier</span></div>
                <div class="row__hint">
                  compiled: {llm.compiled ? 'yes' : 'no — rebuild with --features embedded-llm'} ·
                  model: {llm.model_present ? 'present' : 'missing'} ·
                  loaded: {llm.loaded ? 'yes' : llm.loading ? 'loading' : 'no'}
                </div>
                <div class="model-path" title={modelPathHint('llm', llm)}>
                  model_path: {modelPathHint('llm', llm)}
                </div>
                {#if llm.loaded_model_path && llm.loaded_model_path !== configuredModelPath('llm', llm)}
                  <div class="model-path" title={llm.loaded_model_path}>loaded: {llm.loaded_model_path}</div>
                {/if}
                {#if llm.last_error}<div class="text-[11px] text-warn break-words mt-1">{llm.last_error}</div>{/if}
              </div>
              <div class="tier-actions">
                <button class="mini-btn"
                  disabled={!llm.compiled || tierDownload !== null}
                  onclick={() => downloadModel('llm')}>
                  {tierDownload === 'llm' ? 'Downloading…' : 'Download'}
                </button>
                <button class="mini-btn"
                  disabled={tierAction !== null}
                  onclick={() => chooseModelPath('llm')}>Browse</button>
                <button class="mini-btn"
                  disabled={!llm.compiled || !configuredModelPath('llm', llm) || llm.loaded || llm.loading || tierAction !== null}
                  onclick={() => runTierAction('llm', 'load', configuredModelPath('llm', llm))}>
                  {tierAction === 'llm' || llm.loading ? 'Loading' : 'Load'}
                </button>
                <button class="mini-btn"
                  disabled={!llm.loaded || llm.loading || tierAction !== null}
                  onclick={() => runTierAction('llm', 'unload')}>Unload</button>
                <input type="checkbox"
                  bind:checked={llm.enabled_in_settings}
                  disabled={!llm.compiled || !llm.model_present}
                  onchange={(e) => toggleTier('llm', e.currentTarget.checked)} />
              </div>
            </div>
          {/if}

          {#if olm}
            <div class="row">
              <div class="row__body">
                <div class="row__title">olmOCR-2 <span class="row__sub">vision OCR</span></div>
                <div class="row__hint">
                  compiled: {olm.compiled ? 'yes' : 'no — rebuild with --features embedded-ocr-vision'} ·
                  model: {olm.model_present ? 'present' : 'missing'} ·
                  loaded: {olm.loaded ? 'yes' : olm.loading ? 'loading' : 'no'}
                </div>
                <div class="model-path" title={modelPathHint('olmocr', olm)}>
                  model_path: {modelPathHint('olmocr', olm)}
                </div>
                {#if olm.loaded_model_path && olm.loaded_model_path !== configuredModelPath('olmocr', olm)}
                  <div class="model-path" title={olm.loaded_model_path}>loaded: {olm.loaded_model_path}</div>
                {/if}
                {#if olm.last_error}<div class="text-[11px] text-warn break-words mt-1">{olm.last_error}</div>{/if}
              </div>
              <div class="tier-actions">
                <button class="mini-btn"
                  disabled={!olm.compiled || tierDownload !== null}
                  onclick={() => downloadModel('olmocr')}>
                  {tierDownload === 'olmocr' ? 'Downloading…' : 'Download'}
                </button>
                <button class="mini-btn"
                  disabled={tierAction !== null}
                  onclick={() => chooseModelPath('olmocr')}>Browse</button>
                <button class="mini-btn"
                  disabled={!olm.compiled || !configuredModelPath('olmocr', olm) || olm.loaded || olm.loading || tierAction !== null}
                  onclick={() => runTierAction('olmocr', 'load', configuredModelPath('olmocr', olm))}>
                  {tierAction === 'olmocr' || olm.loading ? 'Loading' : 'Load'}
                </button>
                <button class="mini-btn"
                  disabled={!olm.loaded || olm.loading || tierAction !== null}
                  onclick={() => runTierAction('olmocr', 'unload')}>Unload</button>
                <input type="checkbox"
                  bind:checked={olm.enabled_in_settings}
                  disabled={!olm.compiled || !olm.model_present}
                  onchange={(e) => toggleTier('olmocr', e.currentTarget.checked)} />
              </div>
            </div>
          {/if}
        </section>
      {/if}

      {#if activeTab === 'ontology'}
        <section class="card p-5 space-y-3">
          <div class="flex items-baseline justify-between gap-3 flex-wrap">
            <div>
              <h2 class="text-sm font-semibold">Analyte ontology</h2>
              <p class="text-xs text-fg2">
                Re-installs analyte definitions (descriptions, categorical tiers, default reference ranges, aliases) from the bundled seed.
                Use after upgrading the app, then hit <em>Records → Re-parse all</em> to refresh existing rows.
              </p>
            </div>
            <button class="btn-accent" disabled={reloading} onclick={onReloadOntology}>
              {reloading ? 'Reloading…' : 'Reload from seed'}
            </button>
          </div>
        </section>

        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Browse / edit</h2>
            <p class="text-xs text-fg2">
              The full Ontology management tab — search, filter, view JSON, create / edit / delete user analytes — lives at <code class="font-mono">/ontology</code>.
            </p>
          </div>
          <div class="flex gap-2">
            <button class="btn" onclick={() => goto('/ontology')}>Open ontology browser →</button>
          </div>
        </section>
      {/if}

      {#if activeTab === 'storage'}
        {#if info}
          <!-- ─── Storage paths + sizes ─── -->
          <section class="card p-5 space-y-3">
            <div class="flex items-baseline justify-between gap-3 flex-wrap">
              <div>
                <h2 class="text-sm font-semibold">Storage paths &amp; sizes</h2>
                <p class="text-xs text-fg2">Where data lives on this device. Everything is encrypted at rest.</p>
              </div>
               <div class="flex flex-wrap gap-2">
                 <button class="btn text-xs" onclick={onOpenDataFolder}>
                   <Icon name="external" size={14} /> Open data folder
                 </button>
                 <button class="btn text-xs" onclick={refresh} disabled={loadingStorage}>
                   {loadingStorage ? 'Refreshing…' : 'Refresh'}
                 </button>
               </div>
            </div>
            <dl class="dl">
              <dt>Data dir</dt>
              <dd>
                <div class="font-mono break-all">{info.data_dir}</div>
                <div class="text-fg3 text-[11px] mt-0.5">total {appInfo.formatBytes(info.data_dir_size_bytes)}</div>
              </dd>
              <dt>Database</dt>
              <dd>
                <div class="font-mono break-all">{info.db_path}</div>
                <div class="text-fg3 text-[11px] mt-0.5">
                  {appInfo.formatBytes(info.db_size_bytes)}
                  {#if info.db_sidecar_bytes && info.db_sidecar_bytes > 0}
                    <span class="text-fg3"> · WAL/journal {appInfo.formatBytes(info.db_sidecar_bytes)}</span>
                  {/if}
                </div>
              </dd>
              <dt>Keystore</dt>
              <dd>
                <div class="font-mono break-all">{info.keystore_path}</div>
                <div class="text-fg3 text-[11px] mt-0.5">{appInfo.formatBytes(info.keystore_size_bytes)}</div>
              </dd>
              <dt>Encrypted PDF cache</dt>
              <dd>
                <div class="font-mono break-all">{info.pdf_dir}</div>
                <div class="text-fg3 text-[11px] mt-0.5">
                  {info.pdf_count} file{info.pdf_count === 1 ? '' : 's'}
                  · {appInfo.formatBytes(info.pdf_size_bytes)}
                </div>
              </dd>
              <dt>Models dir</dt>
              <dd>
                <div class="font-mono break-all">{info.models_dir}</div>
                <div class="text-fg3 text-[11px] mt-0.5">
                  {info.models_count} file{info.models_count === 1 ? '' : 's'}
                  · {appInfo.formatBytes(info.models_size_bytes)}
                </div>
              </dd>
            </dl>
          </section>

          <!-- ─── Database internals ─── -->
          {#if info.db_stats}
            {@const stats = info.db_stats}
            {@const pageBytes = (stats.page_size ?? 0) * (stats.page_count ?? 0)}
            <section class="card p-5 space-y-3">
              <div>
                <h2 class="text-sm font-semibold">Database contents</h2>
                <p class="text-xs text-fg2">A look inside the encrypted SQLite file.</p>
              </div>
              <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Patients</div>
                  <div class="kpi-tile__value tabular-nums">{stats.patient_count}</div>
                </div>
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Reports</div>
                  <div class="kpi-tile__value tabular-nums">{stats.report_count}</div>
                </div>
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Results</div>
                  <div class="kpi-tile__value tabular-nums">{stats.result_count}</div>
                  {#if stats.inline_prior_count > 0}
                    <div class="kpi-tile__sub">+{stats.inline_prior_count} inline-prior</div>
                  {/if}
                </div>
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Analytes</div>
                  <div class="kpi-tile__value tabular-nums">{stats.analyte_count}</div>
                  <div class="kpi-tile__sub">{stats.alias_count} aliases</div>
                </div>
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Audit entries</div>
                  <div class="kpi-tile__value tabular-nums">{stats.audit_count}</div>
                </div>
                <div class="kpi-tile">
                  <div class="kpi-tile__label">Date span</div>
                  <div class="kpi-tile__value text-sm">
                    {stats.earliest_collection_date_iso ?? '—'}
                  </div>
                  <div class="kpi-tile__sub">→ {stats.latest_collection_date_iso ?? '—'}</div>
                </div>
              </div>

              <dl class="dl pt-2 border-t border-line">
                <dt>Encryption engine</dt>
                <dd class="font-mono">{stats.sqlcipher_version ?? '—'}</dd>
                <dt>Journal mode</dt>
                <dd class="font-mono uppercase">{stats.journal_mode ?? '—'}</dd>
                <dt>Page size</dt>
                <dd class="font-mono">{stats.page_size ?? '—'} B</dd>
                <dt>Page count</dt>
                <dd class="font-mono">{stats.page_count ?? '—'}</dd>
                <dt>On-disk pages</dt>
                <dd class="font-mono">{appInfo.formatBytes(pageBytes)}</dd>
              </dl>
            </section>
          {:else}
            <section class="card p-5 space-y-2">
              <h2 class="text-sm font-semibold text-fg2">Database contents</h2>
              <p class="text-xs text-fg3">
                Vault locked. Unlock to see patient / report / result counts and SQLCipher diagnostics.
              </p>
            </section>
          {/if}

          <!-- ─── Export / import ─── -->
          <section class="card p-5 space-y-3">
            <div>
              <h2 class="text-sm font-semibold">Backup &amp; restore</h2>
              <p class="text-xs text-fg2">
                Export copies the entire encrypted vault — DB, keystore, PDFs, models — into a folder of
                your choice. The file remains encrypted, so it's safe to keep on a USB stick or sync to
                an external backup tool. Import replaces the current vault with a previous export
                (the current vault is renamed aside, not deleted, so you can roll back).
              </p>
            </div>

            <div class="flex flex-wrap gap-2">
              <button class="btn-accent" onclick={onExportVault} disabled={exporting}>
                {exporting ? 'Exporting…' : '⤓ Export vault…'}
              </button>
              <button class="btn" onclick={onImportVault} disabled={importing}>
                {importing ? 'Importing…' : '⤒ Import vault…'}
              </button>
            </div>

            {#if lastExport}
              <div class="card-tight bg-ok/10 border-ok/40 text-xs text-fg2 space-y-0.5">
                <div class="font-medium text-ok">Last export</div>
                <div class="font-mono break-all">{lastExport.destination}</div>
                <div class="text-fg3">
                  {lastExport.files_copied} files · {appInfo.formatBytes(lastExport.bytes_copied)}
                </div>
              </div>
            {/if}
            {#if lastImport}
              <div class="card-tight bg-warn/10 border-warn/40 text-xs text-fg2 space-y-0.5">
                <div class="font-medium text-warn">Last import — restart the app to use the imported vault.</div>
                <div>Restored from <span class="font-mono break-all">{lastImport.source}</span></div>
                <div class="text-fg3">
                  Previous vault preserved at <span class="font-mono break-all">{lastImport.backup_dir}</span>
                </div>
              </div>
            {/if}

            <div class="text-[11px] text-fg3 space-y-1">
              <p>
                <strong class="text-fg2">Export:</strong> safe at any time. Audit-logged. The destination directory must
                not be the vault itself; it gets created if missing.
              </p>
              <p>
                <strong class="text-fg2">Import:</strong> requires the vault to be locked first (the open SQLite handle
                would otherwise pin the old DB and corrupt the swap on Windows). The current vault is
                renamed to <span class="font-mono">data.backup-&lt;timestamp&gt;</span> for one-click rollback.
              </p>
            </div>
          </section>
        {/if}
      {/if}

      {#if activeTab === 'about'}
        <!-- ─── App identity ─── -->
        {#if info}
          <section class="card p-5 space-y-3">
            <div class="flex items-center gap-3">
              <svg viewBox="0 0 96 96" width="44" height="44" fill="none" aria-hidden="true">
                <defs>
                  <linearGradient id="aboutGrad" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0%" stop-color="#5b8bff"/>
                    <stop offset="100%" stop-color="#8e5bff"/>
                  </linearGradient>
                </defs>
                <circle cx="48" cy="48" r="44" stroke="url(#aboutGrad)" stroke-width="3" opacity="0.30"/>
                <path d="M14 56 L26 48 L36 60 L48 32 L58 50 L70 38 L82 44"
                      stroke="url(#aboutGrad)" stroke-width="5"
                      stroke-linecap="round" stroke-linejoin="round"/>
                <circle cx="48" cy="32" r="4" fill="url(#aboutGrad)"/>
              </svg>
              <div class="flex-1 min-w-0">
                <h2 class="text-base font-semibold">bloody-level</h2>
                <p class="text-xs text-fg2">Local-only clinical lab-PDF tracker with embedded OCR/LLM tiers.</p>
              </div>
              <span class="font-mono text-xs text-fg3 self-start">v{info.version}</span>
            </div>
          </section>

          <!-- ─── Build / environment ─── -->
          <section class="card p-5 space-y-3">
            <h2 class="text-sm font-semibold about-inline-label"><Icon name="info" size={15} /> Build &amp; environment</h2>
            <dl class="dl">
              <dt>Target</dt>
              <dd class="font-mono break-all">{info.target_triple}</dd>
              <dt><span class="about-inline-label"><Icon name="tag" size={13} /> Version</span></dt>
              <dd class="font-mono">v{info.version}</dd>
              <dt>Build profile</dt>
              <dd>{info.build_profile}{info.features.debug_assertions ? ' · debug-assertions' : ''}</dd>
              <dt>Frontend</dt>
              <dd>SvelteKit + Tauri WebView (Edge WebView2 on Windows / WKWebView on macOS / WebKitGTK on Linux)</dd>
              <dt>License</dt>
              <dd>Source-available — see repository LICENSE</dd>
            </dl>
            <div class="flex flex-wrap gap-1.5 pt-1">
              <span class={info.pdfium_available ? 'pill-ok' : 'pill-crit'}>pdfium</span>
              <span class={info.features.embedded_llm ? 'pill-ok' : 'pill-muted'}>embedded-llm</span>
              <span class={info.features.embedded_ocr_vision ? 'pill-ok' : 'pill-muted'}>embedded-ocr-vision</span>
              <span class={info.features.tesseract_ocr ? 'pill-ok' : 'pill-muted'}>tesseract-ocr</span>
              {#if info.features.debug_assertions}<span class="pill-warn">debug-assertions</span>{/if}
            </div>
            {#if info.pdfium_error}
              <p class="text-[11px] text-warn break-words">{info.pdfium_error}</p>
            {/if}
          </section>

          <!-- ─── Privacy & security stack ─── -->
          <section class="card p-5 space-y-3">
            <h2 class="text-sm font-semibold about-inline-label"><Icon name="shield" size={15} /> Privacy &amp; security</h2>
            <p class="text-xs text-fg2">The vault and managed PDF cache stay on this device. There is no telemetry, no cloud sync, no analytics.</p>
            <ul class="privacy-list text-xs text-fg2 space-y-1">
              <li><span class="privacy-list__icon"><Icon name="lock" size={14} /></span><span><strong>SQLCipher</strong> — full-database encryption with a per-vault key.</span></li>
              <li><span class="privacy-list__icon"><Icon name="file" size={14} /></span><span><strong>Encrypted PDF cache</strong> — source copies use XChaCha20-Poly1305 with a vault-derived key.</span></li>
              <li><span class="privacy-list__icon"><Icon name="shield" size={14} /></span><span><strong>Argon2id</strong> KDF derives the data master key from your password.</span></li>
              <li><span class="privacy-list__icon"><Icon name="shield" size={14} /></span><span><strong>XChaCha20-Poly1305</strong> wraps the DMK, with HKDF-SHA-256 sub-derivation.</span></li>
              <li><span class="privacy-list__icon"><Icon name="key" size={14} /></span><span><strong>WebAuthn / Passkey</strong> support for password-less unlock (PRF extension).</span></li>
              <li><span class="privacy-list__icon"><Icon name="ban" size={14} /></span><span><strong>Zero network</strong>: pdfium downloads are build-time only; no runtime egress.</span></li>
            </ul>
          </section>

          <!-- ─── Library credits ─── -->
          <section class="card p-5 space-y-4">
            <h2 class="text-sm font-semibold">Built with</h2>
            <p class="text-xs text-fg2">
              The work of these projects is what makes bloody-level possible. Each is bundled or
              linked under its own license.
            </p>

            <div class="credits-grid">
              {#each credits as group}
                <div class="credits-group">
                  <div class="credits-group__head">{group.head}</div>
                  <ul class="credits-list">
                    {#each group.items as item}
                      <li>
                        <button type="button"
                                class="credits-link"
                                onclick={() => visit(item.url)}
                                title="Open {item.url}">
                          <span class="credits-link__name">{item.name}</span>
                          <span class="credits-link__icon"><Icon name="external" size={12} /></span>
                          <span class="credits-link__note">{item.note}</span>
                        </button>
                      </li>
                    {/each}
                  </ul>
                </div>
              {/each}
            </div>
            <p class="text-[11px] text-fg3 italic">
              Each link opens in your default browser via the OS shell — no embedded webview, no
              redirects through us.
            </p>
          </section>

          <!-- ─── Author / attribution ─── -->
          <section class="card p-5 space-y-2">
            <h2 class="text-sm font-semibold">Author &amp; attribution</h2>
            <dl class="dl">
              <dt>Author</dt>
              <dd>supermarsx and maintainers</dd>
              <dt>Copyright</dt>
              <dd>© 2026 supermarsx</dd>
              <dt>Disclaimer</dt>
              <dd class="text-fg2">
                This software is for personal record-keeping and trend visualisation only.
                It is not a medical device. It does not make clinical decisions.
                Always interpret values with your physician.
              </dd>
            </dl>
          </section>
        {/if}
      {/if}

      {#if activeTab === 'advanced'}
        <section class="card p-5 space-y-4">
          <div>
            <h2 class="text-sm font-semibold">App lifecycle</h2>
            <p class="text-xs text-fg2">
              Refresh the interface, relaunch the native app, or start over with a new empty local instance.
              Your vault is unchanged by the first two actions.
            </p>
          </div>

          <div class="lifecycle-actions">
            <div class="lifecycle-action">
              <div class="lifecycle-action__copy">
                <div class="row__title"><Icon name="settings" size={15} /> Reset all preferences</div>
                <div class="row__hint">Restore appearance, dashboard, charts, compare presets, and tier preferences without deleting vault data.</div>
              </div>
              <button class="btn" type="button" disabled={lifecycleAction !== null} onclick={onResetDefaults}>
                {lifecycleAction === 'defaults' ? 'Resetting…' : 'Reset defaults'}
              </button>
            </div>

            <div class="lifecycle-action">
              <div class="lifecycle-action__copy">
                <div class="row__title"><Icon name="monitor" size={15} /> Restart frontend</div>
                <div class="row__hint">Reload the current webview and preserve the running native process.</div>
              </div>
              <button class="btn" type="button" disabled={lifecycleAction !== null} onclick={onRestartFrontend}>
                {lifecycleAction === 'frontend' ? 'Reloading…' : 'Restart frontend'}
              </button>
            </div>

            <div class="lifecycle-action">
              <div class="lifecycle-action__copy">
                <div class="row__title"><Icon name="settings" size={15} /> Restart whole app</div>
                <div class="row__hint">Close and relaunch bloody-level, including its native services.</div>
              </div>
              <button class="btn" type="button" disabled={lifecycleAction !== null} onclick={onRestartApp}>
                {lifecycleAction === 'app' ? 'Restarting…' : 'Restart app'}
              </button>
            </div>

            <div class="lifecycle-action lifecycle-action--danger">
              <div class="lifecycle-action__copy">
                <div class="row__title"><Icon name="trash" size={15} /> Reset local app</div>
                <div class="row__hint">Permanently delete this local instance and return to the first-run welcome screen.</div>
              </div>
              <button class="btn btn-danger" type="button" disabled={lifecycleAction !== null} onclick={onResetApp}>
                {lifecycleAction === 'reset' ? 'Resetting…' : 'Reset app'}
              </button>
            </div>
          </div>
        </section>

        <section class="card p-5 space-y-3">
          <div>
            <h2 class="text-sm font-semibold">Raw settings</h2>
            <p class="text-xs text-fg2">
              Every key in the encrypted settings table. Read-only — power-user inspection only;
              normal toggles live in the other tabs.
            </p>
          </div>
          <pre class="text-xs text-fg2 whitespace-pre-wrap font-mono bg-bg1 p-3 rounded-md border border-line max-h-[60vh] overflow-y-auto">{JSON.stringify(raw, null, 2)}</pre>
        </section>
      {/if}
    </div>
  </div>
</div>

<style>
  .settings { display: flex; flex-direction: column; gap: 1rem; }
  .settings__head { display: flex; flex-direction: column; gap: 0.15rem; }
  .section-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    width: 2rem;
    height: 2rem;
    border: 1px solid rgb(var(--accent) / 0.35);
    border-radius: 0.55rem;
    color: rgb(var(--accent));
    background: rgb(var(--accent) / 0.1);
  }
  .security-status-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.65rem;
  }
  .security-status-card {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
    padding: 0.7rem;
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    background: rgb(var(--bg-1));
  }
  .security-status-card strong { font-size: 0.82rem; }
  .security-actions {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.7rem;
  }
  .security-toggle {
    display: inline-flex;
    align-items: flex-start;
    gap: 0.55rem;
    cursor: pointer;
  }
  .security-toggle span { display: flex; flex-direction: column; gap: 0.1rem; font-size: 0.75rem; }
  .security-toggle small { color: rgb(var(--fg-3)); font-size: 0.68rem; }
  .security-notes {
    display: grid;
    gap: 0.45rem;
    padding-top: 0.25rem;
    color: rgb(var(--fg-3));
    font-size: 0.7rem;
    line-height: 1.45;
  }
  .security-notes strong { color: rgb(var(--fg-2)); }
  @media (max-width: 520px) {
    .security-status-grid { grid-template-columns: 1fr; }
  }

  .settings__shell {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1rem;
  }
  @media (min-width: 800px) {
    .settings__shell {
      grid-template-columns: 14rem 1fr;
      gap: 1.25rem;
    }
  }

  .settings__nav {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    background: rgb(var(--bg-2));
    border: 1px solid rgb(var(--line));
    border-radius: 0.6rem;
    padding: 0.4rem;
    align-self: start;
    position: sticky;
    top: 1rem;
  }
  .settings__tab {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.5rem 0.7rem;
    background: transparent;
    border: 0;
    border-radius: 0.4rem;
    cursor: pointer;
    text-align: left;
    color: rgb(var(--fg-2));
    transition: background 100ms ease, color 100ms ease;
  }
  .settings__tab:hover { background: rgb(var(--bg-3)); color: rgb(var(--fg-1)); }
  .settings__tab--active {
    background: rgb(var(--accent) / 0.14);
    color: rgb(var(--accent));
  }
  .settings__tab-icon { display: inline-flex; color: currentColor; line-height: 1; }
  .settings__tab-body { display: flex; flex-direction: column; gap: 0.05rem; min-width: 0; }
  .settings__tab-label { font-size: 0.85rem; font-weight: 600; }
  .settings__tab-hint  { font-size: 0.65rem; color: rgb(var(--fg-3)); }
  .settings__tab--active .settings__tab-hint { color: rgb(var(--accent) / 0.8); }

  .settings__pane {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .lifecycle-actions { display: flex; flex-direction: column; }
  .privacy-list li { display: flex; align-items: flex-start; gap: 0.45rem; }
  .privacy-list__icon { display: inline-flex; flex: 0 0 auto; margin-top: 0.1rem; color: rgb(var(--accent)); }
  .about-inline-label { display: inline-flex; align-items: center; gap: 0.4rem; }
  .lifecycle-action {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.8rem 0;
    border-top: 1px solid rgb(var(--line));
  }
  .lifecycle-action:first-child { border-top: 0; padding-top: 0; }
  .lifecycle-action__copy { display: flex; flex-direction: column; gap: 0.2rem; min-width: 0; }
  .lifecycle-action .row__title { align-items: center; gap: 0.4rem; }
  .lifecycle-action--danger .row__title { color: rgb(var(--crit)); }
  .btn-danger {
    border-color: rgb(var(--crit) / 0.5);
    color: rgb(var(--crit));
  }
  .btn-danger:hover:not(:disabled) { background: rgb(var(--crit) / 0.1); border-color: rgb(var(--crit)); }
  @media (max-width: 520px) {
    .lifecycle-action { align-items: flex-start; flex-direction: column; }
  }

  /* ── Row primitive (label + control on one line) ── */
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.85rem;
    padding: 0.75rem 0;
    border-top: 1px solid rgb(var(--line));
  }
  .row:first-of-type { border-top: 0; padding-top: 0; }
  .row__body { display: flex; flex-direction: column; gap: 0.15rem; min-width: 0; }
  .row__title {
    font-size: 0.875rem;
    color: rgb(var(--fg-1));
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }
  .row__sub {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .row__hint { font-size: 0.7rem; color: rgb(var(--fg-3)); line-height: 1.35; }
  .row__hint--error { color: rgb(var(--crit)); }
  .dashboard-sections { display: flex; flex-direction: column; gap: 0.25rem; }
  .dashboard-section-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.65rem 0;
    border-top: 1px solid rgb(var(--line));
  }
  .dashboard-section-row:first-child { border-top: 0; padding-top: 0; }
  .dashboard-section-row__toggle { flex: 1; min-width: 0; border-top: 0; padding: 0; }
  .dashboard-section-row__actions { display: inline-flex; align-items: center; gap: 0.3rem; flex: 0 0 auto; }
  .dashboard-section-row__actions .mini-btn { display: inline-flex; align-items: center; justify-content: center; padding: 0.3rem; }
  .settings-number {
    width: 7rem;
    flex: 0 0 auto;
    padding: 0.4rem 0.55rem;
    border: 1px solid rgb(var(--line));
    border-radius: 0.4rem;
    background: rgb(var(--bg-1));
    color: rgb(var(--fg-1));
    font-size: 0.8rem;
  }
  .settings-number:focus { outline: none; border-color: rgb(var(--accent)); box-shadow: 0 0 0 3px rgb(var(--accent) / 0.18); }
  .tier-actions {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    flex: 0 0 auto;
  }
  .tier-resource-help {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin-top: 0.45rem;
  }
  .mini-btn {
    border: 1px solid rgb(var(--line));
    background: rgb(var(--bg-1));
    color: rgb(var(--fg-2));
    border-radius: 0.35rem;
    padding: 0.28rem 0.55rem;
    font-size: 0.72rem;
    line-height: 1;
    cursor: pointer;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .mini-btn:hover:not(:disabled) {
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--accent) / 0.45);
  }
  .mini-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .model-path {
    max-width: min(42rem, 100%);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.67rem;
    color: rgb(var(--fg-3));
  }

  /* ── Segmented control ── */
  .seg {
    display: inline-flex;
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
    width: max-content;
  }
  .seg__opt {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0.4rem 0.85rem;
    font-size: 0.8rem;
    color: rgb(var(--fg-2));
    background: transparent;
    border: 0;
    border-right: 1px solid rgb(var(--line));
    cursor: pointer;
    white-space: nowrap;
    transition: background 120ms ease, color 120ms ease;
  }
  .seg__opt:last-child { border-right: 0; }
  .seg__opt:hover { background: rgb(var(--bg-2)); color: rgb(var(--fg-1)); }
  .seg__opt--on {
    background: rgb(var(--accent) / 0.18);
    color: rgb(var(--accent));
    font-weight: 600;
  }
  /* Wrapping variant for date-format / multi-option segmented controls so a
     long row breaks onto two lines on narrow viewports instead of clipping. */
  .seg--wrap {
    display: inline-flex;
    flex-wrap: wrap;
    width: auto;
    max-width: 100%;
  }
  .seg--wrap .seg__opt { border-right: 1px solid rgb(var(--line)); }
  .seg--wrap .seg__opt:last-child { border-right: 0; }
  .seg--nowrap {
    flex-wrap: nowrap;
    width: max-content;
    max-width: 100%;
    overflow-x: auto;
  }
  .seg--nowrap .seg__opt { flex: 0 0 auto; }

  /* ── Accent swatches ── */
  .swatches {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: flex-start;
    gap: 0.4rem;
  }
  .swatch {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    width: max-content;
    max-width: 100%;
    gap: 0.45rem;
    padding: 0.45rem 0.65rem;
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.45rem;
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease, transform 120ms ease;
  }
  .swatch:hover { border-color: rgb(var(--fg-3)); }
  .swatch--on   {
    border-color: var(--swatch-color);
    background: color-mix(in srgb, var(--swatch-color) 8%, transparent);
  }
  .swatch__dot {
    width: 18px;
    height: 18px;
    border-radius: 9999px;
    background: var(--swatch-color);
    box-shadow: 0 0 0 2px rgb(var(--bg-1)), 0 0 0 3px rgb(var(--line));
  }
  .swatch--on .swatch__dot {
    box-shadow: 0 0 0 2px rgb(var(--bg-1)), 0 0 0 3px var(--swatch-color);
  }
  .swatch__name {
    font-size: 0.75rem;
    color: rgb(var(--fg-1));
    font-weight: 500;
  }

  /* ── Credits grid (About tab) ── */
  .credits-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0.85rem 1rem;
  }
  @media (min-width: 640px) { .credits-grid { grid-template-columns: 1fr 1fr; } }
  @media (min-width: 1100px) { .credits-grid { grid-template-columns: 1fr 1fr 1fr; } }
  .credits-group { display: flex; flex-direction: column; gap: 0.3rem; }
  .credits-group__head {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: rgb(var(--accent));
    font-weight: 600;
  }
  .credits-list {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .credits-list :global(li) {
    padding: 0;
    border-bottom: 1px dashed rgb(var(--line) / 0.5);
  }
  .credits-list :global(li:last-child) { border-bottom: 0; }
  /* Clickable row — full-width grid: name (·) icon · note. Hover surfaces
     an accent backdrop and the ↗ glyph reveals the linkability without
     forcing every row to look like a hyperlink at rest. */
  .credits-link {
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: baseline;
    gap: 0.45rem;
    width: 100%;
    background: transparent;
    border: 0;
    padding: 0.3rem 0.4rem;
    border-radius: 0.3rem;
    text-align: left;
    cursor: pointer;
    color: inherit;
    transition: background 100ms ease, color 100ms ease;
  }
  .credits-link:hover  { background: rgb(var(--accent) / 0.08); }
  .credits-link:focus-visible {
    outline: 2px solid rgb(var(--accent));
    outline-offset: 1px;
  }
  .credits-link__name { color: rgb(var(--fg-1)); font-weight: 500; font-size: 0.75rem; }
  .credits-link__icon {
    color: rgb(var(--fg-3));
    font-size: 0.7rem;
    transition: color 100ms ease, transform 100ms ease;
  }
  .credits-link:hover .credits-link__icon {
    color: rgb(var(--accent));
    transform: translate(1px, -1px);
  }
  .credits-link__note { color: rgb(var(--fg-3)); font-size: 0.7rem; text-align: right; }

  /* ── Description list (storage / about) ── */
  .dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.4rem 1rem;
    font-size: 0.8rem;
  }
  .dl :global(dt) { color: rgb(var(--fg-3)); }
  .dl :global(dd) { color: rgb(var(--fg-1)); }

  /* ── KPI tile used in the Database contents grid ── */
  .kpi-tile {
    background: rgb(var(--bg-1));
    border: 1px solid rgb(var(--line));
    border-radius: 0.5rem;
    padding: 0.55rem 0.7rem;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    transition: border-color 120ms ease;
  }
  .kpi-tile:hover { border-color: rgb(var(--accent) / 0.4); }
  .kpi-tile__label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: rgb(var(--fg-3));
  }
  .kpi-tile__value {
    font-size: 1.05rem;
    font-weight: 600;
    color: rgb(var(--fg-1));
    line-height: 1.2;
  }
  .kpi-tile__sub {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
  }

  /* Mini reorder buttons used inside the compare-preset editor — sized to
     fit a 3-button row in the picked-analyte list without crowding it. */
  .reorder-btn-mini {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.25rem;
    border: 1px solid rgb(var(--line));
    background: rgb(var(--bg-2));
    color: rgb(var(--fg-2));
    border-radius: 0.25rem;
    cursor: pointer;
    font-size: 0.7rem;
    line-height: 1;
    transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
  }
  .reorder-btn-mini:hover:not(:disabled) {
    background: rgb(var(--bg-3));
    color: rgb(var(--fg-1));
    border-color: rgb(var(--accent) / 0.5);
  }
  .reorder-btn-mini:disabled { opacity: 0.35; cursor: not-allowed; }
</style>
