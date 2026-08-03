import type { ContactEntryDto } from '@/bindings.ts'

export default interface UseContactsListResult {
  contacts: ContactEntryDto[] | undefined
  failed: boolean
  hasNextPage: boolean
  hasPrevPage: boolean
  nextPage: () => void
  prevPage: () => void
  retry: () => void
}
