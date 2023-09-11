use anyhow::{anyhow, Result};
use isahc::ReadResponseExt;
use serde::Deserialize;

pub fn get_chaos_ratio(league: &str) -> Result<f64> {
    let response: CurrencyOverview = isahc::get(format!(
        "https://poe.ninja/api/data/currencyoverview?league={league}&type=Currency"
    ))?
    .json()?;

    response
        .lines
        .iter()
        .find_map(|c| {
            if c.currency_type_name == "Divine Orb" {
                Some(c.chaos_equivalent)
            } else {
                None
            }
        })
        .ok_or(anyhow!("Divine Orb ratio not found in poe.ninja response"))
}

#[derive(Debug, Deserialize)]
struct CurrencyOverview {
    lines: Vec<CurrencyOverviewLine>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyOverviewLine {
    currency_type_name: String,
    chaos_equivalent: f64,
}
