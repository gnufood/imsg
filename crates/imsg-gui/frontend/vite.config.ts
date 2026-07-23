import { URL, fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// Tauri expects a fixed, predictable dev server port.
// `strictPort` fails fast instead of silently drifting to a different port that
// `tauri.conf.json`'s `devUrl` wouldn't match.
export default defineConfig({
  clearScreen: false,
  // A genuine `true`/`false` literal (not `import.meta.env.VITE_WEBDRIVER` compared at
  // Runtime) — Rollup only fully eliminates a dynamically-imported module's chunk when the
  // Guarding condition folds to a literal at build time; a plain env-object property read
  // Doesn't fold, so `@wdio/tauri-plugin` would still get its own (never-fetched, but
  // Present) chunk in dist/ otherwise. See main.tsx.
  define: {
    WEBDRIVER_BUILD: JSON.stringify(process.env['VITE_WEBDRIVER'] === 'true'),
  },
  plugins: [react(), tailwindcss()],
  resolve: {
    // Mirrors tsconfig.app.json's `paths` — keep both in sync.
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url)),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
  },
})
