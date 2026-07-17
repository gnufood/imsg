export default interface UseSendResult {
  error: string | undefined
  send: (text: string) => void
  sending: boolean
}
