# Tauri Overlay Effect (Migration Prototype)

This repository is a Tauri v2 + Svelte migration prototype of the overlay effect app.
This document is the public-facing setup and verification guide.

## Requirements
- Node.js 20.19+
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

### macOS distribution build
```bash
npm run tauri:build:macos
```
- Builds the configured macOS `.app` and `.dmg` bundles.
- The `.app` bundle is created under `src-tauri/target/release/bundle/macos`.
- The `.dmg` bundle is created under `src-tauri/target/release/bundle/dmg`.

To build only one macOS bundle type:
```bash
npm run tauri:build:macos:app
npm run tauri:build:macos:dmg
```

### macOS signing and notarization
The repository does not store signing certificates, API keys, app-specific passwords, or provisioning profiles.

For local signing, install the certificate into your macOS keychain and check the signing identity:
```bash
security find-identity -v -p codesigning
```

Then build with the signing identity supplied through the environment:
```bash
APPLE_SIGNING_IDENTITY="Developer ID Application: Example Team (TEAMID)" npm run tauri:build:macos
```

For notarization with an Apple ID app-specific password, set these environment variables before running the build:
```bash
export APPLE_ID="apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAMID"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Example Team (TEAMID)"

npm run tauri:build:macos
```

`src-tauri/Entitlements.plist` is intentionally minimal. Do not enable App Sandbox without rechecking global mouse/keyboard monitoring, click-through overlay behavior, and macOS permission prompts.

## Troubleshooting
- Port conflict: Vite uses `5173` in this repo.
- Permission prompts (macOS): check Accessibility settings if input permissions are required for full native features.
- Dirty state or unexpected generated files: confirm `.gitignore` is respected (`node_modules`, `.svelte-kit`, `build`, `src-tauri/target`, `src-tauri/gen`).
- macOS Gatekeeper warning: unsigned local builds may require manual approval. Use signing and notarization for public distribution.

## Notes
- The repository keeps generated build artifacts untracked.
- This document should be used for local reproducible checks before feature work.
