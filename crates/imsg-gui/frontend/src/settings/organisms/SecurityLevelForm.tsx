import { useCallback, useState } from 'react'
import Button from '@/ui/atoms/Button.tsx'
import CompactButton from '@/ui/atoms/CompactButton.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import type { SecurityLevelDto } from '@/bindings.ts'
import Select from '@/ui/atoms/Select.tsx'
import SummaryRow from '@/ui/molecules/SummaryRow.tsx'
import Text from '@/ui/atoms/Text.tsx'

// Matches `SecurityLevelDto`'s doc order (weakest to strongest); `Sdp` reads as its own acronym.
const LEVEL_LABELS: Record<SecurityLevelDto, string> = { High: 'High', Low: 'Low', Medium: 'Medium', Sdp: 'SDP' }

const LEVEL_OPTIONS: { label: string; value: SecurityLevelDto }[] = [
  { label: 'SDP (none)', value: 'Sdp' },
  { label: 'Low', value: 'Low' },
  { label: 'Medium', value: 'Medium' },
  { label: 'High', value: 'High' },
]

// `null` means never explicitly configured — `config::BrokerConfig::security_level` then leaves
// The kernel's already-negotiated pairing/bond security untouched. Distinct from `undefined`
// (Still loading), which renders `LoadingState` instead below.
const summaryValue = (committedLevel: SecurityLevelDto | null): string => {
  if (committedLevel === null) {
    return 'Default'
  }
  return LEVEL_LABELS[committedLevel]
}

const renderSummary = (committedLevel: SecurityLevelDto | null): React.JSX.Element => (
  <div className="flex flex-col divide-y divide-line rounded-lg border border-line">
    <SummaryRow mark="Security" value={summaryValue(committedLevel)} valueLabel="Level" />
  </div>
)

interface EditorArgs {
  draft: SecurityLevelDto
  onCancel: () => void
  onDraftChange: (value: string) => void
  onSave: () => void
  pending: boolean
  saveError: string | undefined
}

const renderEditor = ({ draft, onCancel, onDraftChange, onSave, pending, saveError }: EditorArgs): React.JSX.Element => (
  <div className="flex flex-col gap-3 rounded-lg border border-line p-4">
    <Text as="h2" tone="accent">
      RFCOMM security level
    </Text>
    <Select disabled={pending} onChange={onDraftChange} options={LEVEL_OPTIONS} value={draft} />
    <Text size="xs" tone="muted">
      Raising this above the negotiated pairing security can break the MAP connection.
    </Text>
    {saveError !== undefined && (
      <Text size="xs" tone="muted">
        {saveError}
      </Text>
    )}
    <div className="flex justify-end gap-1.5">
      <CompactButton disabled={pending} onClick={onCancel}>
        Cancel
      </CompactButton>
      <CompactButton disabled={pending} onClick={onSave}>
        Apply
      </CompactButton>
    </div>
  </div>
)

// Read-only summary is always shown; the editor is progressive disclosure behind `isEditing` —
// Same pattern as `ChannelOverridesForm`. `onCancel` reverts the hook's draft back to the last
// Committed value (see `use-security-level.ts`), so the only way out of the editor is discard
// (Cancel) or persist (Apply).
const SecurityLevelForm = ({ committedLevel, draft, onCancel, onDraftChange, onSave, saveError, saving }: SecurityLevelArgs): React.JSX.Element => {
  const [isEditing, setIsEditing] = useState(false)

  const openEditor = useCallback(() => {
    setIsEditing(true)
  }, [])

  const closeEditor = useCallback(() => {
    onCancel()
    setIsEditing(false)
  }, [onCancel])

  // `Select`'s `onChange` is string-typed (it's generic over any fixed option list, not just
  // This enum) — `LEVEL_OPTIONS`' values are the only strings it can report, so this narrowing is
  // Safe.
  const handleDraftChange = useCallback(
    (value: string) => {
      onDraftChange(value as SecurityLevelDto)
    },
    [onDraftChange],
  )

  if (committedLevel === undefined) {
    return <LoadingState message="Loading security level…" />
  }

  return (
    <div className="flex flex-col gap-3">
      {renderSummary(committedLevel)}
      {!isEditing && <Button onClick={openEditor}>Change…</Button>}
      {isEditing &&
        renderEditor({
          draft,
          onCancel: closeEditor,
          onDraftChange: handleDraftChange,
          onSave,
          pending: saving,
          saveError,
        })}
    </div>
  )
}

export default SecurityLevelForm
