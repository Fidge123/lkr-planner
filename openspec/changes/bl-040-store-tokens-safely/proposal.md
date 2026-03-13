## Why

API tokens stored in plain text files are vulnerable to exposure. We need to use OS-level secure storage (Keychain on macOS) to protect authentication tokens for Planradar and Daylite APIs.

## What Changes

- Integrate keyring crate for OS secure storage
- Create secret management module in Rust backend
- Migrate existing plain text tokens to secure store on startup
- Update Tauri commands to use secure storage

## Capabilities

### New Capabilities
- `secure-token-storage`: Store tokens in OS keychain
- `token-migration`: Migrate plain text tokens to secure storage

### Modified Capabilities
- `token-management`: Extended to use secure storage

## Impact

- Code: New Rust secret management module, migration logic
- Dependencies: keyring crate (or tauri-plugin-stronghold)
- Security: Tokens no longer visible in plain text files