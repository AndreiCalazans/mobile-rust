//! Repository + session: the app's belief about the user.
//!
//! Holds the source of truth (current session) behind a Mutex, exactly the
//! "store is the only cache" shape from the architecture. The fake login
//! fabricates a user; no real auth server is involved.

use crate::model::{SessionState, User};
use std::sync::Mutex;

/// The user repository. One per domain, app-lifetime scope.
pub struct UserRepository {
    session: Mutex<SessionState>,
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(SessionState::LoggedOut),
        }
    }

    /// Fabricate a session for the given display name. Pure/local — no network.
    pub fn fake_login(&self, display_name: &str) -> SessionState {
        let user = User {
            id: format!("u_{}", display_name.to_lowercase().replace(' ', "_")),
            display_name: display_name.to_string(),
            email: Some(format!(
                "{}@example.com",
                display_name.to_lowercase().replace(' ', ".")
            )),
        };
        let state = SessionState::Active {
            user,
            token: "fake-jwt-token".to_string(),
        };
        *self.session.lock().unwrap() = state.clone();
        state
    }

    pub fn logout(&self) {
        *self.session.lock().unwrap() = SessionState::LoggedOut;
    }

    pub fn current_session(&self) -> SessionState {
        self.session.lock().unwrap().clone()
    }

    pub fn current_user(&self) -> Option<User> {
        match &*self.session.lock().unwrap() {
            SessionState::Active { user, .. } => Some(user.clone()),
            SessionState::LoggedOut => None,
        }
    }
}

impl Default for UserRepository {
    fn default() -> Self {
        Self::new()
    }
}
