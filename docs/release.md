# GitHub Release Distribution Guide

This guide describes how to publish a macOS `.dmg` build of Overlay Effect through GitHub Releases.

The current distribution target is macOS. Windows and Linux release artifacts are intentionally out of scope until those platforms are implemented and verified.

## Release policy

- Publish macOS builds as GitHub Release assets.
- Attach the generated `.dmg` file to the release.
- Keep signing certificates, notarization credentials, Apple IDs, app-specific passwords, keychains, and provisioning profiles out of the repository.
- Keep `package.json` and `src-tauri/tauri.conf.json` versions synchronized when the app version changes.
- Prefer a draft GitHub Release until the `.dmg` has been installed and launched locally.

## Versioning

Use a Git tag for the public release version.

Recommended beta pattern:

- App version: `0.1.0`
- Git tag: `v0.1.0-beta.1`
- Release title: `Overlay Effect 0.1.0 beta 1`

Keep the app version numeric for macOS bundle compatibility. Use the GitHub Release tag and title to express beta status.

Before changing the version, update both files together:

- `package.json`
- `src-tauri/tauri.conf.json`

## Pre-release checklist

Run these checks before creating the GitHub Release.

- [ ] Confirm `README.md` describes the current public behavior.
- [ ] Confirm `docs/migration-readiness.md` matches the current migration state.
- [ ] Confirm no personal machine paths, credentials, certificates, keychains, or local-only files are staged.
- [ ] Confirm `package.json` and `src-tauri/tauri.conf.json` versions match.
- [ ] Run `npm run check`.
- [ ] Run `cargo check` from `src-tauri`.
- [ ] Run `npm run tauri:build:macos`.
- [ ] Install the generated `.dmg` locally.
- [ ] Launch the installed app from Applications.
- [ ] Confirm mouse click effects work.
- [ ] Confirm keyboard overlay works.
- [ ] Confirm tray toggles and checkmarks match app state.
- [ ] Confirm macOS permission guidance appears when input monitoring is unavailable.
- [ ] Confirm the overlay stays above the macOS menu bar after hide/show toggles.

## Build commands

Install dependencies if needed:

```bash
npm install
```

Run frontend checks:

```bash
npm run check
```

Run Rust checks:

```bash
cd src-tauri
cargo check
cd ..
```

Build the macOS app and DMG:

```bash
npm run tauri:build:macos
```

Expected output directories:

- `src-tauri/target/release/bundle/macos`
- `src-tauri/target/release/bundle/dmg`

## Local install verification

Use the generated `.dmg` before uploading it to GitHub Releases.

1. Open the `.dmg`.
2. Drag `Overlay Effect.app` into Applications.
3. Launch `Overlay Effect.app` from Applications.
4. Grant macOS permissions when needed.
5. Confirm the overlay works with real mouse and keyboard input.
6. Confirm the tray menu can hide/show the overlay and toggle mouse/keyboard effects.

If the build is unsigned or not notarized, macOS may show Gatekeeper warnings. Document that state in the GitHub Release notes.

## Checksums

Generate a SHA-256 checksum for the DMG and include it in the release notes.

```bash
shasum -a 256 src-tauri/target/release/bundle/dmg/*.dmg
```

## Creating a GitHub Release

Manual flow:

1. Create a Git tag.
2. Push the tag.
3. Create a draft GitHub Release from the tag.
4. Upload the `.dmg` asset.
5. Paste the checksum into the release notes.
6. Publish the release after local install verification is complete.

GitHub CLI example:

```bash
git tag v0.1.0-beta.1
git push origin v0.1.0-beta.1

gh release create v0.1.0-beta.1 \
  src-tauri/target/release/bundle/dmg/*.dmg \
  --draft \
  --title "Overlay Effect 0.1.0 beta 1" \
  --notes-file release-notes.md
```

Do not commit `release-notes.md` unless it is intentionally maintained as a reusable template.

## Release notes template

````markdown
# Overlay Effect 0.1.0 beta 1

## What this is

Overlay Effect is a macOS app that visualizes mouse clicks and keyboard input over the screen.

## Supported platform

- macOS

Windows and Linux builds are not included in this release.

## Installation

1. Download the attached `.dmg` file.
2. Open the `.dmg`.
3. Drag `Overlay Effect.app` into Applications.
4. Launch the app.
5. Grant Accessibility and Input Monitoring permissions when macOS asks.

## macOS security notice

This build is currently unsigned / not notarized.

macOS may show a security warning on first launch. If this changes for a future release, update this section before publishing.

## Known limitations

- Windows and Linux are not supported yet.
- Chapter-related UI is intentionally hidden.
- Global input monitoring depends on macOS permission state.
- Some permission changes may require restarting the app or using the tray retry item.

## Verification

- Frontend check: `npm run check`
- Rust check: `cargo check`
- Build: `npm run tauri:build:macos`
- Local install from DMG: verified / not verified

## SHA-256

```text
<sha256>  <dmg file name>
```
````

## Signing and notarization

Signing and notarization are optional for local testing, but recommended for public releases.

If signing is used, provide credentials through the local macOS keychain and environment variables. Do not commit any secret material.

Relevant environment variables:

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Example Team (TEAMID)"
export APPLE_ID="apple-id@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAMID"
```

Run the normal build after the local signing environment is configured:

```bash
npm run tauri:build:macos
```

After changing signing or notarization settings, re-run the full local install verification from the generated `.dmg`.
