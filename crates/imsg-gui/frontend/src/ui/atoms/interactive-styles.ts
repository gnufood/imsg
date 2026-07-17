interface InteractiveStyles {
  disabled: string
  disabledContainer: string
  disabledHoverReset: string
  focus: string
  hover: string
}

// Single source for the hover/focus/disabled Tailwind classes shared by every clickable/
// Focusable atom and list-item molecule — `Button`/`IconButton`/list rows reuse `hover`+`focus`,
// `TextInput` reuses `focus` only. `disabledContainer` is for `Checkbox`, whose disabled state
// Lives on a nested `<input>` rather than the styled element itself, via `:has()`.
const interactiveStyles: InteractiveStyles = {
  disabled: 'disabled:cursor-not-allowed disabled:opacity-50',
  disabledContainer: 'has-[:disabled]:cursor-not-allowed has-[:disabled]:opacity-50',
  disabledHoverReset: 'disabled:hover:bg-transparent',
  focus: 'focus:outline-none focus:ring-1 focus:ring-accent',
  hover: 'hover:bg-ink/5',
}

export default interactiveStyles
