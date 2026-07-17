import DeviceListItem from '@/ui/molecules/DeviceListItem.tsx'
import EmptyState from '@/ui/molecules/EmptyState.tsx'
import type { PairedDeviceDto } from '@/bindings.ts'

interface DeviceListProps {
  devices: PairedDeviceDto[]
  onSelect: (address: string) => void
}

const DeviceList = ({ devices, onSelect }: DeviceListProps): React.JSX.Element => {
  if (devices.length === 0) {
    return <EmptyState message="No paired devices found." />
  }

  return (
    <ul className="flex w-full flex-col gap-2">
      {devices.map((device) => (
        <DeviceListItem key={device.address} device={device} onSelect={onSelect} />
      ))}
    </ul>
  )
}

export default DeviceList
