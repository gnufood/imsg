import type { ConfigDto } from '@/bindings.ts'

export default interface UseConfigResult {
  config: ConfigDto | undefined
  failed: boolean
  reload: () => void
}
