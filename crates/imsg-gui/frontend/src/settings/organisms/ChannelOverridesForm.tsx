import { useCallback, useState } from 'react'
import Button from '@/ui/atoms/Button.tsx'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import CompactButton from '@/ui/atoms/CompactButton.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import Select from '@/ui/atoms/Select.tsx'
import SummaryRow from '@/ui/molecules/SummaryRow.tsx'
import Text from '@/ui/atoms/Text.tsx'

// RFCOMM server channels are 1-30 (5-bit DLCI) — matches `imsg-config`'s `validate_channel`.
const CHANNEL_OPTIONS = Array.from({ length: 30 }, (_unused, index) => ({ label: String(index + 1), value: String(index + 1) }))

const renderSummary = (mapChannel: number, pbapChannel: number): React.JSX.Element => (
  <div className="flex flex-col divide-y divide-line rounded-lg border border-line">
    <SummaryRow mark="MAP" value={String(mapChannel)} valueLabel="Channel" />
    <SummaryRow mark="PBAP" value={String(pbapChannel)} valueLabel="Channel" />
  </div>
)

interface FieldArgs {
  disabled: boolean
  id: string
  label: string
  onChange: (draft: string) => void
  value: string
}

const renderField = ({ disabled, id, label, onChange, value }: FieldArgs): React.JSX.Element => (
  <div className="flex flex-col gap-1">
    <label htmlFor={id}>
      <Text as="span" size="xs" tone="muted">
        {label}
      </Text>
    </label>
    <Select disabled={disabled} id={id} onChange={onChange} options={CHANNEL_OPTIONS} value={value} />
  </div>
)

interface EditorArgs {
  detectError: string | undefined
  mapDraft: string
  onCancel: () => void
  onDetect: () => void
  onMapDraftChange: (draft: string) => void
  onPbapDraftChange: (draft: string) => void
  onSave: () => void
  pbapDraft: string
  pending: boolean
  saveError: string | undefined
}

const renderEditor = ({
  detectError,
  mapDraft,
  onCancel,
  onDetect,
  onMapDraftChange,
  onPbapDraftChange,
  onSave,
  pbapDraft,
  pending,
  saveError,
}: EditorArgs): React.JSX.Element => (
  <div className="flex flex-col gap-3 rounded-lg border border-line p-4">
    <Text as="h2" tone="accent">
      Manual channel override
    </Text>
    <div className="grid grid-cols-2 gap-3">
      {renderField({ disabled: pending, id: 'settings-map-channel', label: 'MAP channel', onChange: onMapDraftChange, value: mapDraft })}
      {renderField({ disabled: pending, id: 'settings-pbap-channel', label: 'PBAP channel', onChange: onPbapDraftChange, value: pbapDraft })}
    </div>
    <Text size="xs" tone="muted">
      Manual assignments can conflict if another service is already using the selected channel.
    </Text>
    {(saveError ?? detectError) !== undefined && (
      <Text size="xs" tone="muted">
        {saveError ?? detectError}
      </Text>
    )}
    <div className="flex items-center justify-between gap-2">
      <CompactButton disabled={pending} ghost onClick={onDetect}>
        Detect from device
      </CompactButton>
      <div className="flex gap-1.5">
        <CompactButton disabled={pending} onClick={onCancel}>
          Cancel
        </CompactButton>
        <CompactButton disabled={pending} onClick={onSave}>
          Apply overrides
        </CompactButton>
      </div>
    </div>
  </div>
)

// Read-only `SummaryRow`s are always shown; the editable form is progressive disclosure behind
// `isEditing` — pure local UI state (not threaded through `use-channel-overrides.ts`), same
// Pattern as `DaemonControls`' `useUninstallConfirm`. `onCancel` reverts the hook's drafts back
// To the last committed values (see `use-channel-overrides.ts`), so the only way out of the
// Editor is either discard (Cancel) or persist (Apply) — never a half-edited draft left behind.
const ChannelOverridesForm = ({
  detectError,
  detecting,
  mapChannel,
  mapDraft,
  onCancel,
  onDetect,
  onMapDraftChange,
  onPbapDraftChange,
  onSave,
  pbapChannel,
  pbapDraft,
  saveError,
  saving,
}: ChannelOverridesArgs): React.JSX.Element => {
  const [isEditing, setIsEditing] = useState(false)

  const openEditor = useCallback(() => {
    setIsEditing(true)
  }, [])

  const closeEditor = useCallback(() => {
    onCancel()
    setIsEditing(false)
  }, [onCancel])

  if (mapChannel === undefined || pbapChannel === undefined) {
    return <LoadingState message="Loading channels…" />
  }

  return (
    <div className="flex flex-col gap-3">
      {renderSummary(mapChannel, pbapChannel)}
      {!isEditing && <Button onClick={openEditor}>Change manually…</Button>}
      {isEditing &&
        renderEditor({
          detectError,
          mapDraft,
          onCancel: closeEditor,
          onDetect,
          onMapDraftChange,
          onPbapDraftChange,
          onSave,
          pbapDraft,
          pending: detecting || saving,
          saveError,
        })}
    </div>
  )
}

export default ChannelOverridesForm
