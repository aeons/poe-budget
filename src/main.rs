mod config;
mod ninja;
mod trade;

use std::env;

use crate::config::Config;

use anyhow::Result;
use trade::Trade;

fn main() -> Result<()> {
    dotenv::dotenv()?;
    let session_id = env::var("POESESSID")?;
    let mut config: Config = Config::load()?;

    config.update_chaos_ratio()?;

    let trade = Trade::new(&session_id)?;

    for item in &config.items {
        let prices = search_chaos_prices(&trade, &config, &item.query)?;
        dbg!(prices.iter().sum::<f64>() / prices.len() as f64);
    }

    Ok(())
}

fn search_chaos_prices(trade: &Trade, config: &Config, query: &str) -> Result<Vec<f64>> {
    let search = trade.search(&config.league, query)?;
    let fetch = trade.fetch(search.result)?;

    Ok(fetch
        .result
        .into_iter()
        .filter_map(|r| match r.listing.price.currency.as_str() {
            "chaos" => Some(r.listing.price.amount),
            "divine" => Some(r.listing.price.amount * config.chaos_to_divine),
            _ => None,
        })
        .collect())
}
