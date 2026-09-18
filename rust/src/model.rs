//! Value types shared across the FFI boundary.
//!
//! Everything here is a `uniffi::Record` or `uniffi::Enum`: immutable data that
//! copies by value into Swift structs / Kotlin data classes.

/// The authenticated user (fake, for the PoC).
#[derive(Debug, Clone, uniffi::Record)]
pub struct User {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
}

/// Session lifecycle. A sealed class (Kotlin) / enum with payloads (Swift).
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SessionState {
    LoggedOut,
    Active { user: User, token: String },
}

/// One dictionary entry from dictionaryapi.dev.
#[derive(Debug, Clone, uniffi::Record)]
pub struct WordEntry {
    pub word: String,
    pub phonetic: Option<String>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Definition {
    pub part_of_speech: String,
    pub meaning: String,
    pub example: Option<String>,
}

/// A crypto asset price fetched over GraphQL.
#[derive(Debug, Clone, uniffi::Record)]
pub struct AssetPrice {
    pub symbol: String,
    pub name: String,
    pub price_usd: String,
    pub change_day_percent: f64,
}

/// One typed error surface for the whole core.
/// Swift: `throws` + `LocalizedError`. Kotlin: `CoreException` subclasses.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("network error: {message}")]
    Network { message: String },
    #[error("decoding error: {message}")]
    Decode { message: String },
    #[error("not found: {message}")]
    NotFound { message: String },
    #[error("not authenticated")]
    Unauthenticated,
}
