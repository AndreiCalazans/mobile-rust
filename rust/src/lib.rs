//! app_core — all business logic for the mobile PoC.
//!
//! The FFI boundary sits ABOVE the use-case layer: native code (SwiftUI /
//! Compose) only holds one `AppCore` object and calls its methods. Use cases,
//! repositories, data sources, and transport all live below, in Rust.
//!
//! Layers:
//!   model            value types crossing the boundary
//!   transport        shared HTTP client (bytes only)
//!   dictionary       REST data source (dictionaryapi.dev)
//!   graphql          GraphQL client + typed bitcoin/asset fetch
//!   user_repository  fake session + user source of truth
//!   AppCore (here)   the use-case facade exposed over UniFFI

mod dictionary;
mod graphql;
mod model;
mod transport;
mod user_repository;

use std::sync::Arc;

pub use model::{AssetPrice, CoreError, Definition, SessionState, User, WordEntry};
use user_repository::UserRepository;

uniffi::setup_scaffolding!();

/// The single entry point native code holds. Owns the HTTP client and the
/// repositories; exposes use cases as async/sync methods.
#[derive(uniffi::Object)]
pub struct AppCore {
    http: reqwest::Client,
    users: UserRepository,
}

#[uniffi::export(async_runtime = "tokio")]
impl AppCore {
    /// `AppCore()` in Swift / Kotlin.
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            http: transport::http_client(),
            users: UserRepository::new(),
        })
    }

    // --- Session use cases (local, synchronous) -------------------------

    /// Create a fake session for a display name.
    pub fn login(&self, display_name: String) -> SessionState {
        self.users.fake_login(&display_name)
    }

    pub fn logout(&self) {
        self.users.logout();
    }

    pub fn current_session(&self) -> SessionState {
        self.users.current_session()
    }

    pub fn current_user(&self) -> Option<User> {
        self.users.current_user()
    }

    // --- Dictionary use cases (REST, async) -----------------------------

    /// Fetch definitions for one word.
    pub async fn define_word(&self, word: String) -> Result<WordEntry, CoreError> {
        dictionary::fetch_word(&self.http, &word).await
    }

    /// Fetch a "different set of words" — several words in sequence.
    /// Unknown words are skipped so one 404 does not fail the whole batch.
    pub async fn define_words(&self, words: Vec<String>) -> Vec<WordEntry> {
        let mut out = Vec::new();
        for w in words {
            if let Ok(entry) = dictionary::fetch_word(&self.http, &w).await {
                out.push(entry);
            }
        }
        out
    }

    // --- Crypto use cases (GraphQL, async) ------------------------------

    /// Fetch bitcoin price via GraphQL.
    pub async fn fetch_bitcoin(&self) -> Result<AssetPrice, CoreError> {
        graphql::fetch_bitcoin(&self.http).await
    }

    /// Fetch any asset by display symbol via GraphQL.
    pub async fn fetch_asset(&self, symbol: String) -> Result<AssetPrice, CoreError> {
        graphql::fetch_asset_price(&self.http, &symbol).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_login_produces_active_session() {
        let core = AppCore::new();
        let state = core.login("Ada Lovelace".into());
        match state {
            SessionState::Active { user, token } => {
                assert_eq!(user.display_name, "Ada Lovelace");
                assert_eq!(user.id, "u_ada_lovelace");
                assert_eq!(token, "fake-jwt-token");
            }
            _ => panic!("expected active session"),
        }
        assert!(core.current_user().is_some());
        core.logout();
        assert!(core.current_user().is_none());
    }
}
