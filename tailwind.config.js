/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: ['selector', '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        bg1: 'rgb(var(--bg-1) / <alpha-value>)',
        bg2: 'rgb(var(--bg-2) / <alpha-value>)',
        bg3: 'rgb(var(--bg-3) / <alpha-value>)',
        fg1: 'rgb(var(--fg-1) / <alpha-value>)',
        fg2: 'rgb(var(--fg-2) / <alpha-value>)',
        fg3: 'rgb(var(--fg-3) / <alpha-value>)',
        line: 'rgb(var(--line) / <alpha-value>)',
        accent: 'rgb(var(--accent) / <alpha-value>)',
        ok: 'rgb(var(--ok) / <alpha-value>)',
        warn: 'rgb(var(--warn) / <alpha-value>)',
        crit: 'rgb(var(--crit) / <alpha-value>)'
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'Cascadia Code', 'Consolas', 'monospace']
      }
    }
  },
  plugins: [require('@tailwindcss/forms'), require('@tailwindcss/typography')]
};
