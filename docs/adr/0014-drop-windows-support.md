# ADR 0014: Drop Windows Support

- Status: Accepted
- Date: 2026-10-16

## Context

The application is only built and released for macOS, and the release workflow has no Windows target.
Windows support was carried along implicitly through a Windows credential store and the default features of `tauri-plugin-updater`.
The updater's default `zip` feature pulls in the `zip` crate, which it only uses to unpack zipped Windows installers.
Every `zip` release below 9.0.3 has been yanked after the October 2026 extraction advisories (GHSA-4v9h-p7m9-rh26, GHSA-vv5m-jxxh-4627, GHSA-4684-h2q9-2vcp), and the updater still requires `zip` 4.
`Cargo.lock` resolves dependencies for every target, so the yanked crate blocked a plain `cargo update` even though no shipped build compiles it.

### Evaluated Options
- Drop Windows support and disable the updater's `zip` feature
  - Pros: Removes `zip` and the Windows credential store from the dependency graph; `cargo update` works again; no unused platform code.
  - Cons: Windows builds and updates require re-adding both; Linux updates from compressed AppImages stop working.
- Keep Windows support and update every crate except `zip`
  - Pros: No change to supported platforms.
  - Cons: Every dependency update needs a manual workaround until the updater moves to a patched `zip`; carries a yanked crate for a platform that is never shipped.
- Disable the updater's `zip` feature but keep the Windows credential store
  - Pros: Smallest change that removes `zip`.
  - Cons: Leaves a half-supported platform whose builds would silently lose zipped updates.

## Decision

- Support macOS and Linux only; Linux stays because development and CI run there.
- Remove `windows-native-keyring-store`, the Windows branch of `secret_manager::init`, the `windows_subsystem` attribute, and the Windows entry in the desktop capability.
- Remove the `icon.ico` Windows icon from the bundle configuration and from icon regeneration.
- Depend on `tauri-plugin-updater` with `default-features = false` and only the `rustls-tls` and `system-proxy` features.

## Consequences

- `zip` no longer appears in `Cargo.lock`, and dependency updates need no workaround.
- macOS updates are unaffected because the updater always includes the tar and gzip support for `.app.tar.gz` on macOS.
- Shipping Linux builds with compressed AppImage updates requires re-enabling the updater's `zip` feature.
- Adding Windows support later requires restoring the Windows credential store, the `zip` feature, and the `icon.ico` generation.
