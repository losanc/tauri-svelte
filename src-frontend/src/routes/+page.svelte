<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { initDockview, WgpuPanel, SimplePanel, NativeWgpuPanel } from '$lib/dockview';
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
          case 'nativeWgpu':
            return new NativeWgpuPanel();
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
    const panel = handle.api.addPanel({
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

  function addNativeWgpuPanel() {
    const id = `panel_${Date.now()}`;
    const panel = handle.api.addPanel({
      id,
      component: 'nativeWgpu',
      title: 'New Panel',
      params: {
        title: 'New Panel',
        color: PANEL_COLORS[Math.floor(Math.random() * PANEL_COLORS.length)],
        description: id,
      },
    });

    const viewPanel = panel.view.content as NativeWgpuPanel;
    const panelViewElement = panel.view.content.element;

    function getSurfaceRect(el: HTMLElement) {
      const rect = el.getBoundingClientRect();
      return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
    }

    const init = getSurfaceRect(panelViewElement);
    viewPanel.create_native_wgpu_surface(init.width, init.height, init.x, init.y);

    panel.api.onDidVisibilityChange((e) => {
      if (e.isVisible) {
        viewPanel.display();
      } else {
        viewPanel.hide();
      }
    });

    const updateRect = () => {
      // Update with a frame delay to ensure the DOM has updated with the new panel size/position.
      requestAnimationFrame(() => {
        const rect = getSurfaceRect(panelViewElement);
        viewPanel.move_surface(rect.width, rect.height, rect.x, rect.y);
      });
    };
    
    panel.api.onDidDimensionsChange(updateRect);
    panel.api.onDidLocationChange(updateRect);
  }

</script>

<div
  class="app-layout"
  role="region"
  aria-label="Main layout"
  ondragover={fileDrop.onDragOver}
  ondrop={fileDrop.onDrop}
>
  <AppHeader droppedFiles={fileDrop.files} onAddPanel={addPanel} onAddNativeWgpuPanel = {addNativeWgpuPanel} />
  <div class="app-dock-container" bind:this={container}></div>
</div>
