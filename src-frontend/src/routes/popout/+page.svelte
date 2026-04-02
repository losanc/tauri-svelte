<script lang="ts">
  import { onMount } from 'svelte';
  import { createDockview } from 'dockview-core';
  import { SimplePanel, WgpuPanel } from '$lib/panels';
  import { setupCrossWindowDnd } from '$lib/crossWindowDnd';
  import '@fontsource-variable/inter';
  import 'dockview-core/dist/styles/dockview.css';

  let container: HTMLDivElement;
  let cleanupDnd: (() => void) | null = null;

  onMount(() => {
    const key = new URLSearchParams(window.location.search).get('key');
    if (!key) {
      // Browser mode: dockview injects panel content via window.opener after load.
      return;
    }

    // Tauri mode: render panels from localStorage data written by the main window.
    const stored = localStorage.getItem(key);
    if (!stored) return;

    const panels: { id: string; component: string; title: string; params: any }[] =
      JSON.parse(stored);

    const api = createDockview(container, {
      createComponent({ name }) {
        if (name === 'simple') return new SimplePanel();
        if (name === 'wgpu') return new WgpuPanel();
        throw new Error(`Unknown component: ${name}`);
      },
    });

    api.layout(container.offsetWidth, container.offsetHeight);
    panels.forEach((p) => api.addPanel(p));
    cleanupDnd = setupCrossWindowDnd(api);

    const ro = new ResizeObserver(() => api.layout(container.offsetWidth, container.offsetHeight));
    ro.observe(container);

    return () => {
      ro.disconnect();
      cleanupDnd?.();
      api.dispose();
    };
  });
</script>

<svelte:head>
  <title>Panel</title>
</svelte:head>

<div bind:this={container} class="root"></div>

<style>
  :global(html, body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    background: #0f0f0f;
    overflow: hidden;
    font-family: 'Inter Variable', sans-serif;
    color: #f1f5f9;
  }

  .root {
    width: 100%;
    height: 100vh;
  }

  :global(.dv-dockview) { background: #0f0f0f; }
  :global(.dv-tab) { background: #1e1e2e; color: #94a3b8; border-right: 1px solid #2a2a3a; }
  :global(.dv-tab.dv-active-tab) { background: #2a2a3a; color: #f1f5f9; border-bottom: 2px solid #6366f1; }
  :global(.dv-tabs-container) { background: #1e1e2e; }
</style>
