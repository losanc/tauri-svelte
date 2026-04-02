/**
 * Framework-agnostic dockview initialisation for popout windows.
 *
 * Reads the `?key=` query parameter and reconstructs panels from localStorage
 * data written by the main window. In browser mode (no `?key=`), dockview
 * injects panel content via window.opener after load — return null and do nothing.
 *
 * Does NOT set up GroupHeaderActions or native cursor — popout windows are
 * intentionally simpler than the main window.
 *
 * Usage (any framework):
 *
 *   const handle = initPopoutDockview(containerEl, { createComponent });
 *   if (handle) {
 *     // on unmount:
 *     handle.destroy();
 *   }
 */

import { createDockview } from 'dockview-core';
import type { DockviewApi, IContentRenderer } from 'dockview-core';
import { setupCrossWindowDnd } from './crossWindowDnd';
import type { DockviewHandle } from './dockviewInit';

export interface PopoutInitOptions {
  createComponent: (opts: { name: string }) => IContentRenderer;
}

export function initPopoutDockview(
  container: HTMLElement,
  options: PopoutInitOptions,
): DockviewHandle | null {
  const key = new URLSearchParams(window.location.search).get('key');
  if (!key) {
    // Browser mode: dockview injects panel content via window.opener after load.
    return null;
  }

  const stored = localStorage.getItem(key);
  if (!stored) return null;

  const panels: { id: string; component: string; title: string; params: any }[] =
    JSON.parse(stored);

  const api: DockviewApi = createDockview(container, {
    createComponent: options.createComponent,
  });

  api.layout(container.offsetWidth, container.offsetHeight);
  panels.forEach((p) => api.addPanel(p));

  const cleanupDnd = setupCrossWindowDnd(api);

  const ro = new ResizeObserver(() => api.layout(container.offsetWidth, container.offsetHeight));
  ro.observe(container);

  return {
    api,
    destroy: () => {
      ro.disconnect();
      cleanupDnd();
      api.dispose();
    },
  };
}
