<script lang="ts">
  import { onMount } from 'svelte';
  import * as auth from '$api/auth';
  import { AppError } from '$api/errors';
  import { ask } from '@tauri-apps/plugin-dialog';
  import Icon from './icon.svelte';

  let { onUnlocked } = $props<{ onUnlocked: () => void }>();

  let status = $state<auth.AuthStatus | null>(null);
  let mode = $state<'setup' | 'unlock'>('unlock');
  let password = $state('');
  let confirmPassword = $state('');
  let showPw = $state(false);
  let busy = $state(false);
  let nativeUnlockAttempted = $state(false);
  let resetting = $state(false);
  let err = $state<AppError | null>(null);

  async function refresh() {
    try {
      status = await auth.status();
      mode = status.initialized ? 'unlock' : 'setup';
      if (status.unlocked) onUnlocked();
      else if (status.os_vault_configured && status.os_vault_auto_unlock && !nativeUnlockAttempted) {
        nativeUnlockAttempted = true;
        try {
          await auth.unlockOsVault();
          onUnlocked();
          return;
        } catch {
          // Fall back to the explicit unlock controls when the OS store is
          // unavailable, locked, or the credential was revoked.
        }
      }
    } catch (e) {
      err = AppError.fromUnknown(e);
    }
  }

  onMount(refresh);

  // Lightweight password strength heuristic — purely UX feedback. Argon2id
  // does the actual hardening; this just nudges the user toward longer
  // passphrases since the only recovery is "remember it".
  const strength = $derived.by(() => {
    const p = password;
    if (p.length === 0) return { score: 0, label: '', tone: 'fg3' };
    let score = 0;
    if (p.length >= 10) score += 1;
    if (p.length >= 16) score += 1;
    if (p.length >= 24) score += 1;
    if (/[a-z]/.test(p) && /[A-Z]/.test(p)) score += 1;
    if (/\d/.test(p)) score += 1;
    if (/[^A-Za-z0-9]/.test(p)) score += 1;
    if (/(.)\1\1/.test(p)) score = Math.max(0, score - 1); // penalize aaa
    const labels = ['Too short', 'Weak', 'Fair', 'Good', 'Strong', 'Excellent'];
    const tones = ['crit', 'crit', 'warn', 'warn', 'ok', 'ok'];
    const idx = Math.min(score, labels.length - 1);
    return { score: idx, label: labels[idx], tone: tones[idx] };
  });

  const passwordsMatch = $derived(
    confirmPassword.length === 0 || password === confirmPassword
  );
  const backoffRemaining = $derived(status?.unlock_backoff_remaining_secs ?? 0);

  async function setupPassword() {
    err = null;
    if (password.length < 10) {
      err = AppError.fromUnknown({ kind: 'bad_request', code: 'pw.too_short', message: 'Password must be at least 10 characters.', detail: null, retryable: false, timestamp: 0 });
      return;
    }
    if (password !== confirmPassword) {
      err = AppError.fromUnknown({ kind: 'bad_request', code: 'pw.mismatch', message: 'Passwords do not match.', detail: null, retryable: false, timestamp: 0 });
      return;
    }
    busy = true;
    try {
      await auth.setupPassword(password);
      password = ''; confirmPassword = '';
      await refresh();
    } catch (e) { err = AppError.fromUnknown(e); }
    finally { busy = false; }
  }

  async function setupOsVault() {
    if (busy) return;
    err = null;
    busy = true;
    try {
      await auth.setupOsVault();
      await refresh();
    } catch (e) { err = AppError.fromUnknown(e); }
    finally { busy = false; }
  }

  async function unlockPassword() {
    err = null;
    busy = true;
    try {
      await auth.unlockPassword(password);
      password = '';
      await refresh();
    } catch (e) {
      err = AppError.fromUnknown(e);
      await refresh();
    }
    finally { busy = false; }
  }

  async function unlockOsVault() {
    if (busy) return;
    busy = true;
    err = null;
    try {
      await auth.unlockOsVault();
      onUnlocked();
    } catch (e) {
      err = AppError.fromUnknown(e);
      await refresh();
    } finally {
      busy = false;
    }
  }

  async function unlockPasskey(passkey: auth.PasskeySummary | undefined = status?.passkeys?.[0]) {
    err = null;
    busy = true;
    try {
      if (!passkey) {
        throw new Error('No passkey is registered for this vault');
      }
      if (!auth.isWebAuthnAvailable()) {
        throw new Error('WebAuthn not available in this WebView');
      }
      const assertion = await auth.webauthnAssert(passkey.credential_id_b64, passkey.prf_salt_b64);
      await auth.unlockPasskey({
        credential_id_b64: auth.b64.encode(assertion.credentialId),
        prf_output_b64: auth.b64.encode(assertion.prfOutput)
      });
      await refresh();
    } catch (e) {
      err = AppError.fromUnknown(e);
      await refresh();
    }
    finally { busy = false; }
  }

  async function devSkip() {
    err = null;
    busy = true;
    try {
      await auth.devSkip();
      await refresh();
    } catch (e) { err = AppError.fromUnknown(e); }
    finally { busy = false; }
  }

  async function resetInstance() {
    if (busy || resetting) return;
    const confirmed = await ask(
      'Reset this bloody-level instance?\n\n' +
      'This permanently removes the encrypted database, password, passkeys, imported reports, PDFs, and local model files from this device. Forgotten passwords cannot be recovered.\n\n' +
      'This cannot be undone. Continue?',
      { title: 'Reset instance', kind: 'warning' }
    );
    if (!confirmed) return;

    resetting = true;
    err = null;
    try {
      await auth.resetInstance();
      password = '';
      confirmPassword = '';
      await refresh();
    } catch (e) {
      err = AppError.fromUnknown(e);
    } finally {
      resetting = false;
    }
  }

  function blockContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function keepContextMenu(event: MouseEvent) {
    event.stopPropagation();
  }
</script>

<svelte:window oncontextmenu={blockContextMenu} />

<div class="gate">
  <div class="gate__bg" aria-hidden="true">
    <!-- Soft, slow gradient blobs — subtle motion behind the auth card. -->
    <div class="gate__blob gate__blob--a"></div>
    <div class="gate__blob gate__blob--b"></div>
    <div class="gate__blob gate__blob--c"></div>
  </div>

  <div class="gate__shell">
    <!-- Brand mark + tagline -->
    <header class="gate__brand">
      <svg viewBox="0 0 96 96" width="56" height="56" fill="none" aria-hidden="true">
        <defs>
          <linearGradient id="gateGrad" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="#5b8bff" />
            <stop offset="100%" stop-color="#8e5bff" />
          </linearGradient>
        </defs>
        <circle cx="48" cy="48" r="44" stroke="url(#gateGrad)" stroke-width="2" opacity="0.25" />
        <path d="M14 56 L26 48 L36 60 L48 32 L58 50 L70 38 L82 44"
              stroke="url(#gateGrad)" stroke-width="3"
              stroke-linecap="round" stroke-linejoin="round" />
        <circle cx="48" cy="32" r="3" fill="url(#gateGrad)" />
      </svg>
      <h1 class="gate__title">bloody-level</h1>
      <p class="gate__tagline">Lab results, longitudinal — fully local, fully encrypted.</p>
    </header>

    <div class="card gate__card">
      {#if mode === 'setup'}
        <div class="space-y-1">
          <h2 class="text-base font-semibold">Welcome — let's set up your vault</h2>
          <p class="text-xs text-fg2">
            bloody-level stores everything on <em>this device only</em>. Nothing leaves your computer.
            Choose a password to encrypt the database — you'll need it every time you open the app.
          </p>
        </div>

        <div class="rounded-md border border-line/60 bg-bg2/50 p-3 text-xs text-fg2 space-y-1">
          <p class="font-medium text-fg1 flex items-center gap-1">
            <Icon name="lock" size={14} /> Important — there is no recovery
          </p>
          <p>If you forget this password, your data is unrecoverable. Pick something memorable but long; consider a passphrase like <span class="font-mono">"olive-piano-7-cliffside-music"</span>.</p>
        </div>

        <label class="block">
          <span class="text-xs text-fg2">Password</span>
          <div class="mt-1 relative">
            <input
              type={showPw ? 'text' : 'password'}
              class="block w-full bg-bg1 border border-line rounded-md pl-3 pr-9 py-2 text-sm focus:outline-none focus:border-accent"
              bind:value={password}
              autocomplete="new-password"
              oncontextmenu={keepContextMenu}
              placeholder="At least 10 characters; longer is much better"
            />
            <button type="button" class="gate__pw-toggle"
              onclick={() => (showPw = !showPw)}
              title={showPw ? 'Hide password' : 'Show password'}
              aria-label={showPw ? 'Hide password' : 'Show password'}
            ><Icon name={showPw ? 'eye-off' : 'eye'} size={16} /></button>
          </div>
          {#if password.length > 0}
            <div class="mt-1.5 flex items-center gap-2">
              <div class="gate__strength" aria-hidden="true">
                {#each [0, 1, 2, 3, 4, 5] as i}
                  <span class="gate__strength-pip"
                    class:gate__strength-pip--filled={i <= strength.score}
                    class:gate__strength-pip--crit={strength.tone === 'crit'}
                    class:gate__strength-pip--warn={strength.tone === 'warn'}
                    class:gate__strength-pip--ok={strength.tone === 'ok'}
                  ></span>
                {/each}
              </div>
              <span class="text-[11px] text-{strength.tone}">{strength.label}</span>
            </div>
          {/if}
        </label>

        <label class="block">
          <span class="text-xs text-fg2">Confirm password</span>
          <input
            type={showPw ? 'text' : 'password'}
            class="mt-1 block w-full bg-bg1 border border-line rounded-md px-3 py-2 text-sm focus:outline-none focus:border-accent {confirmPassword && !passwordsMatch ? 'border-crit' : ''}"
            bind:value={confirmPassword}
            autocomplete="new-password"
            oncontextmenu={keepContextMenu}
            onkeydown={(e) => e.key === 'Enter' && setupPassword()}
            placeholder="Type it again"
          />
          {#if confirmPassword.length > 0 && !passwordsMatch}
            <p class="mt-1 text-[11px] text-crit">Passwords don't match yet.</p>
          {/if}
        </label>

        <button
          class="btn-accent w-full"
          disabled={busy || password.length < 10 || !passwordsMatch || confirmPassword.length === 0}
          onclick={setupPassword}
        >{busy ? 'Encrypting…' : 'Create vault'}</button>

        {#if status?.os_vault_supported}
          <div class="gate__divider"><span>or</span></div>
          <button class="btn w-full flex items-center justify-center gap-2" disabled={busy} onclick={setupOsVault}>
            <Icon name="shield" size={15} /> {busy ? 'Preparing native vault…' : 'Use OS vault without a password'}
          </button>
          <p class="text-[11px] text-fg3">
            The encrypted master key will be protected by {status.os_vault_platform}. Add a passkey
            from Settings for another recovery method; anyone who can unlock this OS account may
            access the vault.
          </p>
        {/if}

        {#if status?.is_dev}
          <details class="text-xs text-fg3">
            <summary class="cursor-pointer">Developer shortcut</summary>
            <div class="mt-2 space-y-2">
              <button class="btn w-full" disabled={busy} onclick={devSkip}>
                Skip — initialize with the public dev password
              </button>
              <p class="text-[11px] text-warn">
                Dev builds only. Uses a hard-coded password — never load real medical data this way.
              </p>
            </div>
          </details>
        {/if}
      {:else}
        <div class="space-y-1">
          <h2 class="text-base font-semibold">Welcome back</h2>
          <p class="text-xs text-fg2">
            {status?.has_password
              ? 'Enter your password to unlock.'
              : 'This vault has no password. Use the native OS vault or a registered passkey to unlock.'}
          </p>
        </div>

        {#if status?.os_vault_configured && status.has_password}
          <button class="btn w-full" disabled={busy} onclick={unlockOsVault}>
            <Icon name="shield" size={15} /> Unlock with OS vault
          </button>
          <p class="text-[11px] text-fg3">Uses your configured native credential store; your password remains available as a fallback.</p>
        {/if}

        {#if status?.has_password}
          <label class="block">
            <span class="text-xs text-fg2">Password</span>
            <div class="mt-1 relative">
              <input
                type={showPw ? 'text' : 'password'}
                class="block w-full bg-bg1 border border-line rounded-md pl-3 pr-9 py-2 text-sm focus:outline-none focus:border-accent"
                bind:value={password}
                autocomplete="current-password"
                oncontextmenu={keepContextMenu}
                onkeydown={(e) => e.key === 'Enter' && password && !busy && unlockPassword()}
                placeholder="Your vault password"
              />
              <button type="button" class="gate__pw-toggle"
                onclick={() => (showPw = !showPw)}
                title={showPw ? 'Hide password' : 'Show password'}
                aria-label={showPw ? 'Hide password' : 'Show password'}
              ><Icon name={showPw ? 'eye-off' : 'eye'} size={16} /></button>
            </div>
          </label>

          <button
            class="btn-accent w-full"
            disabled={busy || !password}
            onclick={unlockPassword}
          >{busy ? 'Unlocking…' : 'Unlock'}</button>
        {:else if status?.os_vault_configured}
          <button class="btn-accent w-full" disabled={busy} onclick={unlockPassword}>
            <Icon name="shield" size={15} /> {busy ? 'Unlocking…' : 'Unlock without password'}
          </button>
        {/if}

        <div class="gate__reset-area">
          <button
            type="button"
            class="gate__reset"
            disabled={busy || resetting}
            onclick={resetInstance}
          >
            <Icon name="trash" size={14} />
            {resetting ? 'Resetting instance…' : 'Reset this instance'}
          </button>
          <p>Use only if the vault password cannot be recovered.</p>
        </div>

        {#if status?.has_passkey}
          <div class="gate__divider"><span>or</span></div>
          {#if (status.passkeys?.length ?? 0) <= 1}
            <button class="btn w-full flex items-center justify-center gap-2" disabled={busy} onclick={() => unlockPasskey()}>
              <Icon name="key" size={15} /> Use a passkey
            </button>
          {:else}
            <div class="space-y-2">
              {#each status.passkeys as passkey}
                <button class="btn w-full flex items-center justify-center gap-2" disabled={busy} onclick={() => unlockPasskey(passkey)}>
                  <Icon name="key" size={15} /> {passkey.label || 'Passkey'}
                </button>
              {/each}
            </div>
          {/if}
        {/if}

        {#if status?.is_dev}
          <details class="text-xs text-fg3">
            <summary class="cursor-pointer">Developer shortcut</summary>
            <button class="btn w-full mt-2" disabled={busy} onclick={devSkip}>
              Skip (dev)
            </button>
          </details>
        {/if}

        {#if status && status.failed_unlocks > 0}
          <p class="text-[11px] text-warn flex items-center gap-1">
            <Icon name="warning" size={14} />
            {status.failed_unlocks} failed attempt{status.failed_unlocks === 1 ? '' : 's'} since the last successful unlock.
            {#if backoffRemaining > 0}
              Try again in {backoffRemaining} second{backoffRemaining === 1 ? '' : 's'}.
            {/if}
          </p>
        {/if}
      {/if}

      {#if err}
        <div class="rounded-md border border-crit/40 bg-crit/10 p-2.5 text-xs space-y-0.5">
          <div class="text-crit font-medium">{err.message}</div>
          <div class="text-fg3 font-mono text-[10px]">{err.code}</div>
        </div>
      {/if}
    </div>

    <footer class="gate__footer">
      <span class="gate__footer-pill"><Icon name="lock" size={13} /> Private</span>
      <span class="gate__footer-pill"><Icon name="shield" size={13} /> Encrypted</span>
      <span class="gate__footer-pill"><Icon name="database" size={13} /> Local-only</span>
      <span class="gate__footer-pill"><Icon name="ban" size={13} /> No telemetry</span>
    </footer>
  </div>
</div>

<style>
  .gate {
    position: relative;
    min-height: 100vh;
    display: grid;
    place-items: center;
    padding: 1.5rem;
    overflow: hidden;
    background: rgb(var(--bg-1));
    user-select: none;
    -webkit-user-select: none;
  }
  .gate input {
    user-select: text;
    -webkit-user-select: text;
  }
  .gate__bg {
    position: absolute;
    inset: 0;
    pointer-events: none;
    overflow: hidden;
  }
  .gate__reset-area {
    display: grid;
    justify-items: center;
    gap: 0.25rem;
    margin-top: -0.25rem;
  }
  .gate__reset {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    border: 0;
    padding: 0.25rem 0.5rem;
    color: rgb(var(--fg-3));
    background: transparent;
    border-radius: 0.375rem;
    font-size: 0.6875rem;
    cursor: pointer;
    transition: color 140ms ease, background-color 140ms ease;
  }
  .gate__reset:hover:not(:disabled),
  .gate__reset:focus-visible {
    color: rgb(var(--crit));
    background: rgb(var(--crit) / 0.1);
  }
  .gate__reset:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
  .gate__reset-area p {
    margin: 0;
    color: rgb(var(--fg-3));
    font-size: 0.625rem;
  }
  .gate__blob {
    position: absolute;
    border-radius: 9999px;
    filter: blur(60px);
    opacity: 0.35;
    animation: gate-float 16s ease-in-out infinite;
  }
  .gate__blob--a {
    width: 28rem;
    height: 28rem;
    background: radial-gradient(circle, #5b8bff 0%, transparent 70%);
    top: -8rem;
    left: -6rem;
  }
  .gate__blob--b {
    width: 24rem;
    height: 24rem;
    background: radial-gradient(circle, #8e5bff 0%, transparent 70%);
    bottom: -6rem;
    right: -4rem;
    animation-delay: -5s;
  }
  .gate__blob--c {
    width: 18rem;
    height: 18rem;
    background: radial-gradient(circle, #5bffaa 0%, transparent 70%);
    top: 40%;
    right: 30%;
    opacity: 0.18;
    animation-delay: -10s;
  }
  @keyframes gate-float {
    0%, 100% { transform: translate(0, 0) scale(1); }
    50%      { transform: translate(2rem, -1.5rem) scale(1.06); }
  }

  .gate__shell {
    position: relative;
    width: 100%;
    max-width: 26rem;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 1.25rem;
  }
  .gate__brand {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }
  .gate__title {
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    margin-top: 0.5rem;
  }
  .gate__tagline {
    font-size: 0.8rem;
    color: rgb(var(--fg-2));
    max-width: 22rem;
  }
  .gate__card {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    backdrop-filter: blur(8px);
    background: rgb(var(--bg-2) / 0.85);
  }
  .gate__pw-toggle {
    position: absolute;
    right: 0.5rem;
    top: 50%;
    transform: translateY(-50%);
    background: transparent;
    border: 0;
    cursor: pointer;
    color: rgb(var(--fg-3));
    font-size: 0.95rem;
    line-height: 1;
    padding: 0.25rem;
  }
  .gate__pw-toggle:hover { color: rgb(var(--fg-1)); }
  .gate__strength {
    display: flex;
    gap: 3px;
    flex: 1;
  }
  .gate__strength-pip {
    display: block;
    height: 4px;
    flex: 1;
    border-radius: 2px;
    background: rgb(var(--bg-3));
    transition: background 150ms ease;
  }
  .gate__strength-pip--filled.gate__strength-pip--crit { background: rgb(var(--crit)); }
  .gate__strength-pip--filled.gate__strength-pip--warn { background: rgb(var(--warn)); }
  .gate__strength-pip--filled.gate__strength-pip--ok   { background: rgb(var(--ok)); }
  .gate__divider {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: rgb(var(--fg-3));
    font-size: 0.7rem;
  }
  .gate__divider::before,
  .gate__divider::after {
    content: '';
    flex: 1;
    height: 1px;
    background: rgb(var(--line));
  }
  .gate__footer {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.4rem;
  }
  .gate__footer-pill {
    font-size: 0.65rem;
    color: rgb(var(--fg-3));
    background: rgb(var(--bg-2) / 0.7);
    border: 1px solid rgb(var(--line));
    padding: 0.2rem 0.55rem;
    border-radius: 9999px;
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
  }

  @media (prefers-reduced-motion: reduce) {
    .gate__blob { animation: none; }
  }
</style>
