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
**Decision**: No automatic startup migration
- The app requires keychain access to function; no fallback is provided
- If keychain access is denied, return a localized error to the frontend and show a warning
- Provide a manual dev-only Tauri command (`migrate_legacy_tokens`) for local migration of legacy plain text tokens
- Delete plain text entry after successful manual migration

### Frontend Access
**Decision**: Frontend never handles tokens after creation
- All third-party API logic (Daylite, Planradar) is implemented in the Rust backend
- Frontend only handles the token during the initial login/creation input
- After initial creation, tokens are never passed back to the frontend
- Frontend communicates with backend exclusively via Tauri commands (`invoke`)

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