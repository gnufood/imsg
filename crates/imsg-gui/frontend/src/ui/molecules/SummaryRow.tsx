import Text from '@/ui/atoms/Text.tsx'

interface SummaryRowProps {
  // Short (2-4 char) mark — e.g. Channels' "MAP"/"PBAP" — same plain-ink treatment as
  // `ServiceStatusCard`'s title, not a badge.
  mark: string
  value: string
  valueLabel: string
}

// Read-only row for a settings summary list — the mark on the left, "<label> <value>" on the
// Right (e.g. "Channel 8"). Sibling to `ServiceStatusCard` but without an action button; the
// List it's part of owns whatever affordance edits the underlying value (see
// `ChannelOverridesForm`). `value`'s fixed-width column keeps `valueLabel` at a constant x
// Position regardless of digit count — without it, a 2-digit value would push "Channel" further
// Left than a 1-digit one since the pair is right-aligned as a group.
const SummaryRow = ({ mark, value, valueLabel }: SummaryRowProps): React.JSX.Element => (
  <div className="flex items-center justify-between gap-3 px-3 py-2.5">
    <Text>{mark}</Text>
    <div className="flex items-baseline gap-1">
      <Text size="xs" tone="muted">
        {valueLabel}
      </Text>
      <span className="inline-block w-5 text-right">
        <Text as="span">{value}</Text>
      </span>
    </div>
  </div>
)

export default SummaryRow
