/**
 * Tracks files dropped onto the window.
 *
 * Works because disable_drag_drop_handler() in Rust lets the browser's native
 * drop event through unblocked.
 */
export function useFileDrop() {
  let files = $state<string[]>([]);

  function onDragOver(e: DragEvent) {
    if (e.dataTransfer?.types.includes('Files')) {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'copy';
    }
  }

  function onDrop(e: DragEvent) {
    if (!e.dataTransfer?.files.length) return;
    e.preventDefault();
    files = Array.from(e.dataTransfer.files).map((f) => f.name);
  }

  return {
    get files() {
      return files;
    },
    onDragOver,
    onDrop,
  };
}
