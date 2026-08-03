import Text from '@/ui/atoms/Text.tsx'

interface SummaryRowProps {
  mark: string
  value: string
  valueLabel: string
}

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
