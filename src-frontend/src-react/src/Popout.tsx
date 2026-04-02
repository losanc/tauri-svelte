import { useEffect, useRef } from 'react'
import { SimplePanel, WgpuPanel } from './panels'
import { initPopoutDockview } from './dockview/dockviewPopout'
import 'dockview-core/dist/styles/dockview.css'
import './App.css'

export default function Popout() {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!containerRef.current) return
    const handle = initPopoutDockview(containerRef.current, {
      createComponent({ name }) {
        switch (name) {
          case 'simple': return new SimplePanel()
          case 'wgpu': return new WgpuPanel()
          default: throw new Error(`Unknown component: ${name}`)
        }
      },
    })
    return () => { handle?.destroy() }
  }, [])

  return <div className="dock-container" style={{ height: '100vh' }} ref={containerRef} />
}
