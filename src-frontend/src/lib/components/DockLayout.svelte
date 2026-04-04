<script lang="ts">
  import { createDockview, themeAbyss } from 'dockview-core';
  import 'dockview-core/dist/styles/dockview.css';
  import { onDestroy } from 'svelte';

  let container: HTMLDivElement;
  let api: ReturnType<typeof createDockview>;

  $effect(() => {
    api = createDockview(container, {
      theme: themeAbyss,
      createComponent({ name }) {
        const el = document.createElement('div');
        el.textContent = name;
        return { element: el, init: () => {}, dispose: () => {} };
      },
    });

    // Add a starter panel
    api.addPanel({
      id: 'panel1',
      component: 'default',
      title: 'Explorer',
    });

    return () => api.dispose();
  });
</script>

<div bind:this={container} style="width: 100%; height: 100%;"></div>
