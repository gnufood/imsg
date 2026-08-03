import Button from '@/ui/atoms/Button.tsx'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import Text from '@/ui/atoms/Text.tsx'
import TextInput from '@/ui/atoms/TextInput.tsx'

const ChannelOverridesForm = ({
  error,
  mapDraft,
  onMapDraftChange,
  onPbapDraftChange,
  onSave,
  pbapDraft,
  saving,
}: ChannelOverridesArgs): React.JSX.Element => (
  <div className="flex flex-col gap-3">
    <div className="flex flex-col gap-1">
      <label htmlFor="settings-map-channel">
        <Text as="span" size="xs" tone="muted">
          MAP channel
        </Text>
      </label>
      <TextInput disabled={saving} id="settings-map-channel" onChange={onMapDraftChange} value={mapDraft} />
    </div>
    <div className="flex flex-col gap-1">
      <label htmlFor="settings-pbap-channel">
        <Text as="span" size="xs" tone="muted">
          PBAP channel
        </Text>
      </label>
      <TextInput disabled={saving} id="settings-pbap-channel" onChange={onPbapDraftChange} value={pbapDraft} />
    </div>
    {error !== undefined && (
      <Text size="xs" tone="muted">
        {error}
      </Text>
    )}
    <Button disabled={saving} onClick={onSave}>
      Save
    </Button>
  </div>
)

export default ChannelOverridesForm
