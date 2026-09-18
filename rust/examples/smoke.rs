//! Live smoke test: hits the real dictionary REST API and Coinbase GraphQL.
//! Run with: cargo run --example smoke

use app_core::AppCore;

#[tokio::main]
async fn main() {
    let core = AppCore::new();

    println!("== session ==");
    println!("{:?}", core.login("Ada Lovelace".into()));

    println!("\n== dictionary (REST) ==");
    match core.define_word("serendipity".into()).await {
        Ok(e) => println!(
            "{} [{}] — {} definition(s); first: {}",
            e.word,
            e.phonetic.unwrap_or_default(),
            e.definitions.len(),
            e.definitions
                .first()
                .map(|d| d.meaning.as_str())
                .unwrap_or("")
        ),
        Err(e) => println!("dictionary error: {e}"),
    }

    println!("\n== bitcoin (GraphQL) ==");
    match core.fetch_bitcoin().await {
        Ok(a) => println!(
            "{} ({}) = ${} | day {}%",
            a.symbol, a.name, a.price_usd, a.change_day_percent
        ),
        Err(e) => println!("graphql error: {e}"),
    }
}
