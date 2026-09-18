//! Data source: dictionaryapi.dev (REST).
//!
//! Protocol-blind wire types stay in this module. They map to domain
//! `WordEntry` / `Definition` records before leaving.

use crate::model::{CoreError, Definition, WordEntry};
use serde::Deserialize;

const BASE_URL: &str = "https://api.dictionaryapi.dev/api/v2/entries/en";

// --- Wire types: the exact shape the API returns -------------------------

#[derive(Debug, Deserialize)]
struct WireEntry {
    word: String,
    phonetic: Option<String>,
    #[serde(default)]
    meanings: Vec<WireMeaning>,
}

#[derive(Debug, Deserialize)]
struct WireMeaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,
    #[serde(default)]
    definitions: Vec<WireDefinition>,
}

#[derive(Debug, Deserialize)]
struct WireDefinition {
    definition: String,
    example: Option<String>,
}

/// Fetch and map one word. Returns `NotFound` when the API 404s (unknown word).
pub async fn fetch_word(client: &reqwest::Client, word: &str) -> Result<WordEntry, CoreError> {
    let url = format!("{BASE_URL}/{word}");
    let res = client.get(&url).send().await?;

    if res.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(CoreError::NotFound {
            message: format!("no dictionary entry for '{word}'"),
        });
    }

    let wire: Vec<WireEntry> = res.json().await.map_err(|e| CoreError::Decode {
        message: e.to_string(),
    })?;

    let entry = wire.into_iter().next().ok_or(CoreError::NotFound {
        message: format!("no dictionary entry for '{word}'"),
    })?;

    Ok(map_entry(entry))
}

fn map_entry(w: WireEntry) -> WordEntry {
    let definitions = w
        .meanings
        .into_iter()
        .flat_map(|m| {
            m.definitions.into_iter().map(move |d| Definition {
                part_of_speech: m.part_of_speech.clone(),
                meaning: d.definition,
                example: d.example,
            })
        })
        .collect();

    WordEntry {
        word: w.word,
        phonetic: w.phonetic,
        definitions,
    }
}
