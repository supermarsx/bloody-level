// Tauri serves the SPA from a single bundle.
// Pure SPA mode: no SSR, no prerendering — the static `fallback: index.html`
// from `adapter-static` plus client-side routing handles every URL.
export const ssr = false;
export const prerender = false;
export const trailingSlash = "never";
