import { CirclePlus } from 'lucide-react'
import type { PairedDeviceDto } from '@/bindings.ts'
import Text from '@/ui/atoms/Text.tsx'
import interactiveStyles from '@/ui/atoms/interactive-styles.ts'
import { useCallback } from 'react'

interface DeviceListItemProps {
  device: PairedDeviceDto
  onSelect: (address: string) => void
}

// Expects a `<ul>`/`<ol>` ancestor — DeviceList owns the list semantics, this is just the `<li>`.
const DeviceListItem = ({ device, onSelect }: DeviceListItemProps): React.JSX.Element => {
  const handleClick = useCallback(() => {
    onSelect(device.address)
  }, [device.address, onSelect])

  return (
    <li>
      <button
        type="button"
        className={`flex w-full items-center justify-between gap-3 rounded-md border border-line px-3.5 py-2.5 text-left text-sm text-ink ${interactiveStyles.hover} ${interactiveStyles.focus}`}
        onClick={handleClick}
      >
        <span className="flex flex-col items-start">
          <Text as="span">{device.name ?? device.address}</Text>
          {device.name !== null && (
            <Text as="span" size="xs" tone="muted">
              {device.address}
            </Text>
          )}
        </span>
        <CirclePlus className="size-4 shrink-0 text-muted" aria-hidden="true" />
      </button>
    </li>
  )
}

export default DeviceListItem
