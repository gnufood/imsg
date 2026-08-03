import type { Meta, StoryObj } from '@storybook/react-vite'
import ContactsTemplate from '@/contacts/templates/ContactsTemplate.tsx'
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
  component: ContactsTemplate,
  parameters: { layout: 'fullscreen' },
  title: 'templates/ContactsTemplate',
} satisfies Meta<typeof ContactsTemplate>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const LoadingList: Story = {
  args: { contacts: undefined },
}

export const ListFailed: Story = {
  args: { contacts: undefined, listFailed: true },
}

export const NoSelection: Story = {
  args: { detailContact: undefined },
}

export const SearchActive: Story = {
  args: { detailContact: { display_name: 'Jane Doe', phones: ['+15559999'], uid: 'U9' }, searchActive: true },
}

export const SearchNotFound: Story = {
  args: { detailContact: undefined, detailEmptyMessage: 'No contact found for that number.', searchActive: true },
}

export const Refreshing: Story = {
  args: { refreshing: true },
}
