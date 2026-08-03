import '../src/index.css'
import type { Preview } from '@storybook/react-vite'
import { withThemeByDataAttribute } from '@storybook/addon-themes'

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
    options: {
      storySort: {
        order: ['tokens', 'atoms', 'molecules', 'organisms', 'templates', 'pages'],
      },
    },
  },
}

export default preview
