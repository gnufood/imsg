import type { Meta, StoryObj } from '@storybook/react-vite'
import Modal from '@/ui/atoms/Modal.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    children: 'Modal content fixture',
    label: 'Example dialog',
    onClose: fn(),
  },
  component: Modal,
  parameters: { layout: 'fullscreen' },
  title: 'atoms/Modal',
} satisfies Meta<typeof Modal>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
