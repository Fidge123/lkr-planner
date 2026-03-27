## Why

The application needs to integrate with Planradar for project management. A typed API client is required to enable project search, creation, and status management. This provides the foundation for all Planradar integration features.

## What Changes

- Implement typed Planradar client with project search/list capabilities
- Add project create functionality (template-based when required)
- Add project status read (active/archived/reopen support)
- Normalize API error payloads for frontend usage
- Make tenant/account settings configurable

## Capabilities

### New Capabilities
- `planradar-api-client`: Typed Planradar API client for project operations

### Modified Capabilities
<!-- No existing spec requirements are changing -->

## Impact

- Code: New Rust module for Planradar client in Tauri backend
- APIs: Planradar REST API integration
- Dependencies: Planradar API credentials and configuration