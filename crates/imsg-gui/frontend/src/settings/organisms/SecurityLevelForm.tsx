import CompactButton from '@/ui/atoms/CompactButton.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import type { SecurityLevelDto } from '@/bindings.ts'
import SegmentedControl from '@/ui/atoms/SegmentedControl.tsx'
import Text from '@/ui/atoms/Text.tsx'

// Matches `imsg-config::broker::SecurityLevel`'s own `BT_SECURITY_*` doc language and its
// Weakest-to-strongest variant order; `Sdp` reads as its own acronym, same as the enum's doc.
const LEVEL_LABELS: Record<SecurityLevelDto, string> = { High: 'High', Low: 'Low', Medium: 'Medium', Sdp: 'SDP' }

const LEVEL_OPTIONS: { label: string; value: SecurityLevelDto }[] = [
  { label: 'SDP', value: 'Sdp' },
  { label: 'Low', value: 'Low' },
  { label: 'Medium', value: 'Medium' },
  { label: 'High', value: 'High' },
]

// `null` means never explicitly configured — `config::BrokerConfig::security_level` then leaves
// The kernel's already-negotiated pairing/bond security untouched. There's no "unset" segment in
// `SecurityLevelDto` itself (the draft always resolves to one of the four tiers, `Sdp` by
// Convention — see `use-security-level.ts`'s `UNSET_FALLBACK`), so this label is the only place
// That distinction still surfaces once a segment is showing as selected.
const summaryValue = (committedLevel: SecurityLevelDto | null): string => {
  if (committedLevel === null) {
    return 'Default'
  }
  return LEVEL_LABELS[committedLevel]
}

// Verbatim from `imsg-config::broker::SecurityLevel`'s own variant doc comments, minus the
// Leading `BT_SECURITY_*` constant name — the tier the draft would actually commit to.
const TIER_DESCRIPTIONS: Record<SecurityLevelDto, string> = {
  High: 'Encryption and authentication required (MITM protection).',
  Low: 'No encryption or authentication required.',
  Medium: 'Encryption required; no authentication (no MITM protection).',
  Sdp: 'SDP-only traffic, no security.',
}

interface ActionsArgs {
  draft: SecurityLevelDto
  onCancel: () => void
  onSave: () => void
  pending: boolean
  saveError: string | undefined
}

const renderActions = ({ draft, onCancel, onSave, pending, saveError }: ActionsArgs): React.JSX.Element => (
  <div className="flex flex-col gap-2">
    {saveError !== undefined && (
      <Text size="xs" tone="muted">
        {saveError}
      </Text>
    )}
    <div className="flex items-center justify-between gap-3">
      <Text size="xs" tone="muted">
        {TIER_DESCRIPTIONS[draft]}
      </Text>
      <div className="flex shrink-0 gap-1.5">
        <CompactButton disabled={pending} onClick={onCancel}>
          Cancel
        </CompactButton>
        <CompactButton disabled={pending} onClick={onSave}>
          Apply
        </CompactButton>
      </div>
    </div>
  </div>
)

// Single card, always showing the four `BT_SECURITY` tiers as connected segments — no separate
// Read-only summary row or "Change…" reveal step. Selecting a segment only updates the draft;
// Apply/Cancel only appear once the draft actually differs from the committed value, since raising
// This above the negotiated pairing security can break the MAP connection and shouldn't apply on
// A single click.
const SecurityLevelForm = ({ committedLevel, draft, onCancel, onDraftChange, onSave, saveError, saving }: SecurityLevelArgs): React.JSX.Element => {
  if (committedLevel === undefined) {
    return <LoadingState message="Loading security level…" />
  }

  const isDirty = draft !== (committedLevel ?? 'Sdp')

  return (
    <div className="flex flex-col gap-3 rounded-lg border border-line p-4">
      <div className="flex items-center justify-between gap-3">
        <Text as="h2" tone="accent">
          RFCOMM
        </Text>
        <Text as="span">{summaryValue(committedLevel)}</Text>
      </div>
      <SegmentedControl disabled={saving} name="security-level" onChange={onDraftChange} options={LEVEL_OPTIONS} value={draft} />
      <Text size="xs" tone="muted">
        Raising this above the negotiated pairing security can break the MAP connection.
      </Text>
      {isDirty && renderActions({ draft, onCancel, onSave, pending: saving, saveError })}
    </div>
  )
}

export default SecurityLevelForm
