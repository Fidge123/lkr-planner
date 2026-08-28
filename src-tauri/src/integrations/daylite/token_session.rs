use std::future::Future;
use std::sync::{OnceLock, RwLock};

use super::shared::{DayliteApiError, DayliteTokenState};

/// A leased token has to outlive the whole operation: nothing refreshes mid-request.
const LEASE_MIN_REMAINING_MS: u64 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TokenLease {
    pub(super) tokens: DayliteTokenState,
    generation: u64,
}

struct SessionState {
    tokens: DayliteTokenState,
    generation: u64,
}

/// Only rotation is serialized: Daylite invalidates a refresh token as it issues the next.
/// A request carrying an already valid access token is safe in parallel.
pub(super) struct TokenSession {
    state: RwLock<Option<SessionState>>,
    rotation: tokio::sync::Mutex<()>,
    min_remaining_ms: u64,
}

impl TokenSession {
    fn new(min_remaining_ms: u64) -> Self {
        Self {
            state: RwLock::new(None),
            rotation: tokio::sync::Mutex::new(()),
            min_remaining_ms,
        }
    }

    pub(super) fn seed(&self, tokens: DayliteTokenState) {
        let mut state = self.state.write().expect("token session poisoned");
        if state.is_none() {
            *state = Some(SessionState {
                tokens,
                generation: 0,
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
        let tokens = mint().await?;
        Ok(self.publish(tokens).tokens)
    }

    async fn rotate_locked<F, Fut>(&self, rotate: F) -> Result<TokenLease, DayliteApiError>
    where
        F: Fn(DayliteTokenState) -> Fut,
        Fut: Future<Output = Result<DayliteTokenState, DayliteApiError>>,
    {
        let current = self.current_tokens().unwrap_or_default();
        let rotated = rotate(current).await?;
        Ok(self.publish(rotated))
    }

    fn publish(&self, tokens: DayliteTokenState) -> TokenLease {
        let mut state = self.state.write().expect("token session poisoned");
        let generation = state.as_ref().map(|s| s.generation + 1).unwrap_or(1);
        *state = Some(SessionState {
            tokens: tokens.clone(),
            generation,
        });

        TokenLease { tokens, generation }
    }

    fn current_tokens(&self) -> Option<DayliteTokenState> {
        let state = self.state.read().expect("token session poisoned");
        state.as_ref().map(|s| s.tokens.clone())
    }

    fn fresh_lease(&self, now_ms: u64) -> Option<TokenLease> {
        self.lease_if(|state| has_life_left(&state.tokens, now_ms, self.min_remaining_ms))
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
    SESSION.get_or_init(|| TokenSession::new(LEASE_MIN_REMAINING_MS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tokens(access: &str, expires_at_ms: Option<u64>) -> DayliteTokenState {
        DayliteTokenState {
            access_token: access.to_string(),
            refresh_token: "rt".to_string(),
            access_token_expires_at_ms: expires_at_ms,
        }
    }

    fn session_with(seeded: DayliteTokenState) -> TokenSession {
        let session = TokenSession::new(60_000);
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
}
