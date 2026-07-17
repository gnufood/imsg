import type { Meta, StoryObj } from '@storybook/react-vite'
import Spinner from './Spinner.tsx'

const meta = {
  component: Spinner,
  title: 'atoms/Spinner',
} satisfies Meta<typeof Spinner>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
