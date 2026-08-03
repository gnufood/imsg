export default interface UseChannelOverridesResult {
  error: string | undefined
  mapDraft: string
  onMapDraftChange: (draft: string) => void
  onPbapDraftChange: (draft: string) => void
  pbapDraft: string
  save: () => void
  saving: boolean
}
