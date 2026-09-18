//! Spike: a fake user repository exposed over UniFFI.
//!
//! Goal of the spike: prove that a realistic repository shape — records,
//! enums with data, Option, typed errors, async, a stateful object, and a
//! callback/observer — round-trips into idiomatic Swift and Kotlin.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

uniffi::setup_scaffolding!();

// ---------------------------------------------------------------------------
// Value types — become `data class` (Kotlin) / `struct` (Swift)
// ---------------------------------------------------------------------------

/// A plain value type. Immutable across the boundary.
#[derive(Debug, Clone, uniffi::Record)]
pub struct User {
    pub id: String,
    pub display_name: String,
    /// Optional field -> Kotlin `String?` / Swift `String?`.
    pub email: Option<String>,
    pub role: Role,
}

/// Enum with no data -> Kotlin `enum class` / Swift `enum`.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum Role {
    Guest,
    Member,
    Admin,
}

/// Enum WITH associated data -> Kotlin sealed class / Swift enum with payloads.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SessionState {
    LoggedOut,
    Active { user: User, token: String },
    Expired { since_epoch_secs: u64 },
}

// ---------------------------------------------------------------------------
// Typed errors — become Swift `throws` / Kotlin exceptions
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum RepoError {
    #[error("user {id} not found")]
    NotFound { id: String },
    #[error("network failed: {reason}")]
    Network { reason: String },
}

// ---------------------------------------------------------------------------
// Stateful object — becomes a reference type (class) on both sides
// ---------------------------------------------------------------------------

/// Observer implemented on the NATIVE side and called back from Rust.
/// Proves data can flow Rust -> Swift/Kotlin, not just request/response.
#[uniffi::export(with_foreign)]
pub trait SessionObserver: Send + Sync {
    fn on_session_changed(&self, state: SessionState);
}

/// The fake repository. Holds interior state behind a Mutex, exactly how the
/// real repository would hold its cache / source-of-truth.
#[derive(uniffi::Object)]
pub struct UserRepository {
    users: Mutex<Vec<User>>,
    session: Mutex<SessionState>,
    observer: Mutex<Option<Arc<dyn SessionObserver>>>,
}

#[uniffi::export(async_runtime = "tokio")]
impl UserRepository {
    /// Constructor -> `UserRepository()` in Swift / Kotlin.
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        let seed = vec![
            User {
                id: "u_1".into(),
                display_name: "Ada Lovelace".into(),
                email: Some("ada@example.com".into()),
                role: Role::Admin,
            },
            User {
                id: "u_2".into(),
                display_name: "Guest".into(),
                email: None,
                role: Role::Guest,
            },
        ];
        Arc::new(Self {
            users: Mutex::new(seed),
            session: Mutex::new(SessionState::LoggedOut),
            observer: Mutex::new(None),
        })
    }

    /// Sync read -> plain function returning a list.
    pub fn all_users(&self) -> Vec<User> {
        self.users.lock().unwrap().clone()
    }

    /// Sync read that can fail -> `throws` (Swift) / throwing fun (Kotlin).
    pub fn user(&self, id: String) -> Result<User, RepoError> {
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .cloned()
            .ok_or(RepoError::NotFound { id })
    }

    /// Register the native observer.
    pub fn set_observer(&self, observer: Arc<dyn SessionObserver>) {
        *self.observer.lock().unwrap() = Some(observer);
    }

    /// ASYNC method -> Swift `async throws` / Kotlin `suspend`.
    /// Simulates a fake login round-trip and pushes a state change to the
    /// native observer.
    pub async fn fake_login(&self, id: String) -> Result<SessionState, RepoError> {
        // Simulate network latency on the tokio runtime.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let user = self.user(id)?;
        let state = SessionState::Active {
            user,
            token: "fake-jwt-token".into(),
        };
        *self.session.lock().unwrap() = state.clone();

        if let Some(obs) = self.observer.lock().unwrap().as_ref() {
            obs.on_session_changed(state.clone());
        }
        Ok(state)
    }

    pub fn current_session(&self) -> SessionState {
        self.session.lock().unwrap().clone()
    }
}
