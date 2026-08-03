export default interface ChannelOverridesArgs {
  error: string | undefined
  mapDraft: string
  onMapDraftChange: (draft: string) => void
  onPbapDraftChange: (draft: string) => void
  onSave: () => void
  pbapDraft: string
  saving: boolean
}
