/**
 * Keyboard navigation hook for the file browser.
 * Handles arrow keys, Enter, and Escape for column and entry navigation.
 */

interface DirEntry {
  name: string;
  path: string;
}

interface Column {
  path: string;
  entries: DirEntry[];
  selectedPath: string | null;
  loading: boolean;
}

export interface KeyboardNavOptions {
  getColumns: () => Column[];
  getFocusedColumn: () => number;
  setFocusedColumn: (i: number) => void;
  selectEntry: (colIndex: number, path: string) => Promise<void>;
  navigateRight: () => void;
  navigateLeft: () => void;
}

export function useKeyboardNav(opts: KeyboardNavOptions) {
  function onKeydown(e: KeyboardEvent): void {
    const cols = opts.getColumns();
    const fi = opts.getFocusedColumn();
    const col = cols[fi];
    if (!col) return;

    switch (e.key) {
      case 'ArrowUp': {
        e.preventDefault();
        const idx = col.entries.findIndex((en) => en.path === col.selectedPath);
        const next = Math.max(0, idx - 1);
        const nextPath = col.entries[next]?.path ?? col.entries[0]?.path;
        if (nextPath) {
          opts.selectEntry(fi, nextPath);
        }
        break;
      }

      case 'ArrowDown': {
        e.preventDefault();
        const idx = col.entries.findIndex((en) => en.path === col.selectedPath);
        const next = Math.min(col.entries.length - 1, idx + 1);
        const nextPath = col.entries[next]?.path ?? col.entries[0]?.path;
        if (nextPath) {
          opts.selectEntry(fi, nextPath);
        }
        break;
      }

      case 'ArrowRight': {
        e.preventDefault();
        const idx = col.entries.findIndex((en) => en.path === col.selectedPath);
        const nextPath = col.entries[idx]?.path ?? col.entries[0]?.path;
        if (nextPath) {
          opts.selectEntry(fi, nextPath);
        }
        opts.navigateRight();
        break;
      }
      case 'Enter': {
        e.preventDefault();
        opts.navigateRight();
        break;
      }

      case 'ArrowLeft':
      case 'Escape': {
        e.preventDefault();
        opts.navigateLeft();
        break;
      }
    }
  }

  return { onKeydown };
}
