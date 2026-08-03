import type { Meta, StoryObj } from '@storybook/react-vite'
import ContactListItem from '@/ui/molecules/ContactListItem.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    onSelect: fn(),
  },
  component: ContactListItem,
  // <li> requires a list ancestor for correct a11y semantics.
  decorators: [
    (Story) => (
      <ul>
        <Story />
      </ul>
    ),
  ],
  title: 'molecules/ContactListItem',
} satisfies Meta<typeof ContactListItem>

export default meta
type Story = StoryObj<typeof meta>

export const Named: Story = {
  args: {
    contact: { display_name: 'Jane Doe', uid: 'U1' },
  },
}

export const Unnamed: Story = {
  args: {
    contact: { display_name: null, uid: 'U1' },
  },
}
