use std::collections::HashMap;
use alloy::primitives::{Address, U256};
use rand::Rng;
use reqwest::Method;
use serde_json::json;
use crate::{auth, schema::*, Result};
use super::{AuthLevel, ClobClient, RequestArgs};

const PUBLIC_TAKER_ADDRESS: Address = Address::ZERO;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    /* 
    A 'Fill-Or-Kill' order is an market order to buy 
    shares that must be executed immediately in its entirety; 
    otherwise, the entire order will be cancelled. 
    */
    FOK,
    /* 
    A 'Good-Til-Cancelled' order is a limit order that is 
    active until it is fulfilled or cancelled. 
    */
    GTC,
    /* 
    A 'Good-Til-Day' order is a type of order that is 
    active until its specified date (UTC seconds timestamp), 
    unless it has already been fulfilled or cancelled. 
    */
    GTD,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignatureType {
    // EIP712 signature signed by an EOA.
    EOA,
    // EIP712 signatures signed by a signer associated with funding Polymarket proxy wallet.
    PolyProxy,
    // EIP712 signatures signed by a signer associated with funding Polymarket gnosis safe wallet.
    PolyGnosisSafe,
}

#[derive(Debug, Clone)]
pub struct OrderArgs {
    // Prices are multiplied by 1000 (avoids having to deal with tick size).
    pub price:      u32,
    // Sizes are multiplied by 100 (avoids having to deal with lot size).
    pub size:       u32,
    pub buy:        bool,
    pub asset_id:   String,
    // Neg risk is related to the asset, endpoint exists in /markets.rs.
    // Don't retrieve here because it's wasteful.
    pub neg_risk:   bool,
    pub expiration: Option<u64>,
    pub type_:      OrderType,
}

impl ClobClient {

    pub async fn post_order(&self, args: OrderArgs) -> Result<OrderResponse> {
        
        let signed_order = self.create_signed_order(&args)?;
        let order_type = match args.type_ {
            OrderType::FOK => "FOK",
            OrderType::GTC => "GTC",
            OrderType::GTD => "GTD",
        };
        let api_key = &self.get_creds()?.api_key;
        // TODO: change to formatted for more spped?.
        let body = json!({
            "order":     signed_order,
            "owner":     api_key,
            "orderType": order_type,
        }).to_string();

        self.request(RequestArgs {
            method: Method::POST,
            path: "/order",
            queries: None,
            body: Some(body),
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelResponse> {
        self.request(RequestArgs {
            method: Method::DELETE,
            queries: None,
            path: "/order",
            body: Some(format!("{{\"orderID\":\"{}\"}}", order_id)),
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn cancel_orders(&self, order_ids: &[String]) -> Result<CancelResponse> {
        self.request(RequestArgs {
            method: Method::DELETE,
            path: "/orders",
            queries: None,
            body: Some(format!("{:?}", order_ids)),
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn cancel_all(&self) -> Result<CancelResponse> {
        self.request(RequestArgs {
            method: Method::DELETE,
            path: "/cancel-all",
            queries: None,
            body: None,
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn cancel_orders_in_market(&self, market_id: &str) -> Result<CancelResponse> {
        self.request(RequestArgs {
            method: Method::DELETE,
            path: "/cancel-market-orders",
            queries: None,
            body: Some(format!("{{\"market\":\"{}\"}}", market_id)),
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn get_order(&self, order_id: &str) -> Result<OpenOrder> {
        self.request(RequestArgs {
            method: Method::GET,
            path: format!("/data/order/{}", order_id).as_str(),
            queries: None,
            body: None,
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn get_active_orders(
        &self, 
        next_cursor: Option<&str>,
        asset_id: Option<&str>,
        market: Option<&str>,
    ) -> Result<OpenOrders> {

        let mut queries = vec![("next_cursor", next_cursor.unwrap_or("MA=="))];
        if let Some(asset_id) = asset_id {
            queries.push(("asset_id", asset_id));
        }
        if let Some(market) = market {
            queries.push(("market", market));
        }

        self.request(RequestArgs {
            method: Method::GET,
            path: "/data/orders",
            queries: Some(&queries),
            body: None,
            auth_level: AuthLevel::L2,
        }).await
    }

    pub async fn is_order_scoring(&self, order_id: &str) -> Result<bool> {
        
        #[derive(Debug, serde::Deserialize)]
        struct Resp {
            pub is_scoring: bool,
        }
        
        Ok(
            self.request::<Resp>(RequestArgs {
                method: Method::GET,
                path: "/order-scoring",
                queries: Some(&[("order_id", order_id)]),
                body: None,
                auth_level: AuthLevel::L2,
            }).await?
            .is_scoring
        )
    }

    pub async fn are_orders_scoring(&self, order_ids: &[&str]) -> Result<HashMap<String, bool>> {
        self.request(RequestArgs {
            method: Method::POST,
            path: "/orders-scoring",
            queries: None,
            body: Some(format!("{:?}", order_ids)),
            auth_level: AuthLevel::L2,
        }).await
    }
    
    fn create_signed_order(&self, args: &OrderArgs) -> Result<SignedOrder> {
        
        let (maker_amount, taker_amount) = {
            if args.buy {
                (U256::from(args.price * args.size * 10), U256::from(args.size * 10_000))
            } else {
                (U256::from(args.size * 10_000), U256::from(args.price * args.size * 10))
            }
        };

        // Unwrap safe, checked prior to calling.
        let signer_address = self.get_signer()?.address();
        let (maker_address, sig_type) = if let Some(proxy) = &self.proxy {
            (proxy.address, proxy.sig_type)
        } else {
            (signer_address, SignatureType::EOA)
        };

        // TODO: is this right?
        let salt: u32 = rand::thread_rng().gen();
    
        let raw_order = Order {
            takerAmount:    taker_amount,
            makerAmount:    maker_amount,
            salt:           U256::from(salt),
            maker:          maker_address,
            signer:         signer_address,
            taker:          PUBLIC_TAKER_ADDRESS,
            tokenId:        U256::from_str_radix(&args.asset_id, 10)?,
            expiration:     U256::from(args.expiration.unwrap_or(0)),
            nonce:          U256::ZERO,
            feeRateBps:     U256::ZERO,
            signatureType:  sig_type as u8,
            side:           if args.buy { 0 } else { 1 },
        };
    
        auth::sign_order(self.get_signer()?, raw_order, args.neg_risk)
    }
}

