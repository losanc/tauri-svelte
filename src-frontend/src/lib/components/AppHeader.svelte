<script lang="ts">
  import './AppHeader.css';

  interface Props {
    droppedFiles: string[];
    onAddPanel: () => void;
  }

  let { droppedFiles, onAddPanel, onAddNativeWgpuPanel }: Props = $props();
</script>

<header class="app-header">
  <span class="app-header-brand">Dockview + Svelte</span>

  {#if droppedFiles.length}
    <span class="app-header-dropped">Dropped: {droppedFiles.join(', ')}</span>
  {/if}

  <button class="app-header-btn" onclick={onAddNativeWgpuPanel}>+ Add Native Wgpu Panel</button>
  <button class="app-header-btn" onclick={onAddPanel}>+ Add Panel</button>
</header>
