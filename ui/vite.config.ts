import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

/** Static SPA: the same build is served by the seed node, wrapped by Tauri, and packed
 *  into the .webhapp. No server code — all data comes from a Holochain node. */
export default defineConfig({
  plugins: [
    sveltekit({
      adapter: adapter({ fallback: 'index.html' }),
      compilerOptions: { runes: true },
    }),
  ],
});
