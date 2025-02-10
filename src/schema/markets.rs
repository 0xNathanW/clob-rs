use serde::Deserialize;
use chrono::{DateTime, Utc};
use crate::error::MarketError;

// Where T is the type of the market.
#[derive(Debug, Deserialize)]
pub struct Markets<T> {
    pub limit:       u32,
    pub count:       u32,
    // Pagination item to retrieve the next page base64 encoded. 
    // 'LTE=' means the end and '' means the beginning.
    pub next_cursor: String,
    pub data:        Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Market {
    pub accepting_order_timestamp: Option<String>,
    pub accepting_orders:          bool,
    pub active:                    bool,
    pub closed:                    bool,
    pub archived:                  bool,
    // Id of market which is also the CTF condition ID.
    pub condition_id:              String,
    // Question id of market which is also the CTF question ID which is used to derive the `condition_id`.
    pub question_id:               String,
    pub is_50_50_outcome:          bool,
    pub enable_order_book:         bool,
    pub tokens:                    [Token; 2],
    pub rewards:                   Rewards,
    pub maker_base_fee:            u32,
    pub taker_base_fee:            u32,
    pub minimum_order_size:        u32,
    // Minimum tick size in units of implied probability (max price resolution).
    pub minimum_tick_size:         f64,
    pub description:               String,
    // ISO string of market end date.
    // TODO: convert to datetime
    pub end_date_iso:              Option<String>,
    // ISO string of game start time which is used to trigger delay.
    pub game_start_time:           Option<String>,
    pub question:                  String,
    pub market_slug:               String,
    // Seconds of match delay for in-game trading.
    pub seconds_delay:             i32,
    // Icon url.
    pub icon:                      String,
    // Image url.
    pub image:                     String,
    // Address of associated fixed product market maker on Polygon network.
    pub fpmm:                      String,
    pub neg_risk:                  bool,
    pub neg_risk_market_id:        String,
    pub neg_risk_request_id:       String,
    pub notifications_enabled:     bool,
    pub tags:                      Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SimplifiedMarketResponse {
    pub accepting_orders: bool,
    pub active:           bool,
    pub archived:         bool,
    pub condition_id:     String,
    pub rewards:          Rewards,
    pub tokens:           [Token; 2],
}

#[derive(Debug, Deserialize)]
pub struct Token {
    pub token_id: String,
    pub outcome:  String,
    pub price:    f64,
    pub winner:   bool,
}

#[derive(Debug, Deserialize)]
pub struct Rewards {
    // Minimum size of an order to score.
    pub min_size:   u32,
    // Maximum spread from the midpoint until an order scores.
    pub max_spread: f64,
    pub rates:      Option<Vec<Rates>>,
}

#[derive(Debug, Deserialize)]
pub struct Rates {
    pub asset_address:      String,
    pub rewards_daily_rate: f64,
}

impl Market {

    pub fn is_active(&self) -> bool {
        self.active && !self.closed && !self.archived && self.accepting_orders
    }

    pub fn yes_asset_id(&self) -> Result<&String, MarketError> {
        self.tokens
            .iter()
            .find(|t| t.outcome.to_lowercase() == "yes")
            .ok_or(MarketError::TokenNotFound("yes".to_string()))
            .map(|t| &t.token_id)
    }

    pub fn no_asset_id(&self) -> Result<&String, MarketError> {
        self.tokens
            .iter()
            .find(|t| t.outcome.to_lowercase() == "no")
            .ok_or(MarketError::TokenNotFound("no".to_string()))
            .map(|t| &t.token_id)
    }

    pub fn min_order_size(&self) -> u32 {
        self.minimum_order_size * 100
    }

    // 0.1 -> 100, 0.01 -> 10, 0.001 -> 1
    pub fn min_tick_size(&self) -> u32 {
        let ts: u32 = (self.minimum_tick_size * 1000.0 + 0.5) as u32;
        debug_assert!(ts == 1 || ts == 10 || ts == 100);
        ts
    }

    pub fn end_date_utc(&self) -> Option<Result<DateTime<Utc>, MarketError>> {
        self.end_date_iso
            .as_ref()
            .map(|s| s.parse::<DateTime<Utc>>().map_err(|_| MarketError::InvalidDate(s.clone())))
    }

    pub fn game_start_time_utc(&self) -> Option<Result<DateTime<Utc>, MarketError>> {
        self.game_start_time
            .as_ref()
            .map(|s| s.parse::<DateTime<Utc>>().map_err(|_| MarketError::InvalidDate(s.clone())))
    }

    pub fn rewards_min_size(&self) -> u32 {
        self.rewards.min_size * 100
    }

    pub fn rewards_max_spread(&self) -> u32 {
        (self.rewards.max_spread * 10.0) as u32
    }
}


#[derive(Debug, Deserialize)]
pub struct Orderbook {
    pub market:    String,
    pub asset_id:  String,
    pub hash:      String,
    pub timestamp: String,
    pub bids:      Vec<OrderSummary>,
    pub asks:      Vec<OrderSummary>,
}

#[derive(Debug, Deserialize)]
pub struct OrderSummary {
    pub price: String,
    pub size:  String
}