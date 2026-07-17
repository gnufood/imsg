import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import DeviceList from '@/gate/organisms/DeviceList.tsx'
import type { DeviceSetupState } from '@/gate/application/use-device-setup-flow.ts'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'

const screen = (children: React.ReactNode): React.JSX.Element => <CenteredScreen>{children}</CenteredScreen>

interface DeviceSetupTemplateProps {
  onRetryList: () => void
  onRetryPersist: () => void
  onRetryResolve: () => void
  onSelectDevice: (address: string) => void
  state: DeviceSetupState
}

const DeviceSetupTemplate = ({
  onRetryList,
  onRetryPersist,
  onRetryResolve,
  onSelectDevice,
  state,
}: DeviceSetupTemplateProps): React.JSX.Element => {
  const { stage, devices, errorMessage } = state
  switch (stage) {
    case 'listing': {
      return screen(<LoadingState message="Looking for paired devices…" />)
    }
    case 'listError': {
      return screen(<ErrorState message={errorMessage ?? "Couldn't list paired devices."} onRetry={onRetryList} />)
    }
    case 'picking': {
      return screen(
        <div className="flex w-full max-w-sm flex-col gap-4">
          <h1 className="text-sm text-muted">Choose a device</h1>
          <DeviceList devices={devices} onSelect={onSelectDevice} />
        </div>,
      )
    }
    case 'resolving': {
      return screen(<LoadingState message="Checking messaging support…" />)
    }
    case 'resolveError': {
      return screen(<ErrorState message={errorMessage ?? "Couldn't check messaging support."} onRetry={onRetryResolve} />)
    }
    case 'unsupported': {
      return screen(<ErrorState message="This device doesn't support messaging." onRetry={onRetryList} />)
    }
    case 'persisting': {
      return screen(<LoadingState message="Saving device…" />)
    }
    case 'persistError': {
      return screen(<ErrorState message={errorMessage ?? "Couldn't save device."} onRetry={onRetryPersist} />)
    }
  }
}

export default DeviceSetupTemplate
