import { URL, fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// Tauri expects a fixed, predictable dev server port.
// `strictPort` fails fast instead of silently drifting to a different port that
// `tauri.conf.json`'s `devUrl` wouldn't match.
export default defineConfig({
  clearScreen: false,
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
