import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { ZKPlexProvider } from './contexts/ZKPlexContext'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ZKPlexProvider>
      <App />
    </ZKPlexProvider>
  </StrictMode>,
)