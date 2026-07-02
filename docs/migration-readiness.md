# Migration Readiness and QA Checklist

This document defines the practical gate for treating the Tauri version as the primary implementation of Overlay Effect.

It focuses on user-visible behavior and release confidence. It does not require Windows or Linux parity, because macOS is the current priority platform.

## Migration goal

The Tauri version is ready to replace the Electron version for macOS usage when it can reliably provide these behaviors:

- Show mouse click effects above normal windows.
- Show keyboard input above normal windows.
- Keep the overlay transparent, frameless, shadowless, and click-through.
- Keep the overlay visible above the macOS menu bar and across Spaces where possible.
- Provide tray menu controls for visibility, mouse effects, keyboard effects, input monitoring retry, and quit.
- Persist settings across app restarts.
- Explain permission problems without crashing.
- Produce installable macOS `.app` and `.dmg` bundles.

## Feature parity gate

| Area | Expected behavior | Tauri status | Gate decision | Notes |
|---|---|---:|---|---|
| Overlay window | Transparent, frameless, no shadow | Implemented | Required | Uses Tauri window config plus macOS window-level adjustments. |
| Always-on-top behavior | Overlay stays above normal apps and the macOS menu bar | Implemented | Required | Runtime verification is required after window visibility changes. |
| Click-through | Mouse input passes to apps below the overlay | Implemented | Required | Must be verified with common apps and the desktop. |
| Mouse click effect | Left click displays a circle at the clicked position | Implemented | Required | Coordinate behavior should be checked on Retina and multi-display setups. |
| Mouse drag behavior | Drag should not create unintended click effects | Implemented | Required | Drag tracking should not regress click display. |
| Keyboard overlay | Key presses display readable key labels | Implemented | Required | JIS and US layout differences are partially handled. |
| Key release handling | Released keys disappear from the overlay | Implemented | Required | Rare key-stuck cases should be investigated only after a reliable reproduction case exists. |
| Keyboard layout labels | JIS-specific keys display as JIS labels when detected | Implemented | Required for macOS | US/JIS detection should be verified with real devices. |
| Tray menu | Toggle overlay, mouse, keyboard, retry input monitoring, quit | Implemented | Required | Chapter-related menu items are intentionally hidden. |
| Tray checked state | Tray checkmarks reflect current app state | Implemented | Required | Must stay synced after toggling from the tray and renderer-side settings changes. |
| Settings persistence | Mouse, keyboard, chapter, and timer settings are saved | Implemented | Required | Chapter settings may remain dormant while chapter UI is hidden. |
| Permission guidance | App shows guidance when input monitoring is unavailable | Implemented | Required | Must not crash when permissions are denied. |
| Multi-display bounds | Overlay covers all active displays | Implemented | Required | Must account for menu bar, coordinate origin, and Retina scale. |
| macOS distribution | `.app` and `.dmg` can be built | Implemented | Required | Signing and notarization are documented but not stored in the repository. |
| Chapter UI | Chapter-related UI and menu behavior | Omitted | Not required | The feature is experimental and not part of the current public feature set. |
| Windows support | Native overlay and input monitoring on Windows | Not implemented | Deferred | Low priority until macOS is stable. |
| Linux support | Native overlay and input monitoring on Linux | Not implemented | Deferred | Low priority until macOS is stable. |

## Release blocking issues

Treat these as blockers before calling the Tauri version a practical replacement on macOS:

- The app crashes on launch.
- The overlay window shows a title bar, border, shadow, or opaque background.
- The overlay blocks clicks to apps underneath it.
- The overlay appears below the macOS menu bar during normal operation.
- Mouse click effects appear at visibly wrong coordinates.
- Keyboard labels are unreadable or use replacement glyphs for common keys.
- Basic key release events leave common keys permanently displayed.
- Tray toggles do not match the actual overlay, mouse, or keyboard state.
- Permission denial causes a crash or silent failure with no user-facing guidance.
- The macOS `.app` or `.dmg` bundle cannot be produced from a clean checkout.

## Non-blocking known limitations

These items should be tracked, but they do not block macOS-first migration readiness:

- Windows and Linux runtime support is deferred.
- Chapter-related UI is hidden because it is experimental.
- Some rare key-stuck cases may require a reliable reproduction case before fixing.
- Unsigned local builds may require manual Gatekeeper handling.
- macOS permission state may need manual reset after reinstalling or rebuilding.
- Native input dependencies may still produce lower-level warnings depending on toolchain versions.

## Manual QA checklist

Run this checklist before merging a release-focused PR or tagging a macOS build.

### Startup

- [ ] `npm run tauri:dev` launches the app.
- [ ] The overlay appears without a title bar or border.
- [ ] The overlay background is transparent.
- [ ] The overlay does not intercept clicks.
- [ ] No unexpected debug effects appear without input.

### Mouse input

- [ ] Left click displays a circle.
- [ ] The circle appears at the clicked screen position.
- [ ] Clicking near the macOS menu bar displays the circle above the menu bar.
- [ ] Dragging does not create unexpected click effects.
- [ ] Click effects still work after hiding and showing the overlay from the tray.

### Keyboard input

- [ ] Letter keys display readable labels.
- [ ] Modifier keys display readable symbols or labels.
- [ ] Arrow keys display readable arrows.
- [ ] Tab, Return, Delete, Escape, and Space display readable labels.
- [ ] Keys disappear after release.
- [ ] JIS-specific keys display expected labels on a JIS keyboard.
- [ ] US-specific symbol keys display expected labels on a US keyboard when available.

### Tray menu

- [ ] The tray icon appears in the macOS menu bar.
- [ ] `オーバーレイを表示` toggles overlay visibility.
- [ ] The overlay visibility checkmark matches the actual overlay state.
- [ ] `マウスエフェクト` toggles mouse effects.
- [ ] The mouse checkmark matches the actual mouse effect state.
- [ ] `キーボード表示` toggles keyboard effects.
- [ ] The keyboard checkmark matches the actual keyboard effect state.
- [ ] `入力監視を再試行` can retry permission-dependent monitoring.
- [ ] Chapter-related menu items are not visible.
- [ ] Quit exits the app.

### Permissions

- [ ] With permissions granted, mouse and keyboard monitoring work.
- [ ] With Input Monitoring denied, the app shows permission guidance.
- [ ] With Accessibility denied, the app shows permission guidance when required.
- [ ] Granting permissions and using retry restores input monitoring when macOS allows it.
- [ ] Denied permissions do not crash the app.

### Multi-display and macOS window behavior

- [ ] The overlay covers the primary display.
- [ ] The overlay covers secondary displays when connected.
- [ ] Mouse coordinates are correct on each display.
- [ ] Retina and non-Retina displays do not create obvious coordinate scaling errors.
- [ ] The overlay remains visible across Mission Control / Spaces transitions where possible.
- [ ] The overlay remains above the macOS menu bar after hide/show toggles.

### Build and install

- [ ] `npm run check` passes.
- [ ] `cargo check` passes in `src-tauri`.
- [ ] `npm run tauri:build:macos` creates `.app` and `.dmg` outputs.
- [ ] The `.dmg` can be opened.
- [ ] The app can be dragged into Applications.
- [ ] The app launches from Applications.
- [ ] The installed app requests or uses macOS permissions as expected.

## Merge gate for the migration branch

A PR can move the migration closer to completion when all of these are true:

- It does not reintroduce Electron runtime dependencies into the Tauri path.
- It does not expose personal machine paths, certificates, credentials, or local-only state.
- It keeps macOS behavior working before expanding to other platforms.
- It updates README or this checklist when behavior, commands, or limitations change.
- It passes the relevant mechanical checks for the touched area.
- It receives local Codex code review when code behavior changes.

## Deferred platform work

Windows and Linux support should be handled as separate platform projects later.

Before starting that work, define OS-specific plans for:

- Transparent click-through overlay windows.
- Always-on-top and virtual desktop behavior.
- Global mouse monitoring.
- Global keyboard monitoring.
- Permission or accessibility guidance.
- Tray/menu behavior.
- Display coordinate systems and scaling.
- Packaging and signing.
