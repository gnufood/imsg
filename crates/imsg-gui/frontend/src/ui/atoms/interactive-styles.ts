interface InteractiveStyles {
  disabled: string
  disabledContainer: string
  disabledHoverReset: string
  focus: string
  hasFocus: string
  hover: string
  peerFocus: string
}

const interactiveStyles: InteractiveStyles = {
  disabled: 'disabled:cursor-not-allowed disabled:opacity-50',
  disabledContainer: 'has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50',
  disabledHoverReset: 'disabled:hover:bg-transparent',
  focus: 'focus:outline-none focus:ring-1 focus:ring-accent',
  hasFocus: 'has-[:focus-visible]:outline-none has-[:focus-visible]:ring-1 has-[:focus-visible]:ring-accent',
  hover: 'hover:bg-ink/5',
  peerFocus: 'peer-focus:outline-none peer-focus:ring-1 peer-focus:ring-accent',
}

export default interactiveStyles
