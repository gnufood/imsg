import '@/index.css'

import App from '@/App.tsx'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

const container = document.querySelector('#root')
if (container === null) {
  throw new Error('#root element not found')
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
