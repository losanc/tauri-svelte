import { Component, AfterViewInit, OnDestroy, ElementRef, ViewChild } from '@angular/core'
import { SimplePanel, WgpuPanel } from '../panels'
import { initPopoutDockview } from '../dockview/dockviewPopout'
import type { DockviewHandle } from '../dockview/dockviewInit'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [],
  template: `<div #container class="dock-container" style="height:100vh"></div>`,
})
export class Popout implements AfterViewInit, OnDestroy {
  @ViewChild('container') containerRef!: ElementRef<HTMLDivElement>
  private handle: DockviewHandle | null = null

  ngAfterViewInit() {
    this.handle = initPopoutDockview(this.containerRef.nativeElement, {
      createComponent({ name }) {
        switch (name) {
          case 'simple': return new SimplePanel()
          case 'wgpu': return new WgpuPanel()
          default: throw new Error(`Unknown component: ${name}`)
        }
      },
    })
  }

  ngOnDestroy() {
    this.handle?.destroy()
    this.handle = null
  }
}
