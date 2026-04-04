import type { GroupPanelPartInitParameters } from 'dockview-core';
import { mount, unmount } from 'svelte';
import Page from './Page.svelte';

/**
 * IContentRenderer bridge for the file browser panel.
 * Mounts the Page component imperatively when initialized,
 * and cleans up on dispose.
 */
export class FileBrowserPanel {
  readonly element: HTMLElement;
  private mounted: Record<string, unknown> | null = null;

  constructor() {
    this.element = document.createElement('div');
    this.element.style.cssText = 'width:100%;height:100%;display:flex;flex-direction:column;overflow:hidden;';
  }

  init(_params: GroupPanelPartInitParameters): void {
    this.mounted = mount(Page, { target: this.element });
  }

  dispose(): void {
    if (this.mounted) {
      unmount(this.mounted);
      this.mounted = null;
    }
  }
}
