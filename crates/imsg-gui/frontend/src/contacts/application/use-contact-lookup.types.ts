import type { ContactDto } from '@/bindings.ts'

export default interface UseContactLookupResult {
  clear: () => void
  // `undefined` — no search run yet. `null` — searched, no contact owns that number.
  contact: ContactDto | null | undefined
  error: string | undefined
  search: (number: string) => void
  searching: boolean
}
