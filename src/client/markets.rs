use reqwest::Method;
use serde::Deserialize;
use crate::{schema::*, Result};
use super::{ClobClient, RequestArgs, AuthLevel};

impl ClobClient {

    pub async fn get_market(&self, market_id: &str) -> Result<Market> {
        self.request(RequestArgs {
            method: Method::GET,
            path: &format!("/markets/{}", market_id),
            queries: None,
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }

    pub async fn get_markets(&self, next_cursor: Option<&str>) -> Result<Markets<Market>> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/markets",
            queries: Some(&[("next_cursor", next_cursor.unwrap_or("MTA="))]),
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }

    // Get markets that have rewards enabled.
    pub async fn get_sampling_markets(&self) -> Result<Markets<Market>> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/sampling-markets",
            queries: None,
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }

    // Get markets in a reduced schema.
    pub async fn get_simplified_markets(&self, next_cursor: Option<&str>) -> Result<Markets<SimplifiedMarketResponse>> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/simplified-markets",
            queries: Some(&[("next_cursor", next_cursor.unwrap_or("MTA="))]),
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }

    pub async fn get_simplified_sampling_markets(&self, next_cursor: Option<&str>) -> Result<Markets<SimplifiedMarketResponse>> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/sampling-simplified-markets",
            queries: Some(&[("next_cursor", next_cursor.unwrap_or("MTA="))]),
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }

    pub async fn get_tick_size(&self, token_id: &str) -> Result<f64> {

        #[derive(Deserialize, Debug)]
        struct Resp {
            minimum_tick_size: f64,
        }

        Ok(
            self.request::<Resp>(RequestArgs {
                method: Method::GET,
                path: "/tick-size",
                queries: Some(&[("token_id", token_id)]),
                body: None,
                auth_level: AuthLevel::None,
            })
            .await?
            .minimum_tick_size
        )
    }

    pub async fn is_neg_risk(&self, token_id: &str) -> Result<bool> {

        #[derive(Deserialize, Debug)]
        struct Resp {
            neg_risk: bool,
        }

        Ok(
            self.request::<Resp>(RequestArgs {
                method: Method::GET,
                path: "/neg-risk",
                queries: Some(&[("token_id", token_id)]),
                body: None,
                auth_level: AuthLevel::None,
            })
            .await?
            .neg_risk
        )
    }

    pub async fn get_market_book(&self, token_id: &str) -> Result<Orderbook> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/book",
            queries: Some(&[("token_id", token_id)]),
            body: None,
            auth_level: AuthLevel::None,
        }).await
    }
    
    // TODO: Other market endpoints .. books, prices, etc. Don't need atm.
}
