export default interface UseChannelOverridesResult {
  cancel: () => void
  detect: () => void
  detectError: string | undefined
  detecting: boolean
  mapDraft: string
  onMapDraftChange: (draft: string) => void
  onPbapDraftChange: (draft: string) => void
  pbapDraft: string
  save: () => void
  saveError: string | undefined
  saving: boolean
}
