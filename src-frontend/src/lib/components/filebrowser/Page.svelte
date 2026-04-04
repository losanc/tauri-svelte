<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Layout from './Layout.svelte';
  import { useKeyboardNav } from './keyboard.svelte';
  import './filebrowser.css';

  const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  // ── Types ──────────────────────────────────────────
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

  // ── State ──────────────────────────────────────────
  let columns = $state<Column[]>([]);
  let focusedColumn = $state(0);

  // ── State machine functions ────────────────────────

  async function loadColumn(path: string): Promise<DirEntry[]> {
    if (!isTauri) return [];
    try {
      return await invoke<DirEntry[]>('list_directory', { path });
    } catch {
      return [];
    }
  }

  async function selectEntry(colIndex: number, entryPath: string): Promise<void> {
    // Update selection and show loading column to the right
    columns = [
      ...columns.slice(0, colIndex + 1).map((col, i) =>
        i === colIndex ? { ...col, selectedPath: entryPath } : col
      ),
      { path: entryPath, entries: [], selectedPath: null, loading: true },
    ];
    focusedColumn = colIndex;

    const entries = await loadColumn(entryPath);

    // Guard against stale results if selection changed while loading
    if (columns[colIndex]?.selectedPath === entryPath) {
      columns = [
        ...columns.slice(0, colIndex + 1),
        { path: entryPath, entries, selectedPath: null, loading: false },
      ];
    }
  }

  function navigateRight(): void {
    if (columns[focusedColumn]?.selectedPath && columns[focusedColumn + 1]) {
      focusedColumn = focusedColumn + 1;
    }
  }

  function navigateLeft(): void {
    if (focusedColumn === 0) return;
    const targetIndex = focusedColumn - 1;
    focusedColumn = targetIndex;

    const col = columns[targetIndex];
    if (col && !col.selectedPath && col.entries.length > 0) {
      selectEntry(targetIndex, col.entries[0].path);
    }
  }

  // ── Keyboard ───────────────────────────────────────
  const { onKeydown } = useKeyboardNav({
    getColumns: () => columns,
    getFocusedColumn: () => focusedColumn,
    setFocusedColumn: (i) => {
      focusedColumn = i;
    },
    selectEntry,
    navigateRight,
    navigateLeft,
  });

  // ── Initial load ───────────────────────────────────
  $effect(() => {
    if (!isTauri) {
      columns = [{ path: '/', entries: [], selectedPath: null, loading: false }];
      return;
    }

    (async () => {
      try {
        const home = await invoke<string>('get_home_dir');
        const entries = await loadColumn(home);
        columns = [{ path: home, entries, selectedPath: null, loading: false }];
      } catch {
        columns = [{ path: '/', entries: [], selectedPath: null, loading: false }];
      }
    })();
  });
</script>

<div
  class="filebrowser-root"
  tabindex="0"
  role="tree"
  aria-label="File browser"
  onkeydown={onKeydown}
>
  <Layout {columns} {focusedColumn} onSelect={selectEntry} />
</div>
