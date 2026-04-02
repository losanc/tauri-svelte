<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createDockview } from 'dockview-core';
  import type { DockviewApi } from 'dockview-core';
  import { SimplePanel, WgpuPanel } from '$lib/panels';
  import { setupCrossWindowDnd } from '$lib/crossWindowDnd';
  import 'dockview-core/dist/styles/dockview.css';

  let container: HTMLDivElement;
  let api: DockviewApi;
  let cleanupDnd: (() => void) | null = null;
  let cleanupCursor: (() => void) | null = null;

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  // WKWebView's cursor-rect system overrides Tauri's setCursorIcon for CSS
  // cursor values it cannot handle (ew-resize, ns-resize).  For sashes we
  // bypass the cursor-rect system entirely using NSCursor.push/pop, which sits
  // above cursor rects in the macOS cursor stack.  General cursors (pointer,
  // text, …) still go through setCursorIcon because WKWebView handles those.
  async function setupNativeCursor(): Promise<() => void> {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const { invoke } = await import('@tauri-apps/api/core');
    type CursorIcon = import('@tauri-apps/api/window').CursorIcon;
    const win = getCurrentWindow();

    const CSS_TO_TAURI: Record<string, CursorIcon> = {
      auto: 'default',
      default: 'default',
      none: 'default',
      pointer: 'hand',
      crosshair: 'crosshair',
      move: 'move',
      text: 'text',
      wait: 'wait',
      help: 'help',
      progress: 'progress',
      'not-allowed': 'notAllowed',
      'context-menu': 'contextMenu',
      cell: 'cell',
      'vertical-text': 'verticalText',
      alias: 'alias',
      copy: 'copy',
      'no-drop': 'noDrop',
      grab: 'grab',
      grabbing: 'grabbing',
      'all-scroll': 'allScroll',
      'zoom-in': 'zoomIn',
      'zoom-out': 'zoomOut',
    };

    let currentIcon: CursorIcon = 'default';
    let sashPushed = false;

    const onMove = (e: MouseEvent) => {
      const target = e.target as Element | null;
      if (!target) return;

      // --- Sash resize cursors via NSCursor push/pop ---
      const sash = target.closest('.dv-sash');
      const overActiveSash = !!sash && !sash.classList.contains('dv-disabled');

      if (overActiveSash && !sashPushed) {
        sashPushed = true;
        const container = sash!.closest('.dv-split-view-container');
        const horizontal = container?.classList.contains('dv-horizontal') ?? true;
        invoke('push_resize_cursor', { horizontal });
        return;
      }
      if (!overActiveSash && sashPushed) {
        sashPushed = false;
        invoke('pop_resize_cursor');
      }
      if (sashPushed) return;

      // --- General cursors via setCursorIcon (works for pointer, text, etc.) ---
      const css = getComputedStyle(target).cursor;
      const icon: CursorIcon = CSS_TO_TAURI[css] ?? 'default';
      if (icon !== currentIcon) {
        currentIcon = icon;
        win.setCursorIcon(icon);
      }
    };

    document.addEventListener('mousemove', onMove);
    return () => {
      document.removeEventListener('mousemove', onMove);
      if (sashPushed) invoke('pop_resize_cursor');
    };
  }

  // Header action component — injected into every group's tab bar.
  // Shows float / popout / return-to-grid buttons depending on location.
  class GroupHeaderActions {
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

      const SVG_FLOAT =
        '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M9 3v18M3 9h6"/></svg>';
      const SVG_POPOUT =
        '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>';
      const SVG_DOCK =
        '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 9v12"/></svg>';

      const floatBtn = mkBtn('Float panel', SVG_FLOAT, () => {
        containerApi.addFloatingGroup(params.group);
      });

      const popoutBtn = mkBtn('Open in new window', SVG_POPOUT, async () => {
        if (isTauri) {
          // Serialize all panels in this group
          const panels = (params.group.panels as any[]).map((p: any) => ({
            id: p.id,
            component: p.toJSON().contentComponent ?? 'simple',
            title: p.title,
            params: p.params ?? {},
          }));
          if (!panels.length) return;

          const key = `popout-${Date.now()}`;
          localStorage.setItem(key, JSON.stringify(panels));

          // Snapshot the full layout so we can restore panels to their exact
          // original positions when the popout window is closed.
          const layoutSnapshot = containerApi.toJSON();

          // Remove from main layout
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

          // Listen for the window being closed to restore panels.
          // Skip restore if the window closed because its last panel was moved
          // to another window (case 3) — restore only for close (cases 1 & 2).
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

  onMount(() => {
    api = createDockview(container, {
      createComponent({ name }) {
        switch (name) {
          case 'simple':
            return new SimplePanel();
          case 'wgpu':
            return new WgpuPanel();
          default:
            throw new Error(`Unknown component: ${name}`);
        }
      },
      createRightHeaderActionComponent() {
        return new GroupHeaderActions();
      },
    });

    api.layout(container.offsetWidth, container.offsetHeight);
    cleanupDnd = setupCrossWindowDnd(api);

    if (isTauri) {
      setupNativeCursor().then((cleanup) => {
        cleanupCursor = cleanup;
      });
    }

    api.addPanel({
      id: 'explorer',
      component: 'simple',
      title: 'Explorer',
      params: { title: 'Explorer', color: '#10b981', description: 'File tree goes here' },
    });

    api.addPanel({
      id: 'editor',
      component: 'simple',
      title: 'Editor',
      params: { title: 'Editor', color: '#6366f1', description: 'Code editor goes here' },
      position: { referencePanel: 'explorer', direction: 'right' },
    });

    api.addPanel({
      id: 'terminal',
      component: 'simple',
      title: 'Terminal',
      params: { title: 'Terminal', color: '#f59e0b', description: 'Terminal output here' },
      position: { referencePanel: 'editor', direction: 'below' },
    });

    api.addPanel({
      id: 'preview',
      component: 'simple',
      title: 'Preview',
      params: { title: 'Preview', color: '#ec4899', description: 'Live preview here' },
      position: { referencePanel: 'terminal', direction: 'right' },
    });

    api.addPanel({
      id: 'viewer',
      component: 'wgpu',
      title: 'Viewer',
      position: { referencePanel: 'preview', direction: 'below' },
    });

    const ro = new ResizeObserver(() => {
      api.layout(container.offsetWidth, container.offsetHeight);
    });
    ro.observe(container);

    return () => ro.disconnect();
  });

  onDestroy(() => {
    cleanupDnd?.();
    cleanupCursor?.();
    api?.dispose();
  });

  // File drop handling — works because disable_drag_drop_handler() in Rust
  // lets the browser's native drop event through unblocked.
  let droppedFiles = $state<string[]>([]);

  function onDragOver(e: DragEvent) {
    if (e.dataTransfer?.types.includes('Files')) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'copy';
    }
  }

  function onDrop(e: DragEvent) {
    if (!e.dataTransfer?.files.length) return;
    e.preventDefault();
    droppedFiles = Array.from(e.dataTransfer.files).map((f) => f.name);
  }

  const colors = ['#3b82f6', '#8b5cf6', '#ef4444', '#14b8a6', '#f97316'];

  function addPanel() {
    const id = `panel_${Date.now()}`;
    api.addPanel({
      id,
      component: 'simple',
      title: 'New Panel',
      params: {
        title: 'New Panel',
        color: colors[Math.floor(Math.random() * colors.length)],
        description: id,
      },
    });
  }
</script>

<div class="layout" ondragover={onDragOver} ondrop={onDrop}>
  <header>
    <span class="brand">Dockview + Svelte</span>
    {#if droppedFiles.length}
      <span class="dropped">Dropped: {droppedFiles.join(', ')}</span>
    {/if}
    <button onclick={addPanel}>+ Add Panel</button>
  </header>
  <div class="dock-container" bind:this={container}></div>
</div>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  .layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0f0f0f;
    color: #f1f5f9;
    font-family: sans-serif;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    height: 40px;
    background: #1e1e2e;
    border-bottom: 1px solid #2a2a3a;
    flex-shrink: 0;
  }

  .brand {
    font-weight: 600;
    font-size: 0.9rem;
    opacity: 0.9;
  }

  .dropped {
    font-size: 0.75rem;
    color: #10b981;
    opacity: 0.9;
  }

  button {
    background: #6366f1;
    color: white;
    border: none;
    padding: 4px 12px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  button:hover {
    background: #4f46e5;
  }

  .dock-container {
    flex: 1;
    overflow: hidden;
  }

  :global(.dv-dockview) {
    background: #0f0f0f;
    --dv-group-view-background-color: transparent;
  }

  :global(.dv-tab) {
    background: #1e1e2e;
    color: #94a3b8;
    border-right: 1px solid #2a2a3a;
  }

  :global(.dv-tab.dv-active-tab) {
    background: #2a2a3a;
    color: #f1f5f9;
    border-bottom: 2px solid #6366f1;
  }

  :global(.dv-tabs-container) {
    background: #1e1e2e;
  }
</style>
