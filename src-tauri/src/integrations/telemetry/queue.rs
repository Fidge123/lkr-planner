use super::events::TelemetryEvent;
use super::recorder::TelemetryCommand;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

pub const BATCH_SIZE: usize = 20;
pub const MAX_BUFFERED_EVENTS: usize = 500;
pub const FLUSH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryError;

pub trait EventSink: Send + Sync + 'static {
    fn deliver(
        &self,
        batch: Vec<TelemetryEvent>,
    ) -> impl Future<Output = Result<(), DeliveryError>> + Send;
}

#[derive(Debug, Default)]
pub struct EventBuffer {
    events: VecDeque<TelemetryEvent>,
}

impl EventBuffer {
    pub fn push(&mut self, event: TelemetryEvent) {
        if self.events.len() == MAX_BUFFERED_EVENTS {
            self.events.pop_front();
        }

        self.events.push_back(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn is_batch_ready(&self) -> bool {
        self.events.len() >= BATCH_SIZE
    }

    pub fn take_batch(&mut self) -> Vec<TelemetryEvent> {
        let size = self.events.len().min(BATCH_SIZE);
        self.events.drain(..size).collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub async fn run_flush_task<S: EventSink>(mut receiver: Receiver<TelemetryCommand>, sink: Arc<S>) {
    let mut buffer = EventBuffer::default();
    let mut retry: Option<Vec<TelemetryEvent>> = None;
    let mut ticker = tokio::time::interval(FLUSH_INTERVAL);
    ticker.tick().await;

    loop {
        tokio::select! {
            command = receiver.recv() => match command {
                Some(TelemetryCommand::Record(event)) => {
                    buffer.push(*event);
                    if buffer.is_batch_ready() {
                        flush(&mut buffer, &mut retry, &sink).await;
                    }
                }
                Some(TelemetryCommand::DiscardPending) => {
                    buffer.clear();
                    retry = None;
                }
                Some(TelemetryCommand::Flush) => flush(&mut buffer, &mut retry, &sink).await,
                Some(TelemetryCommand::Shutdown(acknowledgement)) => {
                    flush(&mut buffer, &mut retry, &sink).await;
                    let _ = acknowledgement.send(());
                    return;
                }
                None => break,
            },
            _ = ticker.tick() => flush(&mut buffer, &mut retry, &sink).await,
        }
    }

    flush(&mut buffer, &mut retry, &sink).await;
}

/// A batch that failed once is attempted a second time and then dropped: stale
/// measurements are not worth a persistent retry queue.
async fn flush<S: EventSink>(
    buffer: &mut EventBuffer,
    retry: &mut Option<Vec<TelemetryEvent>>,
    sink: &Arc<S>,
) {
    if let Some(batch) = retry.take() {
        let _ = sink.deliver(batch).await;
    }

    if buffer.is_empty() {
        return;
    }

    let batch = buffer.take_batch();
    if sink.deliver(batch.clone()).await.is_err() {
        *retry = Some(batch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::telemetry::events::{Integration, Operation};
    use crate::integrations::telemetry::recorder::TelemetryRecorder;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    fn event(duration_ms: u64) -> TelemetryEvent {
        TelemetryEvent::operation_completed(
            Operation::LoadWeekEvents,
            Integration::Zep,
            true,
            duration_ms,
        )
    }

    #[derive(Default)]
    struct RecordingSink {
        batches: Mutex<Vec<Vec<TelemetryEvent>>>,
        failures: AtomicUsize,
    }

    impl RecordingSink {
        fn failing(times: usize) -> Self {
            Self {
                batches: Mutex::new(Vec::new()),
                failures: AtomicUsize::new(times),
            }
        }

        fn batches(&self) -> Vec<Vec<TelemetryEvent>> {
            self.batches.lock().unwrap().clone()
        }
    }

    impl EventSink for RecordingSink {
        async fn deliver(&self, batch: Vec<TelemetryEvent>) -> Result<(), DeliveryError> {
            self.batches.lock().unwrap().push(batch);
            if self.failures.load(Ordering::SeqCst) > 0 {
                self.failures.fetch_sub(1, Ordering::SeqCst);
                return Err(DeliveryError);
            }

            Ok(())
        }
    }

    #[test]
    fn buffer_drops_oldest_events_at_the_cap() {
        let mut buffer = EventBuffer::default();

        for index in 0..(MAX_BUFFERED_EVENTS + 5) {
            buffer.push(event(index as u64));
        }

        assert_eq!(buffer.len(), MAX_BUFFERED_EVENTS);
        let batch = buffer.take_batch();
        let first_duration = batch[0].properties().get("duration_ms").unwrap().as_u64();
        assert_eq!(first_duration, Some(5));
    }

    #[test]
    fn buffer_reports_ready_at_the_batch_size() {
        let mut buffer = EventBuffer::default();

        for index in 0..(BATCH_SIZE - 1) {
            buffer.push(event(index as u64));
        }
        assert!(!buffer.is_batch_ready());

        buffer.push(event(99));
        assert!(buffer.is_batch_ready());
    }

    #[test]
    fn buffer_takes_at_most_one_batch() {
        let mut buffer = EventBuffer::default();
        for index in 0..(BATCH_SIZE + 7) {
            buffer.push(event(index as u64));
        }

        let batch = buffer.take_batch();

        assert_eq!(batch.len(), BATCH_SIZE);
        assert_eq!(buffer.len(), 7);
    }

    #[tokio::test]
    async fn flushes_when_the_batch_size_is_reached() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::default());

        for index in 0..BATCH_SIZE {
            recorder.record(event(index as u64));
        }
        drop(recorder);
        run_flush_task(receiver, Arc::clone(&sink)).await;

        let batches = sink.batches();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), BATCH_SIZE);
    }

    #[tokio::test(start_paused = true)]
    async fn flushes_on_the_interval_tick() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::default());
        recorder.record(event(1));

        let task = tokio::spawn(run_flush_task(receiver, Arc::clone(&sink)));
        tokio::time::sleep(FLUSH_INTERVAL + Duration::from_secs(1)).await;
        drop(recorder);
        task.await.expect("flush task should finish");

        assert_eq!(
            sink.batches().len(),
            1,
            "the interval tick delivers the buffered event before shutdown"
        );
    }

    #[tokio::test]
    async fn discard_pending_drops_buffered_events_without_sending() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::default());
        recorder.record(event(1));

        recorder.set_enabled(false);
        drop(recorder);
        run_flush_task(receiver, Arc::clone(&sink)).await;

        assert!(sink.batches().is_empty());
    }

    #[tokio::test]
    async fn failed_delivery_is_retried_once_and_then_dropped() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::failing(1));
        recorder.record(event(1));
        recorder.request_flush();
        recorder.request_flush();
        drop(recorder);

        run_flush_task(receiver, Arc::clone(&sink)).await;

        let batches = sink.batches();
        assert_eq!(batches.len(), 2, "the failed batch is retried exactly once");
        assert_eq!(batches[0], batches[1]);
    }

    #[tokio::test]
    async fn shutdown_delivers_pending_events_and_acknowledges() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::default());
        recorder.record(event(1));

        let acknowledgement = recorder.shutdown();
        run_flush_task(receiver, Arc::clone(&sink)).await;

        assert_eq!(sink.batches().len(), 1);
        assert!(acknowledgement.await.is_ok());
    }

    #[tokio::test]
    async fn delivery_failure_never_reaches_the_recording_caller() {
        let (recorder, receiver) = TelemetryRecorder::channel();
        recorder.apply_settings(&enabled());
        let sink = Arc::new(RecordingSink::failing(usize::MAX));

        for index in 0..(MAX_BUFFERED_EVENTS * 2) {
            recorder.record(event(index as u64));
        }
        assert!(recorder.is_enabled(), "recording survives a failing sink");
        drop(recorder);

        run_flush_task(receiver, Arc::clone(&sink)).await;

        assert!(sink.batches().iter().all(|batch| batch.len() <= BATCH_SIZE));
    }

    fn enabled() -> crate::integrations::local_store::types::TelemetrySettings {
        crate::integrations::local_store::types::TelemetrySettings {
            enabled: true,
            install_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
        }
    }
}
