//! API + data source: GraphQL over the Coinbase public endpoint.
//!
//! Mirrors the GraphQlClient pattern in ~/coinbase/research-mobile-app:
//! POST { query, variables } to graphql.coinbase.com/query with a CB-VERSION
//! header, read `data`, surface UNAUTHENTICATED as an auth error.

use crate::model::{AssetPrice, CoreError};
use serde::Serialize;
use serde_json::Value;

const GRAPHQL_URL: &str = "https://graphql.coinbase.com/query";
const CB_VERSION: &str = "2021-01-11";

#[derive(Serialize)]
struct GraphQlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

/// Thin GraphQL POST helper. Returns the `data` object or a typed error.
/// This is the generic entry point — the Rust layer "supports GraphQL requests"
/// through it, and typed fetchers (below) build on top.
pub async fn query(
    client: &reqwest::Client,
    query: &str,
    variables: Option<Value>,
    bearer: Option<&str>,
) -> Result<Value, CoreError> {
    let body = GraphQlRequest { query, variables };

    let mut req = client
        .post(GRAPHQL_URL)
        .header("Content-Type", "application/json")
        .header("CB-VERSION", CB_VERSION);
    if let Some(token) = bearer {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let res = req.json(&body).send().await?;
    let root: Value = res.json().await.map_err(|e| CoreError::Decode {
        message: e.to_string(),
    })?;

    if let Some(errors) = root.get("errors") {
        if errors.to_string().contains("UNAUTHENTICATED") {
            return Err(CoreError::Unauthenticated);
        }
        return Err(CoreError::Network {
            message: errors.to_string(),
        });
    }

    root.get("data").cloned().ok_or(CoreError::Decode {
        message: "graphql response has no data field".into(),
    })
}

const ASSET_QUERY: &str = r#"
query RetailAssetDetail($s: String!) {
  assetBySymbol(symbol: $s) {
    displaySymbol
    name
    latestPrice(quoteCurrency: USD) {
      price
      percentChanges { day }
    }
  }
}
"#;

/// Typed fetch: one asset's price by display symbol, e.g. "BTC".
pub async fn fetch_asset_price(
    client: &reqwest::Client,
    symbol: &str,
) -> Result<AssetPrice, CoreError> {
    let vars = serde_json::json!({ "s": symbol });
    let data = query(client, ASSET_QUERY, Some(vars), None).await?;

    let a = data.get("assetBySymbol").ok_or(CoreError::NotFound {
        message: format!("no asset '{symbol}'"),
    })?;
    if a.is_null() {
        return Err(CoreError::NotFound {
            message: format!("no asset '{symbol}'"),
        });
    }

    let latest = a.get("latestPrice");
    Ok(AssetPrice {
        symbol: str_field(a, "displaySymbol").unwrap_or_else(|| symbol.to_string()),
        name: str_field(a, "name").unwrap_or_default(),
        price_usd: latest
            .and_then(|l| str_field(l, "price"))
            .unwrap_or_else(|| "0".into()),
        change_day_percent: latest
            .and_then(|l| l.get("percentChanges"))
            .and_then(|p| p.get("day"))
            .and_then(|d| d.as_f64())
            .unwrap_or(0.0),
    })
}

/// Convenience: bitcoin price, the headline demo call.
pub async fn fetch_bitcoin(client: &reqwest::Client) -> Result<AssetPrice, CoreError> {
    fetch_asset_price(client, "BTC").await
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
}
