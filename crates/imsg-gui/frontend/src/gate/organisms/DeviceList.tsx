import DeviceListItem from '@/ui/molecules/DeviceListItem.tsx'
import type { PairedDeviceDto } from '@/bindings.ts'

interface DeviceListProps {
  devices: PairedDeviceDto[]
  onSelect: (address: string) => void
}

const DeviceList = ({ devices, onSelect }: DeviceListProps): React.JSX.Element => (
  <ul className="flex w-full flex-col gap-2">
    {devices.map((device) => (
      <DeviceListItem key={device.address} device={device} onSelect={onSelect} />
    ))}
  </ul>
)

export default DeviceList
