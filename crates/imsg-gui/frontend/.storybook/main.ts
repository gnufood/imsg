import type { StorybookConfig } from '@storybook/react-vite'

const config: StorybookConfig = {
  addons: ['@storybook/addon-docs', '@storybook/addon-themes'],
  framework: '@storybook/react-vite',
  staticDirs: ['../public'],
  stories: ['../src/**/*.stories.tsx', '../src/**/*.mdx'],
}

export default config
