import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'));

export default defineConfig({
  plugins: [svelte()],
  resolve: { conditions: ['browser'] },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_'],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    target: 'es2022',
    rollupOptions: {
      input: {
        main: 'index.html',
        overlay: 'overlay.html',
      },
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    globals: false,
    include: ['tests/**/*.test.ts'],
  },
});
