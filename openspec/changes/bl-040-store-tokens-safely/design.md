## Context

Authentication tokens for external APIs should not be stored in plain text. macOS Keychain provides secure, OS-level credential storage.

## Goals / Non-Goals

**Goals:**
- Store tokens in OS keychain (macOS) / Credential Manager (Windows) / Secret Service (Linux)
- Migrate existing plain text tokens on startup
- No tokens in localStorage/sessionStorage on frontend
- Retrieve tokens via Tauri commands into memory only

**Non-Goals:**
- Encrypting entire local-store database
- Custom encryption algorithms
- Cross-device sync

## Decisions

### Storage Library
**Decision**: Use `keyring` crate in Rust backend
- Cross-platform support (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Simple API for get/set/delete operations
- Alternative: tauri-plugin-stronghold for more features

### Token Migration
**Decision**: Migrate on application startup
- Check legacy store for plain text tokens
- If found, write to secure storage
- Delete plain text entry after successful migration
- Log migration actions

### Frontend Access
**Decision**: Frontend never stores tokens directly
- All token operations go through Tauri commands
- Tokens stored in memory only during session
- Frontend receives token only when making API call

### Service Naming
**Decision**: Use service identifier for keyring entries
- Service name: "lkr-planner-planradar" for Planradar tokens
- Service name: "lkr-planner-daylite" for Daylite tokens
- Username: can be empty or app-specific

## Risks / Trade-offs

- **Risk**: Keychain access denied by user
  - **Mitigation**: Prompt for permission; show error if denied

- **Risk**: Migration fails mid-process
  - **Mitigation**: Transaction-like behavior; rollback on failure

- **Risk**: Lost keychain entry (user deletes)
  - **Mitigation**: Prompt user to re-enter token