use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VcrMode {
    Record,
    Replay,
}

impl VcrMode {
    pub(crate) fn from_env() -> Self {
        match std::env::var("VCR_MODE") {
            Ok(value) if value.trim().eq_ignore_ascii_case("record") => Self::Record,
            _ => Self::Replay,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RecordReplayConfig {
    cassette_path: PathBuf,
    mode: VcrMode,
}

impl RecordReplayConfig {
    pub(crate) fn new(cassette_path: PathBuf, mode: VcrMode) -> Self {
        Self {
            cassette_path,
            mode,
        }
    }

    pub(crate) fn from_env(cassette_path: PathBuf) -> Self {
        Self::new(cassette_path, VcrMode::from_env())
    }

    pub(crate) fn mode(&self) -> VcrMode {
        self.mode
    }

    pub(crate) fn replay(
        &self,
        request: &RecordedRequest,
    ) -> Result<Option<RecordedResponse>, RecordReplayError> {
        let cassette = self.read_existing_cassette()?;

        Ok(cassette
            .interactions
            .into_iter()
            .find(|interaction| interaction.request == *request)
            .map(|interaction| interaction.response))
    }

    pub(crate) fn record(&self, interaction: RecordedInteraction) -> Result<(), RecordReplayError> {
        let mut cassette = self.read_cassette_or_default()?;

        cassette
            .interactions
            .retain(|existing| existing.request != interaction.request);
        cassette.interactions.push(interaction);

        self.write_cassette(&cassette)
    }

    fn read_existing_cassette(&self) -> Result<CassetteFile, RecordReplayError> {
        let content = fs::read_to_string(&self.cassette_path).map_err(|error| {
            RecordReplayError::new(
                &self.cassette_path,
                format!("cassette could not be read: {error}"),
            )
        })?;

        serde_json::from_str(&content).map_err(|error| {
            RecordReplayError::new(
                &self.cassette_path,
                format!("cassette JSON is invalid: {error}"),
            )
        })
    }

    fn read_cassette_or_default(&self) -> Result<CassetteFile, RecordReplayError> {
        if self.cassette_path.exists() {
            return self.read_existing_cassette();
        }

        Ok(CassetteFile::default())
    }

    fn write_cassette(&self, cassette: &CassetteFile) -> Result<(), RecordReplayError> {
        if let Some(parent) = self.cassette_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                RecordReplayError::new(parent, format!("cassette directory is missing: {error}"))
            })?;
        }

        let content = serde_json::to_string_pretty(cassette).map_err(|error| {
            RecordReplayError::new(
                &self.cassette_path,
                format!("cassette could not be serialized: {error}"),
            )
        })?;

        fs::write(&self.cassette_path, content).map_err(|error| {
            RecordReplayError::new(
                &self.cassette_path,
                format!("cassette could not be written: {error}"),
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RecordedResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct RecordedInteraction {
    pub request: RecordedRequest,
    pub response: RecordedResponse,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CassetteFile {
    interactions: Vec<RecordedInteraction>,
}

#[derive(Debug, Clone)]
pub(crate) struct RecordReplayError {
    message: String,
}

impl RecordReplayError {
    fn new(path: &Path, message: String) -> Self {
        Self {
            message: format!("{} ({})", path.display(), message),
        }
    }
}

impl Display for RecordReplayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Process-wide lock serializing every test that mutates the global `VCR_MODE` environment
/// variable. All integration modules (Daylite and Planradar) share this single lock so their
/// env-mutating tests cannot race each other across Cargo's parallel test threads (e.g. one
/// module flipping `VCR_MODE=record` while another relies on replay mode).
pub(crate) fn vcr_env_lock() -> &'static std::sync::Mutex<()> {
    static VCR_ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    VCR_ENV_LOCK.get_or_init(|| std::sync::Mutex::new(()))
}
