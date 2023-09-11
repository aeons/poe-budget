use std::{collections::HashMap, fs};

use anyhow::Result;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use time::ext::NumericalDuration;
use time::OffsetDateTime;

use crate::ninja;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub league: String,
    pub chaos_to_divine: f64,
    pub last_update: Option<OffsetDateTime>,
    pub items: Vec<Item>,
    pub prices: Option<HashMap<String, Prices>>,
}

impl Config {
    const CONFIG_FILE_PATH: &str = "./config.ron";

    pub fn load() -> Result<Self> {
        let config = ron::from_str(&fs::read_to_string(Self::CONFIG_FILE_PATH)?)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        fs::write(
            Self::CONFIG_FILE_PATH,
            ron::ser::to_string_pretty(self, PrettyConfig::default())?,
        )?;
        Ok(())
    }

    pub fn update_chaos_ratio(&mut self) -> Result<()> {
        if self.chaos_to_divine <= 0.0
            || self.last_update.is_none()
            || self
                .last_update
                .is_some_and(|t| t < OffsetDateTime::now_utc() - 1.days())
        {
            let chaos_to_divine = ninja::get_chaos_ratio(&self.league)?;
            self.set_chaos_to_divine(chaos_to_divine)?;
        }

        Ok(())
    }

    fn set_chaos_to_divine(&mut self, ratio: f64) -> Result<()> {
        self.chaos_to_divine = ratio;
        self.last_update = Some(OffsetDateTime::now_utc());
        self.save()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Prices {
    pub chaos: Vec<f64>,
}
