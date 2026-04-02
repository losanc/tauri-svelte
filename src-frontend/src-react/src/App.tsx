import { useEffect, useRef, useState } from 'react'
import { SimplePanel, WgpuPanel } from './panels'
import { initDockview } from './dockview/dockviewInit'
import 'dockview-core/dist/styles/dockview.css'
import './App.css'

const INITIAL_PANELS = [
  { id: 'explorer', component: 'simple', title: 'Explorer', params: { title: 'Explorer', color: '#10b981', description: 'File tree goes here' } },
  { id: 'editor', component: 'simple', title: 'Editor', params: { title: 'Editor', color: '#6366f1', description: 'Code editor goes here' }, position: { referencePanel: 'explorer', direction: 'right' as const } },
  { id: 'terminal', component: 'simple', title: 'Terminal', params: { title: 'Terminal', color: '#f59e0b', description: 'Terminal output here' }, position: { referencePanel: 'editor', direction: 'below' as const } },
  { id: 'preview', component: 'simple', title: 'Preview', params: { title: 'Preview', color: '#ec4899', description: 'Live preview here' }, position: { referencePanel: 'terminal', direction: 'right' as const } },
  { id: 'viewer', component: 'wgpu', title: 'Viewer', position: { referencePanel: 'preview', direction: 'below' as const } },
]

const COLORS = ['#3b82f6', '#8b5cf6', '#ef4444', '#14b8a6', '#f97316']

export default function App() {
  const containerRef = useRef<HTMLDivElement>(null)
  const handleRef = useRef<ReturnType<typeof initDockview> | null>(null)
  const [droppedFiles, setDroppedFiles] = useState<string[]>([])

  useEffect(() => {
    if (!containerRef.current) return
    handleRef.current = initDockview(containerRef.current, {
      createComponent({ name }) {
        switch (name) {
          case 'simple': return new SimplePanel()
          case 'wgpu': return new WgpuPanel()
          default: throw new Error(`Unknown component: ${name}`)
        }
      },
      initialPanels: INITIAL_PANELS,
    })
    return () => { handleRef.current?.destroy(); handleRef.current = null }
  }, [])

  function addPanel() {
    if (!handleRef.current) return
    const id = `panel_${Date.now()}`
    handleRef.current.api.addPanel({
      id,
      component: 'simple',
      title: 'New Panel',
      params: { title: 'New Panel', color: COLORS[Math.floor(Math.random() * COLORS.length)], description: id },
    })
  }

  function onDragOver(e: React.DragEvent) {
    if (e.dataTransfer.types.includes('Files')) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy' }
  }

  function onDrop(e: React.DragEvent) {
    if (!e.dataTransfer.files.length) return
    e.preventDefault()
    setDroppedFiles(Array.from(e.dataTransfer.files).map(f => f.name))
  }

  return (
    <div className="layout" onDragOver={onDragOver} onDrop={onDrop}>
      <header>
        <span className="brand">Dockview + React</span>
        {droppedFiles.length > 0 && <span className="dropped">Dropped: {droppedFiles.join(', ')}</span>}
        <button onClick={addPanel}>+ Add Panel</button>
      </header>
      <div className="dock-container" ref={containerRef} />
    </div>
  )
}
