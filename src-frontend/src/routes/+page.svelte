<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { initDockview, WgpuPanel, SimplePanel } from '$lib/dockview';
  import type { DockviewHandle } from '$lib/dockview';
  import { initialPanels, PANEL_COLORS } from '$lib/config/panels';
  import { useFileDrop } from '$lib/hooks/useFileDrop.svelte';
  import AppHeader from '$lib/components/AppHeader.svelte';
  import { FileBrowserPanel } from '$lib/components/filebrowser/FileBrowserPanel';
  import './page.css';
  import 'dockview-core/dist/styles/dockview.css';

  let container: HTMLDivElement;
  let handle: DockviewHandle;
  const fileDrop = useFileDrop();

  onMount(() => {
    handle = initDockview(container, {
      createComponent({ name }) {
        switch (name) {
          case 'simple':
            return new SimplePanel();
          case 'wgpu':
            return new WgpuPanel();
          case 'filebrowser':
            return new FileBrowserPanel();
          default:
            throw new Error(`Unknown component: ${name}`);
        }
      },
      initialPanels,
    });
  });

  onDestroy(() => handle?.destroy());

  function addPanel() {
    const id = `panel_${Date.now()}`;
    handle.api.addPanel({
      id,
      component: 'simple',
      title: 'New Panel',
      params: {
        title: 'New Panel',
        color: PANEL_COLORS[Math.floor(Math.random() * PANEL_COLORS.length)],
        description: id,
      },
    });
  }
</script>

<div
  class="app-layout"
  role="region"
  aria-label="Main layout"
  ondragover={fileDrop.onDragOver}
  ondrop={fileDrop.onDrop}
>
  <AppHeader droppedFiles={fileDrop.files} onAddPanel={addPanel} />
  <div class="app-dock-container" bind:this={container}></div>
</div>
