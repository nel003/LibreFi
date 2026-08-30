import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { toast } from '#components/ui/toast'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
    <toast.Toaster />
  </StrictMode>,
)
