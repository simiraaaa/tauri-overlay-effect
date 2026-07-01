# Tauri Overlay Effect (Migration Prototype)

This repository is a Tauri v2 + Svelte migration prototype of the overlay effect app.
This document is the public-facing setup and verification guide.

## Requirements
- Node.js 18+
- Rust (for Tauri)
- macOS (minimum first-tier support)

## Setup
```bash
# clone repository
# git clone <repo-url>
cd <repo-root>

npm install
```

## Development check
### Start (recommended)
```bash
npm run tauri:dev
```
- Vite and Tauri launcher starts together.
- Confirm overlay window appears.
- Confirm transparent / frameless / no shadow / always-on-top behavior.
- Confirm mouse events pass through the underlying app when click-through is expected.

### Frontend only
```bash
npm run dev
```
- Quick check for Svelte UI rendering and CSS updates.

### Type / static checks
```bash
npm run check
```
- Runs TypeScript/kit checks.

### Production build (web bundle)
```bash
npm run build
```

### Tauri build
```bash
npm run tauri:build
```
- Build artifacts are created under `src-tauri/target/release` and packaged output.

## Troubleshooting
- Port conflict: Vite uses `5173` in this repo.
- Permission prompts (macOS): check Accessibility settings if input permissions are required for full native features.
- Dirty state or unexpected generated files: confirm `.gitignore` is respected (`node_modules`, `.svelte-kit`, `build`, `src-tauri/target`, `src-tauri/gen`).

## Notes
- The repository keeps generated build artifacts untracked.
- This document should be used for local reproducible checks before feature work.
