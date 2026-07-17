import '../src/index.css'
import type { Preview } from '@storybook/react-vite'

const preview: Preview = {
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
