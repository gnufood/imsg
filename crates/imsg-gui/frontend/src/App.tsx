import Button from '@/ui/atoms/Button.tsx'
import CenteredScreen from '@/ui/templates/CenteredScreen.tsx'
import DeviceList from '@/gate/organisms/DeviceList.tsx'
import ErrorState from '@/ui/molecules/ErrorState.tsx'
import LoadingState from '@/ui/molecules/LoadingState.tsx'
import type { PairedDeviceDto } from '@/bindings.ts'
import Preparing from '@/gate/organisms/Preparing.tsx'
import Spinner from '@/ui/atoms/Spinner.tsx'
import Splash from '@/gate/organisms/Splash.tsx'

// Scratch preview of everything built so far — not the real `Gate` flow (see
// GUI_FRONTEND.md's "Pages remaining"). Swap out once Gate/DeviceSetup land.

const noop = (): void => {}

const MOCK_DEVICES: PairedDeviceDto[] = [
  { address: '00:11:22:33:44:55', name: 'Pixel 8' },
  { address: 'AA:BB:CC:DD:EE:FF', name: "Ethan's AirPods" },
]

const App = (): React.JSX.Element => (
  <CenteredScreen>
    <div className="flex w-full max-w-sm flex-col gap-8">
      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">Atoms</h2>
        <Spinner />
        <Button onClick={noop}>Plain button</Button>
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">LoadingState</h2>
        <LoadingState message="Connecting to daemon…" />
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">ErrorState</h2>
        <ErrorState message="Couldn't reach the daemon." onRetry={noop} />
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">DeviceList</h2>
        <DeviceList devices={MOCK_DEVICES} onSelect={noop} />
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">Preparing (loading)</h2>
        <Preparing stage="daemon" onRetry={noop} />
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">Preparing (error)</h2>
        <Preparing stage="sync" error="Sync failed." onRetry={noop} />
      </section>

      <section className="flex flex-col items-center gap-3">
        <h2 className="text-xs uppercase tracking-wide text-muted">Splash</h2>
        <Splash ready onDone={noop} />
      </section>
    </div>
  </CenteredScreen>
)

export default App
