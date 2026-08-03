import type { Meta, StoryObj } from '@storybook/react-vite'
import Contacts from '@/contacts/pages/Contacts.tsx'
import { fn } from 'storybook/test'

const meta = {
  args: {
    contacts: [
      { display_name: 'Ada Lovelace', uid: 'U1' },
      { display_name: null, uid: 'U2' },
    ],
    detailContact: { display_name: 'Ada Lovelace', phones: ['+15550001'], uid: 'U1' },
    detailEmptyMessage: 'Select a contact from the list.',
    detailFailed: false,
    detailLoading: false,
    hasNextPage: false,
    hasPrevPage: false,
    listFailed: false,
    onClearSearch: fn(),
    onNextPage: fn(),
    onPrevPage: fn(),
    onRefresh: fn(),
    onRetryDetail: fn(),
    onRetryList: fn(),
    onSearch: fn(),
    onSelectContact: fn(),
    refreshing: false,
    searchActive: false,
    searchError: undefined,
    searching: false,
    syncError: undefined,
  },
  component: Contacts,
  parameters: { layout: 'fullscreen' },
  title: 'pages/Contacts',
} satisfies Meta<typeof Contacts>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
