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
  },
}

export default preview
