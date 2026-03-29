# Release and Distribution

This document describes how to ship `wilds` binaries for macOS and Windows.

## Targets

The release workflow builds:

- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `x86_64-pc-windows-msvc`

## Artifact Format

- macOS: `wilds-<version>-<target>.tar.gz`
- Windows: `wilds-<version>-<target>.zip`
- Checksums: `<artifact>.sha256`

Each archive contains:

- the executable (`wilds` or `wilds.exe`)
- `README.md`
- `migrations/`

## Tag-Based Release Flow

1. Ensure CI is green on `main`.
2. Create and push a version tag:
   - `git tag v0.1.0`
   - `git push origin v0.1.0`
3. Wait for `.github/workflows/release.yml` to finish.
4. Verify the GitHub Release contains all 3 artifacts and their checksums.
5. Smoke test one macOS artifact and one Windows artifact.

## Manual Local Packaging

Build first:

- `cargo build --release --locked --target <target-triple>`

Then package:

- Unix/macOS:
  - `bash scripts/release/package_unix.sh wilds x86_64-apple-darwin v0.1.0`
- Windows PowerShell:
  - `./scripts/release/package_windows.ps1 -AppName wilds -Target x86_64-pc-windows-msvc -Version v0.1.0`

Output is written to `dist/`.

## Validation Checklist

- App launches in a standard terminal:
  - macOS Terminal/iTerm/WezTerm
  - Windows Terminal/PowerShell
- `wilds.db` is created and writable in the current working directory.
- Character creation/load paths work.
- Basic combat flow works (start, actions, exit).

## Known Platform Notes

- Unsigned binaries may trigger Gatekeeper/SmartScreen warnings.
- Prefer shipping signed binaries for public distribution.
- Future improvement: move `wilds.db` to per-user app-data directories.

## Optional Hardening

- macOS: code signing + notarization.
- Windows: Authenticode signing for binary or installer.
