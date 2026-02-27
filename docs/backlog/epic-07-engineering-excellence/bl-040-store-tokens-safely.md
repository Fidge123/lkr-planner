# BL-040: Secure Storage of Authentication Tokens

## Scope

- Utilize a secure storage mechanism provided by the OS (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux) instead of storing tokens in plain text in the local-store.
- Integrate the `keyring` crate (or an equivalent Tauri secure storage plugin like `tauri-plugin-stronghold`) to securely store API tokens (e.g., Daylite integration tokens).
- Migrate any existing plaintext tokens to the secure store automatically on application startup, then securely remove them from the plain text store.
- Update the Tauri command handlers that read and write tokens to interface with the secure storage vault.

## Acceptance Criteria

- Tokens (e.g., API keys, access tokens) are no longer visible in plain text within the application's configuration or local-store files on the filesystem.
- Saving a new token or updating an existing one securely stores it within the OS-level credential manager automatically.
- The application seamlessly retrieves tokens using the OS-level credential manager.
- Upon launch, the application checks for plain text tokens from previous versions, migrates them to the secure store, and completely removes them from the legacy file.

## Dependencies

- Requires adding the `keyring` crate (or `tauri-plugin-stronghold`) to the Rust backend dependencies in `src-tauri/Cargo.toml`.
- Access to the target OS's native secret storage APIs.

## Out of Scope

- Encrypting the entire application local-store database or user configuration files; only sensitive tokens/secrets need to be protected.
- Implementing custom encryption algorithms; we strictly rely on established OS-provided or vetted secure storage libraries.
- Synchronizing the secure store across multiple user devices.

## Implementation Notes

- **Backend Integration**: Create a secret management module in Rust that abstracts the OS keyring interactions.
- **Migration Logic**: Create a startup hook in the Tauri application setup phase. If legacy tokens are found in the standard local-store, read them, save them using the secure vault, and delete the plaintext entry.
- **Frontend Restrictions**: Ensure the web frontend's `localStorage` or `sessionStorage` is never used to house these tokens. All API requests relying on tokens should ideally be proxied through the Rust backend, or the token must be fetched securely via a Tauri command into memory only.

## Tests (write first)

- **Plaintext Absence Test**: Write an integration test that saves a token via the application and asserts that the resulting config/store file on disk does NOT contain the token string.
- **Migration Test**: Write a test that simulates a legacy store with a plaintext token, runs the migration logic, and asserts that the token securely transferred to the vault and the legacy token was deleted.
- **Secure Vault Integration Test**: Unit test the Rust backend token manager to ensure `set_token`, `get_token`, and `delete_token` correctly interact with the configured secure backend (using a mock or a test vault environment).
