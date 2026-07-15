import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// Tauri expects a fixed, predictable dev server port.
// `strictPort` fails fast instead of silently drifting to a different port that
// `tauri.conf.json`'s `devUrl` wouldn't match.
export default defineConfig({
  clearScreen: false,
  plugins: [react(), tailwindcss()],
  server: {
    port: 1420,
    strictPort: true,
  },
})
