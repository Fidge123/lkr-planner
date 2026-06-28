use super::client::PlanradarApiClient;
use super::projects::{
    create_project_core, list_projects_core, read_project_status_core,
    PlanradarCreateProjectRequest, PlanradarListProjectsInput,
};
use crate::integrations::http_record_replay::VcrMode;

const PLANRADAR_BASE_URL_ENV: &str = "PLANRADAR_BASE_URL";
const PLANRADAR_API_TOKEN_ENV: &str = "PLANRADAR_API_TOKEN";
const PLANRADAR_CUSTOMER_ID_ENV: &str = "PLANRADAR_CUSTOMER_ID";
const PLANRADAR_VCR_PROJECT_ID_ENV: &str = "PLANRADAR_VCR_PROJECT_ID";
const PLANRADAR_VCR_NEW_PROJECT_NAME_ENV: &str = "PLANRADAR_VCR_NEW_PROJECT_NAME";

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanradarVcrConfig {
    base_url: String,
    api_token: String,
    customer_id: String,
    project_id: String,
    new_project_name: String,
}

impl PlanradarVcrConfig {
    fn from_env() -> Result<Self, String> {
        if VcrMode::from_env() != VcrMode::Record {
            return Err("Live cassette recording requires VCR_MODE=record.".to_string());
        }

        Ok(Self {
            base_url: required_env(PLANRADAR_BASE_URL_ENV)?,
            api_token: required_env(PLANRADAR_API_TOKEN_ENV)?,
            customer_id: required_env(PLANRADAR_CUSTOMER_ID_ENV)?,
            project_id: required_env(PLANRADAR_VCR_PROJECT_ID_ENV)?,
            new_project_name: required_env(PLANRADAR_VCR_NEW_PROJECT_NAME_ENV)?,
        })
    }
}

#[test]
#[ignore = "requires live Planradar credentials, VCR_MODE=record, and writes cassette files"]
fn record_planradar_cassettes_from_live_api() {
    tauri::async_runtime::block_on(async {
        let config = PlanradarVcrConfig::from_env()
            .expect("live Planradar VCR configuration should be provided via env vars");

        read_project_status_core(
            &PlanradarApiClient::with_env_cassette(&config.base_url, "planradar-get-project.json")
                .expect("status cassette client should be created"),
            &config.api_token,
            &config.customer_id,
            &config.project_id,
        )
        .await
        .expect("status cassette should be recorded");

        list_projects_core(
            &PlanradarApiClient::with_env_cassette(
                &config.base_url,
                "planradar-list-projects.json",
            )
            .expect("list cassette client should be created"),
            &config.api_token,
            &config.customer_id,
            &PlanradarListProjectsInput {
                sort: Some("name".to_string()),
                page: Some(1),
                pagesize: Some(10),
            },
        )
        .await
        .expect("list cassette should be recorded");

        create_project_core(
            &PlanradarApiClient::with_env_cassette(
                &config.base_url,
                "planradar-create-project.json",
            )
            .expect("create cassette client should be created"),
            &config.api_token,
            &config.customer_id,
            &PlanradarCreateProjectRequest {
                name: config.new_project_name.clone(),
                ..PlanradarCreateProjectRequest::default()
            },
        )
        .await
        .expect("create cassette should be recorded");
    });
}

fn required_env(key: &str) -> Result<String, String> {
    let Some(value) = std::env::var_os(key) else {
        return Err(format!("{key} must be set for live cassette recording."));
    };

    let value = value.to_string_lossy().trim().to_string();
    if value.is_empty() {
        return Err(format!("{key} must not be blank."));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    #[test]
    fn config_requires_record_mode() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var(PLANRADAR_BASE_URL_ENV, "https://www.planradar.com");
            std::env::set_var(PLANRADAR_API_TOKEN_ENV, "token");
            std::env::set_var(PLANRADAR_CUSTOMER_ID_ENV, "1234");
            std::env::set_var(PLANRADAR_VCR_PROJECT_ID_ENV, "1");
            std::env::set_var(PLANRADAR_VCR_NEW_PROJECT_NAME_ENV, "Neu");
        }

        let error = PlanradarVcrConfig::from_env().expect_err("record mode should be required");
        assert_eq!(error, "Live cassette recording requires VCR_MODE=record.");
        clear_env();
    }

    #[test]
    fn config_reads_all_values_in_record_mode() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var("VCR_MODE", "record");
            std::env::set_var(PLANRADAR_BASE_URL_ENV, "https://www.planradar.com");
            std::env::set_var(PLANRADAR_API_TOKEN_ENV, "token");
            std::env::set_var(PLANRADAR_CUSTOMER_ID_ENV, "1234");
            std::env::set_var(PLANRADAR_VCR_PROJECT_ID_ENV, "1");
            std::env::set_var(PLANRADAR_VCR_NEW_PROJECT_NAME_ENV, "Neu");
        }

        let config = PlanradarVcrConfig::from_env().expect("config should resolve");
        assert_eq!(config.customer_id, "1234");
        assert_eq!(config.new_project_name, "Neu");
        clear_env();
    }

    #[test]
    fn config_reports_missing_values() {
        let _guard = env_lock().lock().expect("env lock should not be poisoned");
        clear_env();
        unsafe {
            std::env::set_var("VCR_MODE", "record");
        }

        let error = PlanradarVcrConfig::from_env().expect_err("missing values should fail");
        assert!(error.contains(PLANRADAR_BASE_URL_ENV));
        clear_env();
    }

    fn env_lock() -> &'static Mutex<()> {
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_env() {
        for key in [
            "VCR_MODE",
            PLANRADAR_BASE_URL_ENV,
            PLANRADAR_API_TOKEN_ENV,
            PLANRADAR_CUSTOMER_ID_ENV,
            PLANRADAR_VCR_PROJECT_ID_ENV,
            PLANRADAR_VCR_NEW_PROJECT_NAME_ENV,
        ] {
            unsafe {
                std::env::remove_var(key);
            }
        }
    }
}
