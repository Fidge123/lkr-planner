use super::client::PostHogClient;
use super::context::EventContext;
use super::events::TelemetryEvent;
use super::queue::{run_flush_task, DeliveryError, EventSink};
use super::recorder::TelemetryRecorder;
use crate::integrations::local_store::types::TelemetrySettings;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct TelemetryState {
    recorder: Arc<TelemetryRecorder>,
}

impl TelemetryState {
    /// The returned task ends when the state is dropped or shutdown is requested.
    pub fn start() -> (Self, impl std::future::Future<Output = ()> + Send) {
        let (recorder, receiver) = TelemetryRecorder::channel();
        let recorder = Arc::new(recorder);
        let sink = Arc::new(PostHogSink {
            client: PostHogClient::from_build_config(),
            recorder: Arc::clone(&recorder),
        });

        (
            Self {
                recorder: Arc::clone(&recorder),
            },
            run_flush_task(receiver, sink),
        )
    }

    pub fn recorder(&self) -> &Arc<TelemetryRecorder> {
        &self.recorder
    }

    pub fn apply_settings(&self, settings: &TelemetrySettings) {
        self.recorder.apply_settings(settings);
    }

    pub fn record(&self, event: TelemetryEvent) {
        self.recorder.record(event);
    }

    pub fn shutdown(&self) -> oneshot::Receiver<()> {
        self.recorder.shutdown()
    }
}

struct PostHogSink {
    client: PostHogClient,
    recorder: Arc<TelemetryRecorder>,
}

impl EventSink for PostHogSink {
    async fn deliver(&self, batch: Vec<TelemetryEvent>) -> Result<(), DeliveryError> {
        let Some(install_id) = self.recorder.install_id() else {
            return Ok(());
        };

        self.client
            .send(batch, &EventContext::current(install_id))
            .await
    }
}
