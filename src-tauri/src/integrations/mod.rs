/// Integrations with external services. Network calls and secrets stay in Rust;
/// the frontend consumes them via Tauri commands (see docs/adr/0001 and 0002).
pub mod calendar;
pub mod daylite;
pub mod health;
pub mod holidays;
#[cfg(test)]
pub(crate) mod http_record_replay;
pub mod local_store;
pub mod zep;
