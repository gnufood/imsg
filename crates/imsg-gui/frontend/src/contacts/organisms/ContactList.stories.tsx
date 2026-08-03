import type { Meta, StoryObj } from '@storybook/react-vite'
import ContactList from '@/contacts/organisms/ContactList.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    contacts: [
      { display_name: 'Ada Lovelace', uid: 'U1' },
      { display_name: null, uid: 'U2' },
      { display_name: 'Jane Doe', uid: 'U3' },
    ],
    onSelect: fn(),
  },
  component: ContactList,
  title: 'organisms/ContactList',
} satisfies Meta<typeof ContactList>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Empty: Story = {
  args: { contacts: [] },
}
