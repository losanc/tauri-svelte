# Tauri + SvelteKit + TypeScript

Desktop app using Tauri 2, SvelteKit 5, and GPU-rendered panels via wgpu/WebGPU. 

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Development

```bash
pnpm tauri dev      # Native desktop app (Vite + Tauri window)
pnpm dev            # Web frontend only (Vite dev server, port 1420)
pnpm preview        # Preview static production build in browser
```

## Build

```bash
pnpm build:wasm     # Compile Rust → WASM (required after any src-backend/ changes)
pnpm build          # Full build: WASM + SvelteKit static output
```

> WASM output lands in `src-frontend/src/lib/wasm_gpu/`.

## Type Checking

```bash
pnpm check          # SvelteKit sync + TypeScript/Svelte type-check
pnpm check:watch    # Same, in watch mode
```

## Formatting

```bash
pnpm format         # Prettier — frontend TypeScript, Svelte, CSS, JS
pnpm format:all     # Prettier + cargo fmt (Rust)
```

## Testing

```bash
pnpm test           # Vitest (frontend unit tests)
pnpm test:all       # Full suite: svelte-check + vitest + cargo fmt check + cargo test --workspace
```

## Deployment Tags

Managed via `dev.py` (also aliased in `package.json`):

```bash
pnpm tag:test       # Reset and push the 'test' tag to HEAD - triggers testrun
pnpm tag:release    # Reset and push the 'release' tag to HEAD - triggers bundle
```

Or directly:

```bash
python3 dev.py test         # Full test suite
python3 dev.py fmt          # Format all (Rust + frontend)
python3 dev.py tag:test
python3 dev.py tag:release
```
