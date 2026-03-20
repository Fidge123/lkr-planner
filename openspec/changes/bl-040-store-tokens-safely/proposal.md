## Why

API tokens stored in plain text files are vulnerable to exposure. We need to use OS-level secure storage (Keychain on macOS) to protect authentication tokens for Planradar and Daylite APIs.

## What Changes

- Integrate keyring crate for OS secure storage
- Create secret management module in Rust backend
- Move all third-party API logic to the Rust backend
- Provide a developer-only manual migration command
- Display clear errors if OS keychain access is denied

## Capabilities

### New Capabilities
- `secure-token-storage`: Store tokens in OS keychain
- `token-migration-dev`: Manual migration tool for developers

### Modified Capabilities
- `token-management`: Extended to use secure storage

## Impact

- Code: New Rust secret management module, migration logic
- Dependencies: keyring crate (or tauri-plugin-stronghold)
- Security: Tokens no longer visible in plain text files