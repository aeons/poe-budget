use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use isahc::prelude::*;
use isahc::Body;
use isahc::HttpClient;
use isahc::Request;
use isahc::Response;
use serde::Deserialize;

type RL = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

pub struct Trade {
    client: HttpClient,
    search_limiter: RL,
    fetch_limiter: RL,
}

impl Trade {
    const BASE_URL: &str = "https://www.pathofexile.com/api/trade";

    pub fn new(poe_session_id: &str) -> Result<Self> {
        let client = HttpClient::builder()
            .default_header("Cookie", format!("POESESSID={poe_session_id}"))
            .build()?;
        // The trade API allows 3 searches per 5 seconds
        let search_period = Duration::from_millis((5000.0 / 3.0) as u64);
        let search_limiter = RateLimiter::direct(Quota::with_period(search_period).unwrap());
        // The trade API allows 6 fetches per 4 seconds
        let fetch_period = Duration::from_millis((4000.0 / 6.0) as u64);
        let fetch_limiter = RateLimiter::direct(Quota::with_period(fetch_period).unwrap());
        Ok(Self {
            client,
            search_limiter,
            fetch_limiter,
        })
    }

    pub fn search(&self, league: &str, query: &str) -> Result<Search> {
        let request = Request::post(format!("{}/search/{}", Self::BASE_URL, league))
            .header("Content-Type", "application/json")
            .body(query)?;
        let response = self.send(&self.search_limiter, request)?.json()?;
        Ok(response)
    }

    pub fn fetch(&self, hashes: Vec<String>) -> Result<Fetch> {
        let first_ten = hashes.into_iter().take(10).collect::<Vec<_>>().join(",");
        let request = Request::get(format!("{}/fetch/{}", Self::BASE_URL, first_ten)).body(())?;
        let response = self.send(&self.fetch_limiter, request)?.json()?;
        Ok(response)
    }

    fn send<T: Into<Body>>(&self, limiter: &RL, request: Request<T>) -> Result<Response<Body>> {
        match limiter.check() {
            Ok(()) => {
                let response = self.client.send(request)?;
                Ok(response)
            }
            Err(not_until) => {
                thread::sleep(not_until.wait_time_from(Instant::now()));
                self.send(limiter, request)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Search {
    pub result: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Fetch {
    pub result: Vec<FetchItem>,
}

#[derive(Debug, Deserialize)]
pub struct FetchItem {
    pub id: String,
    pub listing: FetchItemListing,
}

#[derive(Debug, Deserialize)]
pub struct FetchItemListing {
    pub price: FetchItemPrice,
}

#[derive(Debug, Deserialize)]
pub struct FetchItemPrice {
    pub amount: f64,
    pub currency: String,
}
