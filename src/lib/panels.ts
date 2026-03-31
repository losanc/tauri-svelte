import type { GroupPanelPartInitParameters } from 'dockview-core';
import { invoke } from '@tauri-apps/api/core';

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Mirrors the Rust GpuSurface trait: set_rect + render + dispose.
interface GpuSurface {
  setRect(x: number, y: number, width: number, height: number): void;
  render(): void;
  dispose(): void;
}

class TauriSurface implements GpuSurface {
  private rendering = false;

  setRect(x: number, y: number, width: number, height: number) {
    invoke('set_surface_rect', { x, y, width, height });
  }

  render() {
    if (this.rendering) return;
    this.rendering = true;
    invoke('render_surface').finally(() => { this.rendering = false; });
  }

  dispose() {}
}

class WasmSurface implements GpuSurface {
  private constructor(
    private readonly gpu: { set_rect(x: number, y: number, w: number, h: number): void; render(): void },
  ) {}

  setRect(x: number, y: number, width: number, height: number) {
    this.gpu.set_rect(x, y, width, height);
  }

  render() {
    this.gpu.render();
  }

  dispose() {}

  static async create(canvas: HTMLCanvasElement): Promise<WasmSurface> {
    const wasmMod = await import('$lib/wasm_gpu/wgpu_renderer.js');
    await wasmMod.default();
    const gpu = await wasmMod.WasmRenderer.create(canvas) as any;
    return new WasmSurface(gpu);
  }
}

export class WgpuPanel {
  readonly element: HTMLElement;
  private rafId: number | null = null;
  private surface: GpuSurface | null = null;

  constructor() {
    this.element = document.createElement('div');
    this.element.style.cssText = 'width:100%;height:100%;position:relative;background:transparent;';
  }

  async init(_params: GroupPanelPartInitParameters) {
    if (isTauri) {
      this.surface = new TauriSurface();
    } else {
      const canvas = document.createElement('canvas');
      canvas.style.cssText = 'width:100%;height:100%;display:block;';
      this.element.appendChild(canvas);
      this.surface = await WasmSurface.create(canvas);
    }

    let lastRect = { x: 0, y: 0, w: 0, h: 0 };
    const loop = () => {
      const r = this.element.getBoundingClientRect();
      const top = r.y + this.element.offsetTop;
      if (r.x !== lastRect.x || top !== lastRect.y || r.width !== lastRect.w || r.height !== lastRect.h) {
        lastRect = { x: r.x, y: top, w: r.width, h: r.height };
        this.surface!.setRect(r.x, top, r.width, r.height);
      }
      this.surface!.render();
      this.rafId = requestAnimationFrame(loop);
    };
    this.rafId = requestAnimationFrame(loop);
  }

  dispose() {
    if (this.rafId !== null) {
      cancelAnimationFrame(this.rafId);
      this.rafId = null;
    }
    this.surface?.dispose();
    this.surface = null;
  }
}

export class SimplePanel {
  readonly element: HTMLElement;

  constructor() {
    this.element = document.createElement('div');
    this.element.style.cssText =
      'width:100%;height:100%;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:0.5rem;font-family:sans-serif;';
  }

  init(params: GroupPanelPartInitParameters) {
    const p = params.params as any;
    this.element.innerHTML = `
      <div style="width:48px;height:48px;border-radius:8px;background:${p?.color ?? '#6366f1'};"></div>
      <strong style="font-size:1rem;">${p?.title ?? 'Panel'}</strong>
      <span style="font-size:0.8rem;opacity:0.6;">${p?.description ?? ''}</span>
    `;
  }

  dispose() {}
}
