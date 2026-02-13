use serde::{Deserialize, Serialize};

/// Health status response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthStatusEnum,
    pub timestamp: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusEnum {
    Healthy,
    Unhealthy,
}

/// Check the health status of the application
#[tauri::command]
pub fn check_health() -> Result<HealthStatus, String> {
    let now = chrono::Utc::now();
    let version = env!("CARGO_PKG_VERSION");

    Ok(HealthStatus {
        status: HealthStatusEnum::Healthy,
        timestamp: now.to_rfc3339(),
        version: version.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_health_returns_healthy_status() {
        let result = check_health();
        assert!(result.is_ok());

        let health = result.unwrap();
        assert!(matches!(health.status, HealthStatusEnum::Healthy));
        assert!(!health.timestamp.is_empty());
        assert!(!health.version.is_empty());
    }

    #[test]
    fn test_health_status_has_valid_version() {
        let result = check_health().unwrap();
        assert_eq!(result.version, env!("CARGO_PKG_VERSION"));
    }
}
