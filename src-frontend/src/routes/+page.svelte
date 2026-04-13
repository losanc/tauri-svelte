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

    const mypanel = panel.view.content;
    const initialboundingbox = panel.view.content.element.getBoundingClientRect();
    const width =initialboundingbox.width;
    const height =initialboundingbox.height;
    const x =initialboundingbox.x;
    const y =initialboundingbox.y;
    mypanel.create_native_wgpu_surface(width, height, x,y);


    panel.api.onDidVisibilityChange((e) => {

    if (e.isVisible) {
        mypanel.display();
    } else {
        mypanel.hide();
    }});
    panel.api.onDidDimensionsChange((e) => {
        const newlocation = panel.view.content.element.getBoundingClientRect();
        const width =newlocation.width;
        const height =newlocation.height;
        const x =newlocation.x;
        const y =newlocation.y;
        console.log(newlocation);
        mypanel.move_surface(width, height, x,y);
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
  <AppHeader droppedFiles={fileDrop.files} onAddPanel={addPanel} onAddNativeWgpuPanel = {addNativeWgpuPanel} />
  <div class="app-dock-container" bind:this={container}></div>
</div>
