/**
 * Framework-agnostic dockview initialisation for the main window.
 *
 * Sets up:
 *   - dockview (createDockview)
 *   - GroupHeaderActions on every group header
 *   - ResizeObserver to keep layout in sync with the container
 *   - Cross-window DnD (Tauri only, no-op in browser)
 *   - Native macOS cursor handling (Tauri only, no-op in browser)
 *
 * Usage (any framework):
 *
 *   const handle = initDockview(containerEl, { createComponent, initialPanels });
 *   // on unmount:
 *   handle.destroy();
 */

import { createDockview } from 'dockview-core';
import type { DockviewApi, IContentRenderer } from 'dockview-core';
import { setupCrossWindowDnd } from './crossWindowDnd';
import { setupNativeCursor } from './nativeCursor';
import { GroupHeaderActions } from './groupHeaderActions';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export interface PanelDescriptor {
  id: string;
  component: string;
  title: string;
  params?: Record<string, unknown>;
  position?: {
    referencePanel: string;
    direction: 'left' | 'right' | 'above' | 'below' | 'within';
  };
}

export interface DockviewInitOptions {
  /** Factory matching dockview's createComponent signature. */
  createComponent: (opts: { name: string }) => IContentRenderer;
  /** Panels to add immediately after the instance is created. */
  initialPanels?: PanelDescriptor[];
}

export interface DockviewHandle {
  api: DockviewApi;
  /** Tear down dockview, observers, and all Tauri subscriptions. */
  destroy: () => void;
}

export function initDockview(container: HTMLElement, options: DockviewInitOptions): DockviewHandle {
  const api = createDockview(container, {
    createComponent: options.createComponent,
    createRightHeaderActionComponent: () => new GroupHeaderActions(),
  });

  api.layout(container.offsetWidth, container.offsetHeight);
  options.initialPanels?.forEach((p) => api.addPanel(p));

  const cleanupDnd = setupCrossWindowDnd(api);

  let cleanupCursor: (() => void) | null = null;
  if (isTauri) {
    setupNativeCursor().then((fn) => {
      cleanupCursor = fn;
    });
  }

  const ro = new ResizeObserver(() => api.layout(container.offsetWidth, container.offsetHeight));
  ro.observe(container);

  return {
    api,
    destroy: () => {
      ro.disconnect();
      cleanupDnd();
      cleanupCursor?.();
      api.dispose();
    },
  };
}
