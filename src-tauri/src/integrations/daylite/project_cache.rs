use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use tauri::Manager;

use super::projects::ResolvedProject;

const PROJECT_CACHE_TTL_MS: u64 = 300_000;
/// Above this, stale slots are swept before another one is added.
const PROJECT_CACHE_SWEEP_ABOVE: usize = 4096;
const PROJECT_CACHE_FILE_NAME: &str = "daylite-projects.json";

struct CacheEntry {
    project: ResolvedProject,
    /// None for an entry restored from disk, which is stale until reloaded.
    fetched_at_ms: Option<u64>,
}

#[derive(Default)]
struct Slot {
    known: Mutex<Option<CacheEntry>>,
    /// Held across the load, so concurrent callers for one reference share a single request.
    load: tokio::sync::Mutex<()>,
}

pub(super) struct ProjectCache {
    ttl_ms: u64,
    sweep_above: usize,
    slots: Mutex<HashMap<String, Arc<Slot>>>,
}

impl ProjectCache {
    fn with_bound(ttl_ms: u64, sweep_above: usize) -> Self {
        Self {
            ttl_ms,
            sweep_above,
            slots: Mutex::new(HashMap::new()),
        }
    }

    /// Serves an entry of any age without loading.
    pub(super) fn peek(&self, project_ref: &str) -> Option<ResolvedProject> {
        let slot = self.existing_slot(project_ref)?;
        let known = slot.known.lock().expect("project cache slot poisoned");
        known.as_ref().map(|entry| entry.project.clone())
    }

    pub(super) fn is_fresh(&self, project_ref: &str, now_ms: u64) -> bool {
        self.existing_slot(project_ref).is_some_and(|slot| {
            let known = slot.known.lock().expect("project cache slot poisoned");
            is_fresh(known.as_ref(), now_ms, self.ttl_ms)
        })
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
        self.resolve(project_ref, now_ms, false, load).await
    }

    pub(super) async fn reload<F, Fut>(
        &self,
        project_ref: &str,
        now_ms: u64,
        load: F,
    ) -> Option<ResolvedProject>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<ResolvedProject>>,
    {
        self.resolve(project_ref, now_ms, true, load).await
    }

    /// A failed load serves the known entry, so a transient error keeps the card resolved.
    async fn resolve<F, Fut>(
        &self,
        project_ref: &str,
        now_ms: u64,
        force: bool,
        load: F,
    ) -> Option<ResolvedProject>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<ResolvedProject>>,
    {
        let slot = self.slot_for(project_ref, now_ms);
        let _loading = slot.load.lock().await;

        if !force {
            let known = slot.known.lock().expect("project cache slot poisoned");
            if is_fresh(known.as_ref(), now_ms, self.ttl_ms) {
                return known.as_ref().map(|entry| entry.project.clone());
            }
        }

        let loaded = load().await;
        let mut known = slot.known.lock().expect("project cache slot poisoned");
        if let Some(project) = loaded {
            *known = Some(CacheEntry {
                project,
                fetched_at_ms: Some(now_ms),
            });
        }
        known.as_ref().map(|entry| entry.project.clone())
    }

    /// Fills only empty slots, so a project loaded in this session is never replaced by its saved copy.
    pub(super) fn restore(&self, projects: impl IntoIterator<Item = (String, ResolvedProject)>) {
        let mut slots = self.slots.lock().expect("project cache slots poisoned");
        for (project_ref, project) in projects {
            let slot = slots.entry(project_ref).or_default();
            let mut known = slot.known.lock().expect("project cache slot poisoned");
            if known.is_none() {
                *known = Some(CacheEntry {
                    project,
                    fetched_at_ms: None,
                });
            }
        }
    }

    pub(super) fn snapshot(&self) -> BTreeMap<String, ResolvedProject> {
        let slots = self.slots.lock().expect("project cache slots poisoned");
        slots
            .iter()
            .filter_map(|(project_ref, slot)| {
                let known = slot.known.lock().expect("project cache slot poisoned");
                known
                    .as_ref()
                    .map(|entry| (project_ref.clone(), entry.project.clone()))
            })
            .collect()
    }

    fn existing_slot(&self, project_ref: &str) -> Option<Arc<Slot>> {
        let slots = self.slots.lock().expect("project cache slots poisoned");
        slots.get(project_ref).cloned()
    }

    fn slot_for(&self, project_ref: &str, now_ms: u64) -> Arc<Slot> {
        let mut slots = self.slots.lock().expect("project cache slots poisoned");
        if slots.len() > self.sweep_above && !slots.contains_key(project_ref) {
            sweep_stale(&mut slots, now_ms, self.ttl_ms);
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

fn is_fresh(entry: Option<&CacheEntry>, now_ms: u64, ttl_ms: u64) -> bool {
    entry
        .and_then(|entry| entry.fetched_at_ms)
        .is_some_and(|fetched_at_ms| now_ms.saturating_sub(fetched_at_ms) < ttl_ms)
}

/// Evicting a slot a caller holds orphans its load, so only the map's own reference may remain.
/// `slot_for` hands out the reference before locking, so a lock check alone misses it.
fn sweep_stale(slots: &mut HashMap<String, Arc<Slot>>, now_ms: u64, ttl_ms: u64) {
    slots.retain(|_, slot| {
        if Arc::strong_count(slot) > 1 {
            return true;
        }

        match slot.known.try_lock() {
            Ok(known) => is_fresh(known.as_ref(), now_ms, ttl_ms),
            Err(_) => true,
        }
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

/// The cache only spares requests, so a missing or unreadable file starts it empty.
pub(crate) fn restore_project_cache(app: &tauri::AppHandle) {
    if let Some(projects) = cache_file_path(app).and_then(|path| read_cache_file(&path)) {
        project_cache().restore(projects);
    }
}

pub(super) fn persist_project_cache(app: &tauri::AppHandle) {
    let Some(path) = cache_file_path(app) else {
        return;
    };
    if let Err(error) = write_cache_file(&path, &project_cache().snapshot()) {
        eprintln!(
            "Daylite-Projektcache konnte nicht gespeichert werden ({}): {error}",
            path.display()
        );
    }
}

fn cache_file_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_cache_dir()
        .ok()
        .map(|dir| dir.join(PROJECT_CACHE_FILE_NAME))
}

fn read_cache_file(path: &Path) -> Option<BTreeMap<String, ResolvedProject>> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cache_file(
    path: &Path,
    projects: &BTreeMap<String, ResolvedProject>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string(projects)?)
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
    async fn a_sweep_keeps_a_slot_a_caller_still_holds() {
        let held = Arc::new(Slot {
            known: Mutex::new(Some(CacheEntry {
                project: project("Alt"),
                fetched_at_ms: Some(0),
            })),
            load: tokio::sync::Mutex::new(()),
        });
        let mut slots = HashMap::from([("/v1/projects/1".to_string(), held.clone())]);

        sweep_stale(&mut slots, 61_000, 30_000);
        assert_eq!(slots.len(), 1);

        drop(held);
        sweep_stale(&mut slots, 61_000, 30_000);
        assert!(slots.is_empty());
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

    #[tokio::test]
    async fn a_stale_entry_is_served_when_its_refresh_fails() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Alt")) })
            .await;

        let resolved = cache
            .get_or_load("/v1/projects/1", 60_000, || async { None })
            .await;

        assert_eq!(resolved, Some(project("Alt")));
    }

    #[tokio::test]
    async fn peek_returns_an_entry_of_any_age_without_loading() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Alt")) })
            .await;

        assert_eq!(cache.peek("/v1/projects/1"), Some(project("Alt")));
        assert_eq!(cache.peek("/v1/projects/2"), None);
    }

    #[tokio::test]
    async fn freshness_follows_the_ttl() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 1_000, || async { Some(project("Nord")) })
            .await;

        assert!(cache.is_fresh("/v1/projects/1", 30_999));
        assert!(!cache.is_fresh("/v1/projects/1", 31_000));
        assert!(!cache.is_fresh("/v1/projects/2", 1_000));
    }

    #[tokio::test]
    async fn restored_entries_are_served_by_peek_but_stale() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache.restore([("/v1/projects/1".to_string(), project("Gespeichert"))]);

        assert_eq!(cache.peek("/v1/projects/1"), Some(project("Gespeichert")));
        assert!(!cache.is_fresh("/v1/projects/1", 0));

        let resolved = cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Neu")) })
            .await;
        assert_eq!(resolved, Some(project("Neu")));
    }

    #[tokio::test]
    async fn restore_does_not_replace_a_loaded_entry() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Geladen")) })
            .await;

        cache.restore([("/v1/projects/1".to_string(), project("Gespeichert"))]);

        assert_eq!(cache.peek("/v1/projects/1"), Some(project("Geladen")));
        assert!(cache.is_fresh("/v1/projects/1", 0));
    }

    #[tokio::test]
    async fn a_forced_reload_bypasses_a_fresh_entry() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Alt")) })
            .await;

        let resolved = cache
            .reload("/v1/projects/1", 1_000, || async { Some(project("Neu")) })
            .await;

        assert_eq!(resolved, Some(project("Neu")));
    }

    #[tokio::test]
    async fn a_failed_forced_reload_keeps_the_known_entry() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Alt")) })
            .await;

        let resolved = cache
            .reload("/v1/projects/1", 1_000, || async { None })
            .await;

        assert_eq!(resolved, Some(project("Alt")));
    }

    #[tokio::test]
    async fn snapshot_lists_every_known_project() {
        let cache = ProjectCache::with_bound(30_000, usize::MAX);
        cache.restore([("/v1/projects/2".to_string(), project("Zwei"))]);
        cache
            .get_or_load("/v1/projects/1", 0, || async { Some(project("Eins")) })
            .await;
        cache
            .get_or_load("/v1/projects/3", 0, || async { None })
            .await;

        let snapshot = cache.snapshot();

        assert_eq!(
            snapshot,
            BTreeMap::from([
                ("/v1/projects/1".to_string(), project("Eins")),
                ("/v1/projects/2".to_string(), project("Zwei")),
            ])
        );
    }

    #[test]
    fn a_written_cache_file_reads_back() {
        let path = temp_cache_path("round-trip");
        let projects = BTreeMap::from([("/v1/projects/1".to_string(), project("Eins"))]);

        write_cache_file(&path, &projects).expect("cache file should be written");

        assert_eq!(read_cache_file(&path), Some(projects));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn an_unreadable_cache_file_is_ignored() {
        let path = temp_cache_path("corrupt");
        std::fs::write(&path, "{ nicht json").expect("corrupt file should be written");

        assert_eq!(read_cache_file(&path), None);
        assert_eq!(read_cache_file(&temp_cache_path("missing")), None);
        let _ = std::fs::remove_file(&path);
    }

    fn temp_cache_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "lkr-planner-project-cache-{label}-{}.json",
            std::process::id()
        ))
    }
}
