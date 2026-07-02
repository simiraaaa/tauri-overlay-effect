# Tauri Overlay Effect

Tauri Overlay Effect is a macOS overlay app that visualizes mouse clicks and keyboard input on top of the screen.

It is useful for screen sharing, demos, tutorials, recordings, and live coding sessions where viewers need to see what you clicked or typed.

This repository is a Tauri v2 + Svelte migration of the original Electron overlay app.

## Features

- Shows mouse click effects over the desktop.
- Shows keyboard input as an overlay.
- Uses a transparent, frameless, click-through overlay window.
- Keeps the overlay above normal windows and macOS Spaces where possible.
- Provides a tray menu to show/hide the overlay, toggle mouse effects, toggle keyboard effects, retry input monitoring, and quit the app.
- Supports multi-display overlay bounds on macOS.
- Provides macOS permission guidance when global input monitoring is not active.

## Platform support

macOS is the primary supported platform.

Windows and Linux are not currently supported or verified. Cross-platform support is intentionally low priority until the macOS version is stable.

## Basic usage on macOS

1. Download or build the macOS `.dmg`.
2. Open the `.dmg`.
3. Drag `Overlay Effect.app` into the Applications folder.
4. Launch the app.
5. Grant macOS permissions when needed.
6. Use the tray icon in the menu bar to toggle overlay, mouse, and keyboard display.

Unsigned local builds may show a macOS Gatekeeper warning. Public distribution should use signing and notarization.

## macOS permissions

The app needs macOS permissions to observe global mouse and keyboard input.

Enable these permissions in System Settings when prompted:

- Accessibility
- Input Monitoring

If the app stops receiving input after reinstalling or rebuilding, remove the old app entry from the permission list, launch the app again, and grant permission again.

If input monitoring was granted while the app was already running, use the tray menu item `入力監視を再試行` (`Retry Input Monitoring`).

## Developer setup

Requirements:

- Node.js 20.19+
- Rust
- macOS for the currently supported native runtime

Install dependencies:

```bash
npm install
```

Run the Tauri app in development mode:

```bash
npm run tauri:dev
```

Known development note: macOS native input monitoring can interact poorly with WebView text input in terminal-launched dev builds and may crash while editing the experimental chapter settings window. A packaged app build is more representative for testing that experimental UI.

Run the Svelte frontend only:

```bash
npm run dev
```

Run type and Svelte checks:

```bash
npm run check
```

Build the web bundle:

```bash
npm run build
```

## macOS builds

Build the configured macOS `.app` and `.dmg` bundles:

```bash
npm run tauri:build:macos
```

Build only one bundle type:

```bash
npm run tauri:build:macos:app
npm run tauri:build:macos:dmg
```

Build outputs are created under:

- `src-tauri/target/release/bundle/macos`
- `src-tauri/target/release/bundle/dmg`

DMG creation uses macOS `hdiutil`. It may fail inside restricted sandbox environments even when it succeeds in a normal local terminal.

GitHub Release distribution steps are maintained in [docs/release.md](docs/release.md).

## Signing and notarization

The repository does not store signing certificates, API keys, app-specific passwords, keychains, or provisioning profiles.

For local signing, install the certificate into your macOS keychain and check the signing identity:

```bash
security find-identity -v -p codesigning
```

Then pass the identity through the environment:

```bash
APPLE_SIGNING_IDENTITY="Developer ID Application: Example Team (TEAMID)" npm run tauri:build:macos
```

For notarization with an Apple ID app-specific password:

```bash
export APPLE_ID="apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAMID"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Example Team (TEAMID)"

npm run tauri:build:macos
```

`src-tauri/Entitlements.plist` is intentionally minimal. Do not enable App Sandbox without rechecking global mouse monitoring, global keyboard monitoring, click-through overlay behavior, and macOS permission prompts.

## Current limitations

- Windows and Linux support is not implemented or verified.
- The app is still a migration prototype, not a final replacement release.
- Some lower-level warnings from vendored native input dependencies remain during Rust builds.
- Global input monitoring depends on macOS permission state and may require manual permission reset after reinstalling or rebuilding.
- Chapter-related UI from the original app is experimental and not part of the current public feature set.

## Migration readiness

The macOS-first migration gate and manual QA checklist are maintained in [docs/migration-readiness.md](docs/migration-readiness.md).
