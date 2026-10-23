use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{OnceLock, RwLock};

use super::shared::{
    store_daylite_tokens, DayliteApiError, DayliteApiErrorCode, DayliteTokenState,
};

/// A leased token has to outlive the whole operation: nothing refreshes mid-request.
const LEASE_MIN_REMAINING_MS: u64 = 60_000;

type Persist = Box<dyn Fn(&DayliteTokenState) -> Result<(), DayliteApiError> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TokenLease {
    pub(super) tokens: DayliteTokenState,
    generation: u64,
}

struct SessionState {
    tokens: DayliteTokenState,
    generation: u64,
    persisted: bool,
}

/// Only rotation is serialized: Daylite invalidates a refresh token as it issues the next.
/// A request carrying an already valid access token is safe in parallel.
pub(super) struct TokenSession {
    state: RwLock<Option<SessionState>>,
    rotation: tokio::sync::Mutex<()>,
    min_remaining_ms: u64,
    persist: Persist,
    closing: AtomicBool,
}

impl TokenSession {
    fn new(min_remaining_ms: u64, persist: Persist) -> Self {
        Self {
            state: RwLock::new(None),
            rotation: tokio::sync::Mutex::new(()),
            min_remaining_ms,
            persist,
            closing: AtomicBool::new(false),
        }
    }

    pub(super) fn seed(&self, tokens: DayliteTokenState) {
        let mut state = self.state.write().expect("token session poisoned");
        if state.is_none() {
            *state = Some(SessionState {
                tokens,
                generation: 0,
                persisted: true,
            });
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.state.read().expect("token session poisoned").is_none()
    }

    pub(super) async fn lease<F, Fut>(
        &self,
        now_ms: u64,
        rotate: F,
    ) -> Result<TokenLease, DayliteApiError>
    where
        F: Fn(DayliteTokenState) -> Fut,
        Fut: Future<Output = Result<DayliteTokenState, DayliteApiError>>,
    {
        if let Some(lease) = self.fresh_lease(now_ms) {
            return Ok(lease);
        }

        let _guard = self.rotation.lock().await;
        self.persist_pending()?;
        // Another task may have rotated while this one waited for the lock.
        if let Some(lease) = self.fresh_lease(now_ms) {
            return Ok(lease);
        }

        self.rotate_locked(rotate).await
    }

    /// A burst of rejections costs one rotation: the rest take the newer generation.
    pub(super) async fn renew<F, Fut>(
        &self,
        stale: &TokenLease,
        rotate: F,
    ) -> Result<TokenLease, DayliteApiError>
    where
        F: Fn(DayliteTokenState) -> Fut,
        Fut: Future<Output = Result<DayliteTokenState, DayliteApiError>>,
    {
        let _guard = self.rotation.lock().await;
        self.persist_pending()?;
        if let Some(lease) = self.lease_newer_than(stale.generation) {
            return Ok(lease);
        }

        self.rotate_locked(rotate).await
    }

    /// Takes the rotation lock so a connect cannot interleave with one.
    pub(super) async fn adopt<F, Fut>(&self, mint: F) -> Result<DayliteTokenState, DayliteApiError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<DayliteTokenState, DayliteApiError>>,
    {
        let _guard = self.rotation.lock().await;
        self.ensure_open()?;
        let tokens = mint().await?;
        Ok(self.publish(tokens)?.tokens)
    }

    /// Exiting mid-rotation would strand the keychain on a refresh token Daylite already retired.
    /// Flag before locking so queued waiters cannot start a rotation.
    pub(super) async fn close(&self) -> Result<(), DayliteApiError> {
        self.closing.store(true, Ordering::SeqCst);
        let _guard = self.rotation.lock().await;
        self.persist_pending()
    }

    fn ensure_open(&self) -> Result<(), DayliteApiError> {
        if !self.closing.load(Ordering::SeqCst) {
            return Ok(());
        }

        Err(DayliteApiError::new(
            DayliteApiErrorCode::RequestFailed,
            "Die Anwendung wird beendet.",
            "Daylite-Token-Rotation nach Beginn des Beendens abgelehnt.",
        ))
    }

    async fn rotate_locked<F, Fut>(&self, rotate: F) -> Result<TokenLease, DayliteApiError>
    where
        F: Fn(DayliteTokenState) -> Fut,
        Fut: Future<Output = Result<DayliteTokenState, DayliteApiError>>,
    {
        self.ensure_open()?;
        let current = self.current_tokens().unwrap_or_default();
        let rotated = rotate(current).await?;
        self.publish(rotated)
    }

    /// Held in memory before persisting: once rotated, the previous refresh token is dead server-side.
    fn publish(&self, tokens: DayliteTokenState) -> Result<TokenLease, DayliteApiError> {
        let lease = {
            let mut state = self.state.write().expect("token session poisoned");
            let generation = state.as_ref().map(|s| s.generation + 1).unwrap_or(1);
            *state = Some(SessionState {
                tokens: tokens.clone(),
                generation,
                persisted: false,
            });
            TokenLease { tokens, generation }
        };

        self.persist_pending()?;
        Ok(lease)
    }

    /// Callers hold the rotation lock, so writes cannot land out of order.
    fn persist_pending(&self) -> Result<(), DayliteApiError> {
        let Some(pending) = self.lease_if(|state| !state.persisted) else {
            return Ok(());
        };
        (self.persist)(&pending.tokens)?;

        let mut state = self.state.write().expect("token session poisoned");
        if let Some(state) = state
            .as_mut()
            .filter(|state| state.generation == pending.generation)
        {
            state.persisted = true;
        }
        Ok(())
    }

    fn current_tokens(&self) -> Option<DayliteTokenState> {
        let state = self.state.read().expect("token session poisoned");
        state.as_ref().map(|s| s.tokens.clone())
    }

    fn fresh_lease(&self, now_ms: u64) -> Option<TokenLease> {
        self.lease_if(|state| {
            state.persisted && has_life_left(&state.tokens, now_ms, self.min_remaining_ms)
        })
    }

    fn lease_newer_than(&self, generation: u64) -> Option<TokenLease> {
        self.lease_if(|state| state.generation > generation)
    }

    fn lease_if(&self, accept: impl Fn(&SessionState) -> bool) -> Option<TokenLease> {
        let state = self.state.read().expect("token session poisoned");
        let state = state.as_ref().filter(|state| accept(state))?;

        Some(TokenLease {
            tokens: state.tokens.clone(),
            generation: state.generation,
        })
    }
}

fn has_life_left(tokens: &DayliteTokenState, now_ms: u64, min_remaining_ms: u64) -> bool {
    if tokens.access_token.trim().is_empty() {
        return false;
    }

    match tokens.access_token_expires_at_ms {
        Some(expires_at_ms) => expires_at_ms > now_ms.saturating_add(min_remaining_ms),
        None => false,
    }
}

pub(super) fn token_session() -> &'static TokenSession {
    static SESSION: OnceLock<TokenSession> = OnceLock::new();
    SESSION
        .get_or_init(|| TokenSession::new(LEASE_MIN_REMAINING_MS, Box::new(store_daylite_tokens)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Mutex};

    fn tokens(access: &str, expires_at_ms: Option<u64>) -> DayliteTokenState {
        DayliteTokenState {
            access_token: access.to_string(),
            refresh_token: "rt".to_string(),
            access_token_expires_at_ms: expires_at_ms,
        }
    }

    fn session_with(seeded: DayliteTokenState) -> TokenSession {
        session_backed_by(seeded, Arc::new(Keychain::default()))
    }

    #[derive(Default)]
    struct Keychain {
        stored: Mutex<Vec<String>>,
        refuses: AtomicBool,
    }

    impl Keychain {
        fn stored(&self) -> Vec<String> {
            self.stored.lock().unwrap().clone()
        }
    }

    fn session_backed_by(seeded: DayliteTokenState, keychain: Arc<Keychain>) -> TokenSession {
        let session = TokenSession::new(
            60_000,
            Box::new(move |tokens: &DayliteTokenState| {
                if keychain.refuses.load(Ordering::SeqCst) {
                    return Err(DayliteApiError::new(
                        DayliteApiErrorCode::ServerError,
                        "Keychain verweigert",
                        "denied",
                    ));
                }
                keychain
                    .stored
                    .lock()
                    .unwrap()
                    .push(tokens.access_token.clone());
                Ok(())
            }),
        );
        session.seed(seeded);
        session
    }

    #[tokio::test]
    async fn leases_from_memory_while_the_token_has_life_left() {
        let session = session_with(tokens("at", Some(500_000)));
        let rotations = AtomicUsize::new(0);

        let lease = session
            .lease(100_000, |_| async {
                rotations.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("rotated", Some(900_000)))
            })
            .await
            .expect("lease should succeed");

        assert_eq!(lease.tokens.access_token, "at");
        assert_eq!(rotations.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn rotates_when_the_token_is_inside_the_lease_margin() {
        let session = session_with(tokens("at", Some(150_000)));

        let lease = session
            .lease(100_000, |_| async { Ok(tokens("rotated", Some(900_000))) })
            .await
            .expect("lease should succeed");

        assert_eq!(lease.tokens.access_token, "rotated");
    }

    #[tokio::test]
    async fn rotates_when_no_expiry_is_known() {
        let session = session_with(tokens("at", None));

        let lease = session
            .lease(100_000, |_| async { Ok(tokens("rotated", Some(900_000))) })
            .await
            .expect("lease should succeed");

        assert_eq!(lease.tokens.access_token, "rotated");
    }

    #[tokio::test]
    async fn concurrent_leases_rotate_once() {
        let session = session_with(tokens("", None));
        let rotations = AtomicUsize::new(0);

        let rotate = |_| async {
            rotations.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            Ok(tokens("rotated", Some(900_000)))
        };

        let (first, second, third) = tokio::join!(
            session.lease(100_000, rotate),
            session.lease(100_000, rotate),
            session.lease(100_000, rotate),
        );

        assert_eq!(first.unwrap().tokens.access_token, "rotated");
        assert_eq!(second.unwrap().tokens.access_token, "rotated");
        assert_eq!(third.unwrap().tokens.access_token, "rotated");
        assert_eq!(rotations.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_burst_of_rejected_leases_rotates_once() {
        let session = session_with(tokens("at", Some(900_000)));
        let stale = session
            .lease(100_000, |_| async { unreachable!("token is fresh") })
            .await
            .expect("lease should succeed");
        let rotations = AtomicUsize::new(0);

        let rotate = |_| async {
            rotations.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            Ok(tokens("rotated", Some(1_800_000)))
        };

        let (first, second, third) = tokio::join!(
            session.renew(&stale, rotate),
            session.renew(&stale, rotate),
            session.renew(&stale, rotate),
        );

        assert_eq!(first.unwrap().tokens.access_token, "rotated");
        assert_eq!(second.unwrap().tokens.access_token, "rotated");
        assert_eq!(third.unwrap().tokens.access_token, "rotated");
        assert_eq!(rotations.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn renew_hands_the_rotated_refresh_token_to_the_next_rotation() {
        let session = session_with(tokens("at", Some(900_000)));
        let stale = session
            .lease(100_000, |_| async { unreachable!("token is fresh") })
            .await
            .expect("lease should succeed");

        session
            .renew(&stale, |current| async move {
                assert_eq!(current.refresh_token, "rt");
                Ok(DayliteTokenState {
                    access_token: "rotated".to_string(),
                    refresh_token: "rt-2".to_string(),
                    access_token_expires_at_ms: Some(1_800_000),
                })
            })
            .await
            .expect("renew should succeed");

        let renewed = session
            .lease(100_000, |_| async { unreachable!("token is fresh") })
            .await
            .expect("lease should succeed");

        session
            .renew(&renewed, |current| async move {
                assert_eq!(current.refresh_token, "rt-2");
                Ok(tokens("rotated-again", Some(2_700_000)))
            })
            .await
            .expect("second renew should succeed");
    }

    #[tokio::test]
    async fn seed_does_not_overwrite_a_rotated_session() {
        let session = session_with(tokens("at", Some(150_000)));
        session
            .lease(100_000, |_| async { Ok(tokens("rotated", Some(900_000))) })
            .await
            .expect("lease should succeed");

        session.seed(tokens("from-keychain", Some(900_000)));

        let lease = session
            .lease(100_000, |_| async { unreachable!("token is fresh") })
            .await
            .expect("lease should succeed");
        assert_eq!(lease.tokens.access_token, "rotated");
    }

    #[tokio::test]
    async fn persists_rotated_tokens() {
        let keychain = Arc::new(Keychain::default());
        let session = session_backed_by(tokens("at", None), keychain.clone());

        session
            .lease(100_000, |_| async { Ok(tokens("rotated", Some(900_000))) })
            .await
            .expect("lease should succeed");

        assert_eq!(keychain.stored(), vec!["rotated"]);
    }

    #[tokio::test]
    async fn persists_adopted_tokens() {
        let keychain = Arc::new(Keychain::default());
        let session = session_backed_by(tokens("at", None), keychain.clone());

        session
            .adopt(|| async { Ok(tokens("minted", Some(900_000))) })
            .await
            .expect("adopt should succeed");

        assert_eq!(keychain.stored(), vec!["minted"]);
    }

    #[tokio::test]
    async fn keeps_rotated_tokens_the_keychain_refused_and_persists_them_later() {
        let keychain = Arc::new(Keychain::default());
        let session = session_backed_by(tokens("at", None), keychain.clone());
        keychain.refuses.store(true, Ordering::SeqCst);

        session
            .lease(100_000, |_| async { Ok(tokens("rotated", Some(900_000))) })
            .await
            .expect_err("a refused keychain write should surface");
        session
            .lease(100_000, |_| async {
                unreachable!("rotated tokens are kept")
            })
            .await
            .expect_err("tokens stay unleased while unpersisted");

        keychain.refuses.store(false, Ordering::SeqCst);
        let lease = session
            .lease(100_000, |_| async {
                unreachable!("rotated tokens are kept")
            })
            .await
            .expect("lease should succeed once the keychain accepts");

        assert_eq!(lease.tokens.access_token, "rotated");
        assert_eq!(keychain.stored(), vec!["rotated"]);
    }

    #[tokio::test]
    async fn renew_persists_tokens_the_keychain_refused_instead_of_rotating_again() {
        let keychain = Arc::new(Keychain::default());
        let session = session_backed_by(tokens("at", Some(900_000)), keychain.clone());
        let stale = session
            .lease(100_000, |_| async { unreachable!("token is fresh") })
            .await
            .expect("lease should succeed");
        keychain.refuses.store(true, Ordering::SeqCst);
        session
            .renew(&stale, |_| async { Ok(tokens("rotated", Some(1_800_000))) })
            .await
            .expect_err("a refused keychain write should surface");

        keychain.refuses.store(false, Ordering::SeqCst);
        let renewed = session
            .renew(&stale, |_| async {
                unreachable!("rotated tokens are kept")
            })
            .await
            .expect("renew should succeed once the keychain accepts");

        assert_eq!(renewed.tokens.access_token, "rotated");
        assert_eq!(keychain.stored(), vec!["rotated"]);
    }

    #[tokio::test]
    async fn close_waits_for_an_inflight_rotation_and_blocks_later_ones() {
        let keychain = Arc::new(Keychain::default());
        let session = Arc::new(session_backed_by(tokens("at", None), keychain.clone()));
        let (respond, response) = tokio::sync::oneshot::channel::<()>();
        let response = Mutex::new(Some(response));

        let inflight = {
            let session = session.clone();
            tokio::spawn(async move {
                session
                    .lease(100_000, |_| {
                        let response = response.lock().unwrap().take();
                        async move {
                            response.expect("rotates once").await.unwrap();
                            Ok(tokens("rotated", Some(900_000)))
                        }
                    })
                    .await
            })
        };
        tokio::task::yield_now().await;

        let closing = {
            let session = session.clone();
            tokio::spawn(async move { session.close().await })
        };
        tokio::task::yield_now().await;
        assert!(!closing.is_finished());

        respond.send(()).unwrap();
        inflight
            .await
            .unwrap()
            .expect("inflight rotation should finish");
        closing.await.unwrap().expect("close should succeed");
        assert_eq!(keychain.stored(), vec!["rotated"]);

        session
            .lease(1_000_000, |_| async { unreachable!("session is closed") })
            .await
            .expect_err("a closed session should not rotate");
    }

    #[tokio::test]
    async fn a_lease_queued_before_close_does_not_rotate_after_a_failed_rotation() {
        let session = Arc::new(session_with(tokens("at", None)));
        let rotations = Arc::new(AtomicUsize::new(0));
        let (respond, response) = tokio::sync::oneshot::channel::<()>();
        let response = Arc::new(Mutex::new(Some(response)));

        let spawn_lease = |session: Arc<TokenSession>| {
            let rotations = rotations.clone();
            let response = response.clone();
            tokio::spawn(async move {
                session
                    .lease(100_000, |_| {
                        rotations.fetch_add(1, Ordering::SeqCst);
                        let response = response.lock().unwrap().take();
                        async move {
                            response.expect("rotates once").await.unwrap();
                            Err(DayliteApiError::new(
                                DayliteApiErrorCode::Timeout,
                                "Zeitüberschreitung",
                                "timeout",
                            ))
                        }
                    })
                    .await
            })
        };
        let inflight = spawn_lease(session.clone());
        tokio::task::yield_now().await;
        let queued = spawn_lease(session.clone());
        tokio::task::yield_now().await;
        let closing = {
            let session = session.clone();
            tokio::spawn(async move { session.close().await })
        };
        tokio::task::yield_now().await;

        respond.send(()).unwrap();
        inflight.await.unwrap().expect_err("rotation fails");
        queued
            .await
            .unwrap()
            .expect_err("a closing session should not rotate");
        closing.await.unwrap().expect("close should succeed");

        assert_eq!(rotations.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_closed_session_does_not_adopt() {
        let session = session_with(tokens("at", None));
        session.close().await.expect("close should succeed");

        session
            .adopt(|| async { unreachable!("session is closed") })
            .await
            .expect_err("a closed session should not adopt");
    }

    #[tokio::test]
    async fn keeps_adopted_tokens_the_keychain_refused() {
        let keychain = Arc::new(Keychain::default());
        let session = session_backed_by(tokens("at", None), keychain.clone());
        keychain.refuses.store(true, Ordering::SeqCst);

        session
            .adopt(|| async { Ok(tokens("minted", Some(900_000))) })
            .await
            .expect_err("a refused keychain write should surface");

        keychain.refuses.store(false, Ordering::SeqCst);
        let lease = session
            .lease(100_000, |_| async {
                unreachable!("adopted tokens are kept")
            })
            .await
            .expect("lease should succeed once the keychain accepts");

        assert_eq!(lease.tokens.access_token, "minted");
        assert_eq!(keychain.stored(), vec!["minted"]);
    }
}
