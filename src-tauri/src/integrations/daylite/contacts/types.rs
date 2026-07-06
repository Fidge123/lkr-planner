use crate::integrations::local_store::DayliteContactUrl;
use serde::{Deserialize, Serialize};
use specta::Type;

// Raw contact record as returned by the Daylite API. Only `self` needs a serde
// rename (Rust keyword). Ingestion-only: never serialized to the frontend
// (commands return `PlanningContactRecord`) or persisted.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct DayliteContactSummary {
    #[serde(rename = "self")]
    pub reference: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub urls: Vec<DayliteContactUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteUpdateContactIcalUrlsInput {
    pub contact_reference: String,
    pub primary_ical_url: String,
    pub absence_ical_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct PlanningContactRecord {
    #[serde(rename = "self")]
    pub reference: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub urls: Vec<DayliteContactUrl>,
}
