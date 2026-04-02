/**
 * Native macOS cursor handling for Tauri.
 *
 * WKWebView's cursor-rect system overrides Tauri's setCursorIcon for CSS
 * cursor values it cannot handle (ew-resize, ns-resize). For sashes we
 * bypass the cursor-rect system entirely using NSCursor.push/pop, which sits
 * above cursor rects in the macOS cursor stack. General cursors (pointer,
 * text, …) still go through setCursorIcon because WKWebView handles those.
 */

export async function setupNativeCursor(): Promise<() => void> {
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  const { invoke } = await import('@tauri-apps/api/core');
  type CursorIcon = import('@tauri-apps/api/window').CursorIcon;
  const win = getCurrentWindow();

  const CSS_TO_TAURI: Record<string, CursorIcon> = {
    'auto':          'default',
    'default':       'default',
    'none':          'default',
    'pointer':       'hand',
    'crosshair':     'crosshair',
    'move':          'move',
    'text':          'text',
    'wait':          'wait',
    'help':          'help',
    'progress':      'progress',
    'not-allowed':   'notAllowed',
    'context-menu':  'contextMenu',
    'cell':          'cell',
    'vertical-text': 'verticalText',
    'alias':         'alias',
    'copy':          'copy',
    'no-drop':       'noDrop',
    'grab':          'grab',
    'grabbing':      'grabbing',
    'all-scroll':    'allScroll',
    'zoom-in':       'zoomIn',
    'zoom-out':      'zoomOut',
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
