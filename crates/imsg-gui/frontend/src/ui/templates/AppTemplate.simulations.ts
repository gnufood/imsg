import { useCallback, useState } from 'react'
import type ChannelOverridesArgs from '@/settings/organisms/ChannelOverridesForm.types.ts'
import type DaemonControlsArgs from '@/settings/organisms/DaemonControls.types.ts'
import type SecurityLevelArgs from '@/settings/organisms/SecurityLevelForm.types.ts'
import { fn } from 'storybook/test'

const SIMULATED_DELAY_MS = 400

interface SecuritySimState {
  committedLevel: SecurityLevelArgs['draft']
  draft: SecurityLevelArgs['draft']
  saving: boolean
}

const SECURITY_SIM_INITIAL: SecuritySimState = { committedLevel: 'Medium', draft: 'Medium', saving: false }

const useSimulatedSecurityLevel = (): SecurityLevelArgs => {
  const [state, setState] = useState(SECURITY_SIM_INITIAL)

  const onDraftChange = useCallback((draft: SecurityLevelArgs['draft']) => {
    setState((current) => ({ ...current, draft }))
  }, [])

  const onCancel = useCallback(() => {
    setState((current) => ({ ...current, draft: current.committedLevel }))
  }, [])

  const onSave = useCallback(() => {
    setState((current) => ({ ...current, saving: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, committedLevel: current.draft, saving: false }))
    }, SIMULATED_DELAY_MS)
  }, [])

  return { committedLevel: state.committedLevel, draft: state.draft, onCancel, onDraftChange, onSave, saveError: undefined, saving: state.saving }
}

interface DaemonSimState {
  installing: boolean
  uninstalling: boolean
  userInstalled: boolean
}

const DAEMON_SIM_INITIAL: DaemonSimState = { installing: false, uninstalling: false, userInstalled: false }

const useSimulatedDaemonControls = (): DaemonControlsArgs => {
  const [state, setState] = useState(DAEMON_SIM_INITIAL)

  const onInstall = useCallback(() => {
    setState((current) => ({ ...current, installing: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, installing: false, userInstalled: true }))
    }, SIMULATED_DELAY_MS)
  }, [])

  const onUninstall = useCallback(() => {
    setState((current) => ({ ...current, uninstalling: true }))
    setTimeout(() => {
      setState((current) => ({ ...current, uninstalling: false, userInstalled: false }))
    }, SIMULATED_DELAY_MS)
  }, [])

  return {
    installError: undefined,
    installing: state.installing,
    onInstall,
    onRestart: fn(),
    onResumeServiceStatusPolling: fn(),
    onStop: fn(),
    onUninstall,
    restartError: undefined,
    restarting: false,
    serviceStatusPollFailed: false,
    stopError: undefined,
    stopping: false,
    systemInstalled: false,
    uninstallError: undefined,
    uninstalling: state.uninstalling,
    userInstalled: state.userInstalled,
  }
}

interface ChannelSimState {
  committedMap: number
  committedPbap: number
  detecting: boolean
  editorOpen: boolean
  mapDraft: string
  pbapDraft: string
  saving: boolean
}

const CHANNEL_SIM_INITIAL: ChannelSimState = {
  committedMap: 8,
  committedPbap: 12,
  detecting: false,
  editorOpen: false,
  mapDraft: '8',
  pbapDraft: '12',
  saving: false,
}

const CHANNEL_SIM_DETECTED = { map: '16', pbap: '19' }

const withRevertedDrafts = (current: ChannelSimState): ChannelSimState => ({
  ...current,
  editorOpen: false,
  mapDraft: String(current.committedMap),
  pbapDraft: String(current.committedPbap),
})

const withDetectedDrafts = (current: ChannelSimState): ChannelSimState => ({
  ...current,
  detecting: false,
  mapDraft: CHANNEL_SIM_DETECTED.map,
  pbapDraft: CHANNEL_SIM_DETECTED.pbap,
})

const withCommittedDrafts = (current: ChannelSimState): ChannelSimState => ({
  ...current,
  committedMap: Number(current.mapDraft),
  committedPbap: Number(current.pbapDraft),
  editorOpen: false,
  saving: false,
})

const useSimulatedChannelOverrides = (): ChannelOverridesArgs => {
  const [state, setState] = useState(CHANNEL_SIM_INITIAL)

  const onMapDraftChange = useCallback((draft: string) => {
    setState((current) => ({ ...current, mapDraft: draft }))
  }, [])

  const onPbapDraftChange = useCallback((draft: string) => {
    setState((current) => ({ ...current, pbapDraft: draft }))
  }, [])

  const onRequestEdit = useCallback(() => {
    setState((current) => ({ ...current, editorOpen: true }))
  }, [])

  const onCancel = useCallback(() => {
    setState(withRevertedDrafts)
  }, [])

  const onDetect = useCallback(() => {
    setState((current) => ({ ...current, detecting: true }))
    setTimeout(() => setState(withDetectedDrafts), SIMULATED_DELAY_MS)
  }, [])

  const onSave = useCallback(() => {
    setState((current) => ({ ...current, saving: true }))
    setTimeout(() => setState(withCommittedDrafts), SIMULATED_DELAY_MS)
  }, [])

  return {
    detectError: undefined,
    detecting: state.detecting,
    editorOpen: state.editorOpen,
    mapChannel: state.committedMap,
    mapDraft: state.mapDraft,
    onCancel,
    onDetect,
    onMapDraftChange,
    onPbapDraftChange,
    onRequestEdit,
    onSave,
    pbapChannel: state.committedPbap,
    pbapDraft: state.pbapDraft,
    saveError: undefined,
    saving: state.saving,
  }
}

interface SimulatedSettings {
  channelOverrides: ChannelOverridesArgs
  daemonControls: DaemonControlsArgs
  securityLevel: SecurityLevelArgs
}

const useSimulatedSettings = (): SimulatedSettings => {
  const channelOverrides = useSimulatedChannelOverrides()
  const daemonControls = useSimulatedDaemonControls()
  const securityLevel = useSimulatedSecurityLevel()
  return { channelOverrides, daemonControls, securityLevel }
}

export default useSimulatedSettings
