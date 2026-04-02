import { Component, AfterViewInit, OnDestroy, ElementRef, ViewChild } from '@angular/core'
import { SimplePanel, WgpuPanel } from '../panels'
import { initDockview } from '../dockview/dockviewInit'
import type { DockviewHandle } from '../dockview/dockviewInit'

const COLORS = ['#3b82f6', '#8b5cf6', '#ef4444', '#14b8a6', '#f97316']

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [],
  templateUrl: './app.html',
  styleUrl: './app.css',
})
export class App implements AfterViewInit, OnDestroy {
  @ViewChild('container') containerRef!: ElementRef<HTMLDivElement>
  private handle: DockviewHandle | null = null

  ngAfterViewInit() {
    this.handle = initDockview(this.containerRef.nativeElement, {
      createComponent({ name }) {
        switch (name) {
          case 'simple': return new SimplePanel()
          case 'wgpu': return new WgpuPanel()
          default: throw new Error(`Unknown component: ${name}`)
        }
      },
      initialPanels: [
        { id: 'explorer', component: 'simple', title: 'Explorer', params: { title: 'Explorer', color: '#10b981', description: 'File tree goes here' } },
        { id: 'editor', component: 'simple', title: 'Editor', params: { title: 'Editor', color: '#6366f1', description: 'Code editor goes here' }, position: { referencePanel: 'explorer', direction: 'right' } },
        { id: 'terminal', component: 'simple', title: 'Terminal', params: { title: 'Terminal', color: '#f59e0b', description: 'Terminal output here' }, position: { referencePanel: 'editor', direction: 'below' } },
        { id: 'preview', component: 'simple', title: 'Preview', params: { title: 'Preview', color: '#ec4899', description: 'Live preview here' }, position: { referencePanel: 'terminal', direction: 'right' } },
        { id: 'viewer', component: 'wgpu', title: 'Viewer', position: { referencePanel: 'preview', direction: 'below' } },
      ],
    })
  }

  ngOnDestroy() {
    this.handle?.destroy()
    this.handle = null
  }

  addPanel() {
    if (!this.handle) return
    const id = `panel_${Date.now()}`
    this.handle.api.addPanel({
      id,
      component: 'simple',
      title: 'New Panel',
      params: { title: 'New Panel', color: COLORS[Math.floor(Math.random() * COLORS.length)], description: id },
    })
  }
}
