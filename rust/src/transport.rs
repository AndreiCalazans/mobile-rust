//! Transport layer: the single shared HTTP client.
//!
//! This is the bottom of the stack — it moves bytes and never knows about
//! domain types. Data sources build on top of it.

use crate::model::CoreError;

/// Build the process-wide reqwest client. One connection pool for the app.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("mobile-rust-poc/0.1")
        .build()
        .expect("failed to build HTTP client")
}

impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        CoreError::Network {
            message: e.to_string(),
        }
    }
}
