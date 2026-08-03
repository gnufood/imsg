import type { ContactDto } from '@/bindings.ts'

export default interface UseContactDetailResult {
  contact: ContactDto | undefined
  failed: boolean
  retry: () => void
}
