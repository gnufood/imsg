import type { Meta, StoryObj } from '@storybook/react-vite'
import ContactDetail from '@/contacts/organisms/ContactDetail.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    contact: { display_name: 'Jane Doe', phones: ['+15550001', '+15550002'], uid: 'U1' },
    emptyMessage: 'Select a contact from the list.',
    failed: false,
    loading: false,
    onRetry: fn(),
  },
  component: ContactDetail,
  parameters: { layout: 'fullscreen' },
  title: 'organisms/ContactDetail',
} satisfies Meta<typeof ContactDetail>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const NoPhones: Story = {
  args: { contact: { display_name: 'Jane Doe', phones: [], uid: 'U1' } },
}

export const Unnamed: Story = {
  args: { contact: { display_name: null, phones: ['+15550001'], uid: 'U1' } },
}

export const Empty: Story = {
  args: { contact: undefined },
}

export const Loading: Story = {
  args: { contact: undefined, loading: true },
}

export const Failed: Story = {
  args: { contact: undefined, failed: true },
}
