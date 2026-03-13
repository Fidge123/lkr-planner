## 1. Secure Storage Module

- [ ] 1.1 Add keyring crate to Cargo.toml
- [ ] 1.2 Create secret_manager module in Rust backend
- [ ] 1.3 Implement `set_token(service, token)` function
- [ ] 1.4 Implement `get_token(service)` function
- [ ] 1.5 Implement `delete_token(service)` function

## 2. Token Migration

- [ ] 2.1 Add startup hook in Tauri setup
- [ ] 2.2 Check legacy store for plain text tokens
- [ ] 2.3 Write tokens to secure storage
- [ ] 2.4 Delete plain text entries after migration
- [ ] 2.5 Log migration actions

## 3. Tauri Command Updates

- [ ] 3.1 Update get-token command to use secure storage
- [ ] 3.2 Update set-token command to use secure storage
- [ ] 3.3 Update delete-token command to use secure storage
- [ ] 3.4 Ensure frontend uses commands instead of localStorage

## 4. Testing

- [ ] 4.1 Write integration test: save token, verify not in store file
- [ ] 4.2 Write migration test: legacy token moves to secure storage
- [ ] 4.3 Write unit test for set_token/get_token/delete_token
- [ ] 4.4 Write test for migration with no legacy tokens