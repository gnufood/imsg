export default interface ChannelOverridesArgs {
  detectError: string | undefined
  detecting: boolean
  mapChannel: number | undefined
  mapDraft: string
  onCancel: () => void
  onDetect: () => void
  onMapDraftChange: (draft: string) => void
  onPbapDraftChange: (draft: string) => void
  onSave: () => void
  pbapChannel: number | undefined
  pbapDraft: string
  saveError: string | undefined
  saving: boolean
}
