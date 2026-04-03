<script lang="ts">
  import './filebrowser.css';

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

  interface Props {
    columns: Column[];
    focusedColumn: number;
    onSelect: (colIndex: number, path: string) => void;
  }

  let { columns, focusedColumn, onSelect }: Props = $props();

  let columnsEl: HTMLElement;

  $effect(() => {
    const col = columnsEl?.children[focusedColumn] as HTMLElement | undefined;
    col?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
  });
</script>

<div class="filebrowser-columns" bind:this={columnsEl}>
  {#each columns as col, colIndex (col.path)}
    <div class="filebrowser-column" class:focused={colIndex === focusedColumn}>
      {#if col.loading}
        <div class="filebrowser-loading">
          <span class="filebrowser-spinner"></span>
        </div>
      {/if}

      {#each col.entries as entry (entry.path)}
        <button
          class="filebrowser-entry"
          class:selected={entry.path === col.selectedPath}
          onclick={() => onSelect(colIndex, entry.path)}
          type="button"
        >
          <span class="filebrowser-entry-icon">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20 6h-8l-2-2H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2z" />
            </svg>
          </span>
          <span class="filebrowser-entry-name">{entry.name}</span>
          {#if entry.path === col.selectedPath && columns[colIndex + 1]}
            <span class="filebrowser-entry-chevron">›</span>
          {/if}
        </button>
      {/each}

      {#if !col.loading && col.entries.length === 0}
        <div class="filebrowser-empty">No subdirectories</div>
      {/if}
    </div>
  {/each}
</div>
