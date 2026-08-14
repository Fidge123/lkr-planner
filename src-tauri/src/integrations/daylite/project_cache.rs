use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use super::projects::ResolvedProject;

const PROJECT_CACHE_TTL_MS: u64 = 30_000;
/// Above this, expired slots are swept before another one is added.
const PROJECT_CACHE_SWEEP_ABOVE: usize = 512;

struct CacheEntry {
    project: ResolvedProject,
    fetched_at_ms: u64,
}

/// Held across the load, so concurrent callers for one reference share a single request.
type Slot = Arc<tokio::sync::Mutex<Option<CacheEntry>>>;

pub(super) struct ProjectCache {
    ttl_ms: u64,
    sweep_above: usize,
    slots: Mutex<HashMap<String, Slot>>,
}

impl ProjectCache {
    fn with_bound(ttl_ms: u64, sweep_above: usize) -> Self {
        Self {
            ttl_ms,
            sweep_above,
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
        let slot = self.slot_for(project_ref, now_ms);
        let mut entry = slot.lock().await;

        if let Some(cached) = entry.as_ref() {
            if now_ms.saturating_sub(cached.fetched_at_ms) < self.ttl_ms {
                return Some(cached.project.clone());
            }
        }

        // A cached failure would degrade the card for a whole TTL, so only successes are stored.
        let project = load().await?;
        *entry = Some(CacheEntry {
            project: project.clone(),
            fetched_at_ms: now_ms,
        });

        Some(project)
    }

    fn slot_for(&self, project_ref: &str, now_ms: u64) -> Slot {
        let mut slots = self.slots.lock().expect("project cache slots poisoned");
        if slots.len() > self.sweep_above && !slots.contains_key(project_ref) {
            sweep_expired(&mut slots, now_ms, self.ttl_ms);
        }

        slots.entry(project_ref.to_string()).or_default().clone()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.slots
            .lock()
            .expect("project cache slots poisoned")
            .len()
    }
}

/// A slot another task holds is skipped: `try_lock` failing means a load is in flight.
fn sweep_expired(slots: &mut HashMap<String, Slot>, now_ms: u64, ttl_ms: u64) {
    slots.retain(|_, slot| match slot.try_lock() {
        Ok(entry) => entry
            .as_ref()
            .is_some_and(|cached| now_ms.saturating_sub(cached.fetched_at_ms) < ttl_ms),
        Err(_) => true,
    });
}

pub(super) fn project_cache() -> &'static ProjectCache {
    static CACHE: OnceLock<ProjectCache> = OnceLock::new();
    CACHE.get_or_init(|| ProjectCache::with_bound(PROJECT_CACHE_TTL_MS, PROJECT_CACHE_SWEEP_ABOVE))
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
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
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
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
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
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
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
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
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
    async fn expired_entries_are_swept_once_the_map_grows_past_its_bound() {
        let cache = ProjectCache::with_bound(30_000, 4);

        for id in 0..8 {
            cache
                .get_or_load(&format!("/v1/projects/{id}"), 1_000, || async {
                    Some(project("Alt"))
                })
                .await;
        }
        assert_eq!(cache.len(), 8);

        cache
            .get_or_load("/v1/projects/8", 61_000, || async { Some(project("Neu")) })
            .await;

        assert_eq!(cache.len(), 1);
    }

    #[tokio::test]
    async fn a_sweep_keeps_entries_that_are_still_fresh() {
        let cache = ProjectCache::with_bound(30_000, 1);

        cache
            .get_or_load("/v1/projects/1", 1_000, || async { Some(project("Eins")) })
            .await;
        cache
            .get_or_load("/v1/projects/2", 2_000, || async { Some(project("Zwei")) })
            .await;
        cache
            .get_or_load("/v1/projects/3", 3_000, || async { Some(project("Drei")) })
            .await;

        assert_eq!(cache.len(), 3);

        let calls = AtomicUsize::new(0);
        let resolved = cache
            .get_or_load("/v1/projects/1", 4_000, || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Some(project("Neu"))
            })
            .await;

        assert_eq!(resolved, Some(project("Eins")));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn distinct_references_do_not_share_an_entry() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);

        cache
            .get_or_load("/v1/projects/1", 1_000, || async { Some(project("Eins")) })
            .await;
        let second = cache
            .get_or_load("/v1/projects/2", 1_000, || async { Some(project("Zwei")) })
            .await;

        assert_eq!(second, Some(project("Zwei")));
    }
}
