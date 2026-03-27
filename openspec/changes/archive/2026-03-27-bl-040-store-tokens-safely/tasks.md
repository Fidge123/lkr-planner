## 1. Secure Storage Module

- [x] 1.1 Add keyring crate to Cargo.toml and scaffold `secret_manager` module
- [x] 1.2 Write failing unit tests for `set_token`, `get_token`, and `delete_token`
- [x] 1.3 Implement `set_token`, `get_token`, and `delete_token` to make tests pass
- [x] 1.4 Refactor `secret_manager` module as needed

## 2. Token Migration & Error Handling

- [x] 2.1 Write failing test: keychain access denied returns correct strongly-typed error
- [x] 2.2 Implement robust error handling for keychain access denial to make test pass
- [x] 2.3 Write failing test for dev manual migration command `migrate_legacy_tokens`
- [x] 2.4 Implement logic to read legacy store and write to secure storage in dev command
- [x] 2.5 Ensure plain text entries are deleted after successful manual migration

## 3. Tauri Command Updates & Integration Validation

- [x] 3.1 Write failing integration test: saving a token does NOT write to local plain text store
- [x] 3.2 Update `set-token` and `delete-token` Tauri commands to use secure storage to make test pass
- [x] 3.3 Write failing test and implement `check-token` command (returns boolean, not the actual token)
- [x] 3.4 Migrate any remaining third-party API requests to Rust backend to isolate tokens
- [x] 3.5 Remove any frontend commands that retrieve the actual token
- [x] 3.6 Implement user-friendly frontend error UI for missing keychain permissions