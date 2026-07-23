import type { SecurityLevelDto } from '@/bindings.ts'

export default interface SecurityLevelArgs {
  committedLevel: SecurityLevelDto | null | undefined
  draft: SecurityLevelDto
  onCancel: () => void
  onDraftChange: (draft: SecurityLevelDto) => void
  onSave: () => void
  saveError: string | undefined
  saving: boolean
}
