/**
 * Dockview group header action buttons: Float, Popout (Tauri window), Dock.
 *
 * Implements dockview's header action component contract:
 *   element  — the DOM node dockview mounts into the tab bar
 *   init()   — called once with group/api references
 *   dispose() — called when the group is torn down
 */

import type { DockviewApi } from 'dockview-core';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

const SVG_FLOAT =
  '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18M3 9h6"/></svg>';
const SVG_POPOUT =
  '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>';
const SVG_DOCK =
  '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 9v12"/></svg>';

export class GroupHeaderActions {
  readonly element: HTMLElement;
  private disposables: (() => void)[] = [];

  constructor() {
    this.element = document.createElement('div');
    this.element.style.cssText =
      'display:flex;align-items:center;gap:2px;padding:0 4px;height:100%;';
  }

  init(params: { containerApi: DockviewApi; api: any; group: any }) {
    const { containerApi, api } = params;

    const mkBtn = (title: string, svg: string, onClick: () => void) => {
      const btn = document.createElement('button');
      btn.title = title;
      btn.innerHTML = svg;
      btn.style.cssText =
        'background:none;border:none;color:#94a3b8;cursor:pointer;padding:2px 4px;display:flex;align-items:center;border-radius:3px;line-height:0;';
      btn.addEventListener('mouseenter', () => (btn.style.color = '#f1f5f9'));
      btn.addEventListener('mouseleave', () => (btn.style.color = '#94a3b8'));
      btn.addEventListener('click', onClick);
      return btn;
    };

    const floatBtn = mkBtn('Float panel', SVG_FLOAT, () => {
      containerApi.addFloatingGroup(params.group);
    });

    const popoutBtn = mkBtn('Open in new window', SVG_POPOUT, async () => {
      if (isTauri) {
        const panels = (params.group.panels as any[]).map((p: any) => ({
          id: p.id,
          component: p.toJSON().contentComponent ?? 'simple',
          title: p.title,
          params: p.params ?? {},
        }));
        if (!panels.length) return;

        const key = `popout-${Date.now()}`;
        localStorage.setItem(key, JSON.stringify(panels));

        const layoutSnapshot = containerApi.toJSON();

        panels.forEach((p) => {
          const panel = containerApi.getPanel(p.id);
          if (panel) containerApi.removePanel(panel);
        });

        const restore = () => {
          localStorage.removeItem(key);
          containerApi.fromJSON(layoutSnapshot);
        };

        const { invoke } = await import('@tauri-apps/api/core');
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');

        try {
          await invoke('create_popout_window', {
            label: key,
            url: `/popout?key=${key}`,
            title: panels[0]?.title ?? 'Panel',
            width: 800,
            height: 600,
          });
        } catch (err) {
          console.error('[popout] window creation failed:', err);
          restore();
          return;
        }

        const win = await WebviewWindow.getByLabel(key);
        let skipRestore = false;
        let unlistenMoveClose: (() => void) | null = null;
        try {
          const { listen } = await import('@tauri-apps/api/event');
          unlistenMoveClose = await listen<{ label: string }>(
            'dockview:closing-due-to-move',
            ({ payload }) => {
              if (payload.label === key) skipRestore = true;
            }
          );
        } catch {
          /* event plugin unavailable */
        }
        win?.once('tauri://destroyed', () => {
          unlistenMoveClose?.();
          if (!skipRestore) restore();
        });
      } else {
        // Browser: use dockview's built-in popout (injects into window.open)
        containerApi.addPopoutGroup(params.group, { popoutUrl: '/popout' });
      }
    });

    const dockBtn = mkBtn('Return to layout', SVG_DOCK, () => {
      const panels = params.group.panels as any[];
      if (!panels.length) return;
      panels.forEach((panel) => {
        containerApi.addPanel({
          id: `${panel.id}_docked`,
          component: (panel as any).toJSON().contentComponent ?? 'simple',
          title: panel.title,
          params: panel.params ?? {},
        });
        containerApi.removePanel(panel);
      });
    });

    const render = () => {
      this.element.innerHTML = '';
      const loc = api.location?.type ?? 'grid';
      if (loc === 'grid') {
        this.element.append(floatBtn, popoutBtn);
      } else if (loc === 'floating') {
        this.element.append(dockBtn, popoutBtn);
      }
    };

    render();
    const unsub = api.onDidLocationChange(() => render());
    this.disposables.push(() => unsub.dispose());
  }

  dispose() {
    this.disposables.forEach((d) => d());
    this.disposables = [];
  }
}
