use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::contacts::{
    list_contacts_core, update_contact_ical_urls_core, DayliteUpdateContactIcalUrlsInput,
};
use super::projects::{list_projects_core, search_projects_core};
use super::shared::{DayliteSearchInput, DayliteTokenState};
use crate::integrations::http_record_replay::VcrMode;
use crate::integrations::local_store::LocalStore;
use std::sync::{Mutex, OnceLock};

const DAYLITE_BASE_URL_ENV: &str = "DAYLITE_BASE_URL";
const DAYLITE_REFRESH_TOKEN_ENV: &str = "DAYLITE_REFRESH_TOKEN";
const DAYLITE_VCR_SCOPE_ENV: &str = "DAYLITE_VCR_SCOPE";
const DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV: &str = "DAYLITE_VCR_PROJECT_SEARCH_TERM";
const DAYLITE_VCR_CONTACT_REFERENCE_ENV: &str = "DAYLITE_VCR_CONTACT_REFERENCE";
const DAYLITE_VCR_PRIMARY_ICAL_URL_ENV: &str = "DAYLITE_VCR_PRIMARY_ICAL_URL";
const DAYLITE_VCR_ABSENCE_ICAL_URL_ENV: &str = "DAYLITE_VCR_ABSENCE_ICAL_URL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayliteVcrScope {
    ReadOnly,
    All,
}

impl DayliteVcrScope {
    fn from_env() -> Result<Self, String> {
        match optional_env(DAYLITE_VCR_SCOPE_ENV)?
            .unwrap_or_else(|| "readonly".to_string())
            .to_lowercase()
            .as_str()
        {
            "readonly" => Ok(Self::ReadOnly),
            "all" => Ok(Self::All),
            value => Err(format!(
                "{DAYLITE_VCR_SCOPE_ENV} must be `readonly` or `all`, got `{value}`."
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DayliteVcrConfig {
    base_url: String,
    refresh_token: String,
    scope: DayliteVcrScope,
    project_search_term: String,
    update_contact_input: Option<DayliteUpdateContactIcalUrlsInput>,
}

impl DayliteVcrConfig {
    fn from_env() -> Result<Self, String> {
        if VcrMode::from_env() != VcrMode::Record {
            return Err("Live cassette recording requires VCR_MODE=record.".to_string());
        }

        let scope = DayliteVcrScope::from_env()?;
        let update_contact_input = match scope {
            DayliteVcrScope::ReadOnly => None,
            DayliteVcrScope::All => Some(DayliteUpdateContactIcalUrlsInput {
                contact_reference: required_env(DAYLITE_VCR_CONTACT_REFERENCE_ENV)?,
                primary_ical_url: required_env(DAYLITE_VCR_PRIMARY_ICAL_URL_ENV)?,
                absence_ical_url: required_env(DAYLITE_VCR_ABSENCE_ICAL_URL_ENV)?,
            }),
        };

        Ok(Self {
            base_url: required_env(DAYLITE_BASE_URL_ENV)?,
            refresh_token: required_env(DAYLITE_REFRESH_TOKEN_ENV)?,
            scope,
            project_search_term: required_env(DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV)?,
            update_contact_input,
        })
    }
}

#[test]
#[ignore = "requires live Daylite credentials, VCR_MODE=record, and writes cassette files"]
fn record_daylite_cassettes_from_live_api() {
    tauri::async_runtime::block_on(async {
        let config = DayliteVcrConfig::from_env()
            .expect("live Daylite VCR configuration should be provided via env vars");

        let refreshed_tokens = refresh_tokens(
            &DayliteApiClient::with_env_cassette(&config.base_url, "daylite-refresh-tokens.json")
                .expect("refresh cassette client should be created"),
            config.refresh_token.clone(),
        )
        .await
        .expect("refresh token cassette should be recorded");

        let stable_token_state = DayliteTokenState {
            access_token: refreshed_tokens.access_token.clone(),
            refresh_token: refreshed_tokens.refresh_token.clone(),
            access_token_expires_at_ms: Some(u64::MAX),
        };

        list_projects_core(
            &DayliteApiClient::with_env_cassette(&config.base_url, "daylite-list-projects.json")
                .expect("project list cassette client should be created"),
            stable_token_state.clone(),
        )
        .await
        .expect("project list cassette should be recorded");

        search_projects_core(
            &DayliteApiClient::with_env_cassette(&config.base_url, "daylite-search-projects.json")
                .expect("project search cassette client should be created"),
            stable_token_state.clone(),
            &DayliteSearchInput {
                search_term: config.project_search_term.clone(),
                limit: Some(5),
            },
        )
        .await
        .expect("project search cassette should be recorded");

        list_contacts_core(
            &DayliteApiClient::with_env_cassette(&config.base_url, "daylite-list-contacts.json")
                .expect("contact list cassette client should be created"),
            stable_token_state.clone(),
        )
        .await
        .expect("contact list cassette should be recorded");

        if let Some(update_contact_input) = &config.update_contact_input {
            update_contact_ical_urls_core(
                &DayliteApiClient::with_env_cassette(
                    &config.base_url,
                    "daylite-update-contact-ical-urls.json",
                )
                .expect("contact update cassette client should be created"),
                stable_token_state,
                &mut LocalStore::default(),
                update_contact_input,
            )
            .await
            .expect("contact update cassette should be recorded");
        }
    });
}

fn required_env(key: &str) -> Result<String, String> {
    optional_env(key)?.ok_or_else(|| format!("{key} must be set for live cassette recording."))
}

fn optional_env(key: &str) -> Result<Option<String>, String> {
    let Some(value) = std::env::var_os(key) else {
        return Ok(None);
    };

    let value = value.to_string_lossy().trim().to_string();
    if value.is_empty() {
        return Err(format!("{key} must not be blank."));
    }

    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_defaults_to_read_only() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();

        assert_eq!(DayliteVcrScope::from_env(), Ok(DayliteVcrScope::ReadOnly));
    }

    #[test]
    fn scope_parses_all() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var(DAYLITE_VCR_SCOPE_ENV, "all");
        }

        assert_eq!(DayliteVcrScope::from_env(), Ok(DayliteVcrScope::All));
    }

    #[test]
    fn config_requires_mutation_inputs_for_all_scope() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var("VCR_MODE", "record");
            std::env::set_var(DAYLITE_BASE_URL_ENV, "https://daylite.example");
            std::env::set_var(DAYLITE_REFRESH_TOKEN_ENV, "refresh-token");
            std::env::set_var(DAYLITE_VCR_SCOPE_ENV, "all");
            std::env::set_var(DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV, "Nord");
        }

        let error =
            DayliteVcrConfig::from_env().expect_err("all scope should require PATCH inputs");
        assert!(error.contains(DAYLITE_VCR_CONTACT_REFERENCE_ENV));
    }

    #[test]
    fn config_allows_read_only_without_patch_inputs() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var("VCR_MODE", "record");
            std::env::set_var(DAYLITE_BASE_URL_ENV, "https://daylite.example");
            std::env::set_var(DAYLITE_REFRESH_TOKEN_ENV, "refresh-token");
            std::env::set_var(DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV, "Nord");
        }

        let config =
            DayliteVcrConfig::from_env().expect("read-only scope should not require PATCH inputs");

        assert_eq!(config.scope, DayliteVcrScope::ReadOnly);
        assert_eq!(config.update_contact_input, None);
    }

    #[test]
    fn config_requires_record_mode() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var(DAYLITE_BASE_URL_ENV, "https://daylite.example");
            std::env::set_var(DAYLITE_REFRESH_TOKEN_ENV, "refresh-token");
            std::env::set_var(DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV, "Nord");
        }

        let error = DayliteVcrConfig::from_env().expect_err("record mode should be required");
        assert_eq!(error, "Live cassette recording requires VCR_MODE=record.");
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_env() {
        for key in [
            "VCR_MODE",
            DAYLITE_BASE_URL_ENV,
            DAYLITE_REFRESH_TOKEN_ENV,
            DAYLITE_VCR_SCOPE_ENV,
            DAYLITE_VCR_PROJECT_SEARCH_TERM_ENV,
            DAYLITE_VCR_CONTACT_REFERENCE_ENV,
            DAYLITE_VCR_PRIMARY_ICAL_URL_ENV,
            DAYLITE_VCR_ABSENCE_ICAL_URL_ENV,
        ] {
            unsafe {
                std::env::remove_var(key);
            }
        }
    }
}
