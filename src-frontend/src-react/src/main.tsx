import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import Popout from './Popout.tsx'

const isPopout = window.location.pathname.startsWith('/popout')

createRoot(document.getElementById('root')!).render(
  isPopout ? <Popout /> : <App />
)
