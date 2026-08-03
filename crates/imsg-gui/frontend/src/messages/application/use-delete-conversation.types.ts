export default interface UseDeleteConversationResult {
  cancelDelete: () => void
  confirmDelete: () => void
  confirmOpen: boolean
  deleting: boolean
  error: string | undefined
  requestDelete: () => void
}
