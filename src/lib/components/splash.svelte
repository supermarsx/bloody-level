<script lang="ts">
  // Branded splash overlay shown during initial app boot. Fades out smoothly
  // once the parent flips `visible = false`, so the main layout's first paint
  // doesn't snap into view.
  let { visible = true, message = 'Loading…' } = $props<{
    visible?: boolean;
    message?: string;
  }>();
</script>

<div
  class="splash"
  class:splash--hidden={!visible}
  aria-hidden={!visible}
  role="status"
  aria-live="polite"
>
  <div class="splash__inner">
    <div class="splash__logo">
      <!-- Stylized waveform — visually evokes a longitudinal lab-value trend. -->
      <svg viewBox="0 0 96 96" width="96" height="96" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <defs>
          <linearGradient id="splashGrad" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stop-color="var(--splash-grad-from, #5b8bff)" />
            <stop offset="100%" stop-color="var(--splash-grad-to, #8e5bff)" />
          </linearGradient>
        </defs>
        <circle cx="48" cy="48" r="44" stroke="url(#splashGrad)" stroke-width="2" opacity="0.25" />
        <path
          d="M14 56 L26 48 L36 60 L48 32 L58 50 L70 38 L82 44"
          stroke="url(#splashGrad)"
          stroke-width="3"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="splash__waveform"
        />
        <circle cx="48" cy="32" r="3" fill="url(#splashGrad)" class="splash__dot" />
      </svg>
    </div>
    <div class="splash__title">bloody-level</div>
    <div class="splash__bar" aria-hidden="true">
      <div class="splash__bar-fill"></div>
    </div>
    <div class="splash__msg">{message}</div>
  </div>
</div>

<style>
  .splash {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: rgb(var(--bg-1, 11 13 16));
    color: rgb(var(--fg-1, 243 244 246));
    transition: opacity 220ms ease, visibility 0s linear 0s;
    opacity: 1;
  }
  .splash--hidden {
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 220ms ease, visibility 0s linear 220ms;
  }
  .splash__inner {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
  }
  .splash__logo {
    animation: splash-pulse 2s ease-in-out infinite;
  }
  .splash__title {
    font-size: 0.95rem;
    letter-spacing: 0.02em;
    font-weight: 600;
    color: rgb(var(--fg-1, 243 244 246));
  }
  .splash__msg {
    font-size: 0.75rem;
    color: rgb(var(--fg-2, 156 163 175));
    min-height: 1em;
  }
  .splash__bar {
    width: 12rem;
    height: 2px;
    background: rgb(var(--bg-3, 31 36 43));
    border-radius: 1px;
    overflow: hidden;
    position: relative;
  }
  .splash__bar-fill {
    position: absolute;
    inset: 0;
    width: 35%;
    background: linear-gradient(
      90deg,
      transparent 0%,
      var(--splash-grad-from, #5b8bff) 50%,
      transparent 100%
    );
    animation: splash-sweep 1.4s ease-in-out infinite;
  }
  .splash__waveform {
    stroke-dasharray: 220;
    stroke-dashoffset: 220;
    animation: splash-trace 2s ease-out forwards 0.1s;
  }
  .splash__dot {
    transform-origin: 48px 32px;
    animation: splash-dot 2s ease-in-out infinite;
  }
  @keyframes splash-trace {
    to {
      stroke-dashoffset: 0;
    }
  }
  @keyframes splash-pulse {
    0%, 100% { transform: scale(1); }
    50%      { transform: scale(1.04); }
  }
  @keyframes splash-sweep {
    0%   { transform: translateX(-100%); }
    100% { transform: translateX(285%); }
  }
  @keyframes splash-dot {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%      { opacity: 0.4; transform: scale(1.4); }
  }

  @media (prefers-reduced-motion: reduce) {
    .splash__logo,
    .splash__waveform,
    .splash__dot,
    .splash__bar-fill {
      animation: none !important;
    }
  }
</style>
