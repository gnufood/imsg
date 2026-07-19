interface InteractiveStyles {
  disabled: string
  disabledContainer: string
  disabledHoverReset: string
  focus: string
  hasFocus: string
  hover: string
  peerFocus: string
}

// Single source for the hover/focus/disabled Tailwind classes shared by every clickable/
// Focusable atom and list-item molecule — `Button`/`IconButton`/list rows reuse `hover`+`focus`,
// `TextInput` reuses `focus` only. `disabledContainer`/`peerFocus` are for `Checkbox`, whose
// Native `<input>` is visually hidden (see `Checkbox.tsx`) — its disabled/focus state has to
// Reach the sibling icon via `:has()`/`peer-focus` instead of styling the input directly.
// `hasFocus` is for `SegmentedControl`, where the hidden input is a *child* of the element
// Needing the ring rather than a sibling — `has-[:focus-visible]` reaches into it instead.
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
