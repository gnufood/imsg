import type { StorybookConfig } from '@storybook/react-vite'

// Framework's Vite builder loads this project's root `vite.config.ts` automatically
// (Resolves `configDir/..` as project root) — the `@` alias and Tailwind plugin need no
// Duplication here.
const config: StorybookConfig = {
  addons: ['@storybook/addon-docs'],
  framework: '@storybook/react-vite',
  // Splash's story references `/splash.webp` from `public/` — Storybook's own dev/build
  // Server doesn't inherit Vite's `publicDir` the way `viteFinal` inherits the rest of the
  // Config, so it needs to be told explicitly.
  staticDirs: ['../public'],
  stories: ['../src/**/*.stories.tsx', '../src/**/*.mdx'],
}

export default config
