<script lang="ts">
  import { onMount } from 'svelte';
  import { initPopoutDockview, WgpuPanel, SimplePanel } from '$lib/dockview';
  import './page.css';
  import 'dockview-core/dist/styles/dockview.css';

  let container: HTMLDivElement;

  onMount(() => {
    const handle = initPopoutDockview(container, {
      createComponent({ name }) {
        if (name === 'simple') return new SimplePanel();
        if (name === 'wgpu') return new WgpuPanel();
        throw new Error(`Unknown component: ${name}`);
      },
    });

    return () => handle?.destroy();
  });
</script>

<svelte:head>
  <title>Panel</title>
</svelte:head>

<div bind:this={container} class="popout-root"></div>
