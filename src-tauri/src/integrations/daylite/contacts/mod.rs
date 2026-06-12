mod api;
pub mod commands;
mod ical_urls;
mod mapping;
pub mod types;

pub use api::sync_contact_ical_urls;
pub use types::DayliteUpdateContactIcalUrlsInput;

// Re-exported for the daylite recording harness (test-only) which drives the
// cores directly to record/replay VCR cassettes.
#[cfg(test)]
pub(in crate::integrations::daylite) use api::{list_contacts_core, update_contact_ical_urls_core};
