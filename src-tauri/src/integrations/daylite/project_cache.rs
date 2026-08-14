use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use super::projects::ResolvedProject;

const PROJECT_CACHE_TTL_MS: u64 = 30_000;

struct CacheEntry {
    project: ResolvedProject,
    fetched_at_ms: u64,
}

/// One slot per project reference. Holding the slot across the load is what makes
/// concurrent callers for the same reference share a single request instead of
/// each dispatching their own.
type Slot = Arc<tokio::sync::Mutex<Option<CacheEntry>>>;

pub(super) struct ProjectCache {
    ttl_ms: u64,
    slots: Mutex<HashMap<String, Slot>>,
}

impl ProjectCache {
    fn new(ttl_ms: u64) -> Self {
        Self {
            ttl_ms,
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub(super) async fn get_or_load<F, Fut>(
        &self,
        project_ref: &str,
        now_ms: u64,
        load: F,
    ) -> Option<ResolvedProject>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<ResolvedProject>>,
    {
        let slot = self.slot_for(project_ref);
        let mut entry = slot.lock().await;

        if let Some(cached) = entry.as_ref() {
            if now_ms.saturating_sub(cached.fetched_at_ms) < self.ttl_ms {
                return Some(cached.project.clone());
            }
        }

        // A failed load stays uncached so a rate-limited or timed-out reference
        // resolves again on the next week load instead of degrading for a whole TTL.
        let project = load().await?;
        *entry = Some(CacheEntry {
            project: project.clone(),
            fetched_at_ms: now_ms,
        });

        Some(project)
    }

    fn slot_for(&self, project_ref: &str) -> Slot {
        let mut slots = self.slots.lock().expect("project cache slots poisoned");
        slots.entry(project_ref.to_string()).or_default().clone()
    }
}

pub(super) fn project_cache() -> &'static ProjectCache {
    static CACHE: OnceLock<ProjectCache> = OnceLock::new();
    CACHE.get_or_init(|| ProjectCache::new(PROJECT_CACHE_TTL_MS))
}

/// Monotonic so a system clock adjustment cannot make an entry look fresh forever.
pub(super) fn cache_now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    u64::try_from(START.get_or_init(Instant::now).elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn project(name: &str) -> ResolvedProject {
        ResolvedProject {
            name: name.to_string(),
            status: "in_progress".to_string(),
            category: None,
        }
    }

    #[tokio::test]
    async fn serves_a_fresh_entry_without_loading_again() {
        let cache = ProjectCache::new(30_000);
        let calls = AtomicUsize::new(0);

        for _ in 0..3 {
            let resolved = cache
                .get_or_load("/v1/projects/1", 1_000, || async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Some(project("Projekt Nord"))
                })
                .await;
            assert_eq!(resolved, Some(project("Projekt Nord")));
        }

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn reloads_once_the_entry_is_older_than_the_ttl() {
        let cache = ProjectCache::new(30_000);
        let calls = AtomicUsize::new(0);

        cache
            .get_or_load("/v1/projects/1", 0, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Some(project("Alt"))
            })
            .await;

        let resolved = cache
            .get_or_load("/v1/projects/1", 30_000, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Some(project("Neu"))
            })
            .await;

        assert_eq!(resolved, Some(project("Neu")));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_callers_for_one_reference_load_once() {
        let cache = ProjectCache::new(30_000);
        let calls = AtomicUsize::new(0);

        let load = || async {
            calls.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            Some(project("Projekt Nord"))
        };

        let (first, second) = tokio::join!(
            cache.get_or_load("/v1/projects/1", 1_000, load),
            cache.get_or_load("/v1/projects/1", 1_000, load),
        );

        assert_eq!(first, Some(project("Projekt Nord")));
        assert_eq!(second, Some(project("Projekt Nord")));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_failed_load_is_not_cached() {
        let cache = ProjectCache::new(30_000);
        let calls = AtomicUsize::new(0);

        let failed = cache
            .get_or_load("/v1/projects/1", 1_000, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                None
            })
            .await;
        assert_eq!(failed, None);

        let retried = cache
            .get_or_load("/v1/projects/1", 1_000, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Some(project("Projekt Nord"))
            })
            .await;

        assert_eq!(retried, Some(project("Projekt Nord")));
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn distinct_references_do_not_share_an_entry() {
        let cache = ProjectCache::new(30_000);

        cache
            .get_or_load("/v1/projects/1", 1_000, || async { Some(project("Eins")) })
            .await;
        let second = cache
            .get_or_load("/v1/projects/2", 1_000, || async { Some(project("Zwei")) })
            .await;

        assert_eq!(second, Some(project("Zwei")));
    }
}
