/**
 * Cross-window panel drag-and-drop for Tauri.
 *
 * Hooks into dockview's own event pipeline rather than reimplementing it:
 *
 * - onWillDragPanel     → stamp panel data onto the native DataTransfer so other
 *                         windows can read it across the OS drag boundary.
 * - onUnhandledDragOverEvent → fires only when LocalSelectionTransfer has no data
 *                         (i.e. always for cross-window drags). Calling accept()
 *                         makes dockview render its own drop-zone overlay.
 * - onWillDrop          → fires synchronously from the native drop event for every
 *                         drop. We intercept cross-window drops, call preventDefault
 *                         to suppress dockview's internal move, then addPanel using
 *                         dockview's own computed group + position.
 * - Tauri event bus     → tells the source window to remove the panel once it has
 *                         been successfully received by the target window.
 */

import type { DockviewApi } from 'dockview-core';
import { positionToDirection } from 'dockview-core';

const DRAG_TYPE = 'application/x-dockview-panel';
const MOVE_EVENT = 'dockview-panel-moved';

interface PanelPayload {
  id: string;
  component: string;
  title: string;
  params: Record<string, unknown>;
  sourceLabel: string;
}

export function setupCrossWindowDnd(api: DockviewApi): () => void {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    return () => {};
  }

  let myLabel = '';
  let unlisten: (() => void) | null = null;
  let isRemovingDueToMove = false;

  (async () => {
    try {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      myLabel = getCurrentWebviewWindow().label;
    } catch { /* non-Tauri */ }
  })();

  // ── Source side ────────────────────────────────────────────────────────────
  // Stamp the panel's serialised data onto the native drag event so the OS
  // carries it into whichever window the user drops onto.

  const sub1 = api.onWillDragPanel((event) => {
    if (!event.nativeEvent.dataTransfer) return;
    const payload: PanelPayload = {
      id: event.panel.id,
      component: event.panel.toJSON().contentComponent ?? 'simple',
      title: event.panel.title ?? 'Panel',
      params: (event.panel.params as Record<string, unknown>) ?? {},
      sourceLabel: myLabel,
    };
    event.nativeEvent.dataTransfer.setData(DRAG_TYPE, JSON.stringify(payload));
  });

  // Remove a panel that was successfully landed in another window.
  (async () => {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlisten = await listen<{ panelId: string; sourceLabel: string }>(
        MOVE_EVENT,
        async ({ payload }) => {
          if (payload.sourceLabel !== myLabel) return;
          isRemovingDueToMove = true;
          const panel = api.getPanel(payload.panelId);
          if (panel) api.removePanel(panel);
          isRemovingDueToMove = false;
          if (api.panels.length === 0) {
            const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
            const win = getCurrentWebviewWindow();
            const { emit } = await import('@tauri-apps/api/event');
            await emit('dockview:closing-due-to-move', { label: myLabel });
            win.close();
          }
        },
      );
    } catch { /* event plugin unavailable */ }
  })();

  // ── Target side ────────────────────────────────────────────────────────────
  // onUnhandledDragOverEvent fires ONLY when LocalSelectionTransfer has no data,
  // which is always the case for drags originating in a different window.
  // Accepting the event tells dockview to render its own drop-zone overlay.

  const sub2 = api.onUnhandledDragOverEvent((event) => {
    if (event.nativeEvent.dataTransfer?.types.includes(DRAG_TYPE)) {
      event.accept();
    }
  });

  // onWillDrop fires synchronously from the native drop handler for every drop.
  // We read the payload (dataTransfer.getData is safe here), block dockview's
  // default handling, and re-add the panel using dockview's computed position.

  const sub3 = api.onWillDrop((event) => {
    const raw = event.nativeEvent.dataTransfer?.getData(DRAG_TYPE);
    if (!raw) return;

    let payload: PanelPayload;
    try { payload = JSON.parse(raw); } catch { return; }

    // Intra-window drag — let dockview handle it as normal.
    if (payload.sourceLabel === myLabel) return;

    event.preventDefault();

    try {
      if (event.group) {
        api.addPanel({
          id: payload.id,
          component: payload.component,
          title: payload.title,
          params: payload.params,
          position: {
            referenceGroup: event.group,
            direction: positionToDirection(event.position),
          },
        });
      } else {
        // Dropped outside any group (root edge) — add as floating near cursor.
        api.addPanel({
          id: payload.id,
          component: payload.component,
          title: payload.title,
          params: payload.params,
          floating: {
            position: {
              left: Math.max(0, event.nativeEvent.clientX - 200),
              top: Math.max(0, event.nativeEvent.clientY - 150),
            },
            width: 400,
            height: 300,
          },
        });
      }
    } catch {
      // ID collision — mint a fresh id and add as floating.
      api.addPanel({
        id: `${payload.id}_${Date.now()}`,
        component: payload.component,
        title: payload.title,
        params: payload.params,
        floating: {
          position: {
            left: Math.max(0, event.nativeEvent.clientX - 200),
            top: Math.max(0, event.nativeEvent.clientY - 150),
          },
          width: 400,
          height: 300,
        },
      });
    }

    (async () => {
      try {
        const { emit } = await import('@tauri-apps/api/event');
        await emit(MOVE_EVENT, { panelId: payload.id, sourceLabel: payload.sourceLabel });
      } catch { /* best-effort */ }
    })();
  });

  // Close popout window when the last panel is removed by the user (case 1).
  // Skipped when the removal is driven by a cross-window move (case 3).
  const sub4 = api.onDidRemovePanel(async () => {
    if (isRemovingDueToMove) return;
    if (api.panels.length === 0 && myLabel.startsWith('popout-')) {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      getCurrentWebviewWindow().close();
    }
  });

  return () => {
    sub1.dispose();
    sub2.dispose();
    sub3.dispose();
    sub4.dispose();
    unlisten?.();
  };
}
