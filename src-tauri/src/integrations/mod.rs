/// Integration modules for external services and APIs
///
/// This module contains all integration code for:
/// - External API clients (Daylite, Planradar, iCal)
/// - Health checks and status monitoring
/// - Any other external service integrations
///
/// Architecture principle:
/// - Network calls and secrets are handled here in Rust
/// - Frontend consumes these via Tauri commands
/// - Each integration exposes Tauri commands for the frontend service layer
pub mod health;
pub mod local_store;
