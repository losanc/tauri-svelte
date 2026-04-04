import type { PanelDescriptor } from '$lib/dockview';

export const PANEL_COLORS = ['#3b82f6', '#8b5cf6', '#ef4444', '#14b8a6', '#f97316'];

export const initialPanels: PanelDescriptor[] = [
  {
    id: 'explorer',
    component: 'simple',
    title: 'Explorer',
    params: { title: 'Explorer', color: '#10b981', description: 'File tree goes here' },
  },
  {
    id: 'filebrowser',
    component: 'filebrowser',
    title: 'Files',
    position: { referencePanel: 'explorer', direction: 'within' },
  },
  {
    id: 'editor',
    component: 'simple',
    title: 'Editor',
    params: { title: 'Editor', color: '#6366f1', description: 'Code editor goes here' },
    position: { referencePanel: 'explorer', direction: 'right' },
  },
  {
    id: 'terminal',
    component: 'simple',
    title: 'Terminal',
    params: { title: 'Terminal', color: '#f59e0b', description: 'Terminal output here' },
    position: { referencePanel: 'editor', direction: 'below' },
  },
  {
    id: 'preview',
    component: 'simple',
    title: 'Preview',
    params: { title: 'Preview', color: '#ec4899', description: 'Live preview here' },
    position: { referencePanel: 'terminal', direction: 'right' },
  },
  {
    id: 'viewer',
    component: 'wgpu',
    title: 'Viewer',
    position: { referencePanel: 'preview', direction: 'below' },
  },
];
