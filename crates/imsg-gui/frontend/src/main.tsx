import '@/index.css'

import App from '@/App.tsx'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

// `WEBDRIVER_BUILD` (not `import.meta.env.DEV`) — a WebDriver test build can be `--release`
// (Rust `webdriver` feature doesn't imply a debug frontend build), so gating on dev/prod mode
// Would both miss that case and leak the plugin into ordinary production builds. It's a
// `vite.config.ts`-injected literal, not an `import.meta.env` property read, so that when it's
// `false` this whole branch — dynamic import included — is dead code Rollup strips from the
// Output entirely, not just left unreachable at runtime. See Cargo.toml/build.rs for the
// Matching Rust-side `webdriver` feature gate.
if (WEBDRIVER_BUILD) {
  await import('@wdio/tauri-plugin')
}

const container = document.querySelector('#root')
if (container === null) {
  throw new Error('#root element not found')
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
