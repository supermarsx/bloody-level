import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),
    alias: {
      $api: 'src/lib/api',
      $charts: 'src/lib/charts',
      $components: 'src/lib/components',
      $theme: 'src/lib/theme',
      $format: 'src/lib/format'
    }
  }
};

export default config;
