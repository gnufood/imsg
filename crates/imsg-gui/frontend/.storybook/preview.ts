import '../src/index.css'
import type { Preview } from '@storybook/react-vite'
import { withThemeByDataAttribute } from '@storybook/addon-themes'

// The app itself only ever sets `data-theme` post-gate (`AppReady`, see
// Theme/application/use-theme-preference.ts) — Storybook has no gate and no `AppReady`, so
// Without this decorator there's no way to preview dark mode here at all. Defaults
// (`attributeName: 'data-theme'`, `parentSelector: 'html'`) already match what the app itself
// Sets, so no override needed. `backgrounds.surface` already references `var(--color-surface)`,
// So it follows this automatically.
const preview: Preview = {
  decorators: [
    withThemeByDataAttribute({
      defaultTheme: 'light',
      themes: { dark: 'dark', light: 'light' },
    }),
  ],
  parameters: {
    backgrounds: {
      default: 'surface',
      options: {
        surface: { name: 'surface', value: 'var(--color-surface)' },
      },
    },
    layout: 'centered',
    // Alphabetical default puts `pages` before `templates`; this enforces the atomic-design tier
    // Order (`GUI_ATOMIC_DESIGN.md`) instead. Stories within a tier still sort alphabetically.
    options: {
      storySort: {
        order: ['tokens', 'atoms', 'molecules', 'organisms', 'templates', 'pages'],
      },
    },
  },
}

export default preview
