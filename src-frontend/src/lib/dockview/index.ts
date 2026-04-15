/**
 * Dockview Tauri integration — framework-agnostic, reusable modules.
 *
 * Exports all public APIs for setting up dockview in Tauri (native) and
 * browser (WebGPU) contexts. Can be used from any JavaScript framework.
 *
 * Core orchestration:
 *   - initDockview(container, opts) → DockviewHandle
 *   - initPopoutDockview(container, opts) → DockviewHandle | null
 *
 * Panel implementations:
 *   - WgpuPanel — GPU-rendered via wgpu (native) or WebGPU (browser)
 *   - SimplePanel — fallback colored box + title
 *
 * Platform features:
 *   - setupCrossWindowDnd(api) — cross-window panel drag-and-drop (Tauri only)
 *   - setupNativeCursor() — macOS NSCursor push/pop for resize handles
 *   - GroupHeaderActions — float/popout/dock buttons
 */

export { setupCrossWindowDnd } from './crossWindowDnd';
export { WgpuPanel, SimplePanel, NativeWgpuPanel, tabBarHeight } from './panels';
export { setupNativeCursor } from './nativeCursor';
export { GroupHeaderActions } from './groupHeaderActions';
export {
  initDockview,
  type DockviewHandle,
  type DockviewInitOptions,
  type PanelDescriptor,
} from './dockviewInit';
export { initPopoutDockview, type PopoutInitOptions } from './dockviewPopout';
