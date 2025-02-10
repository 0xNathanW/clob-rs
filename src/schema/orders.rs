use std::collections::HashMap;
use alloy::{sol, primitives::U256};
use serde::{Serializer, Serialize, Deserialize};
use crate::error::OrderError;

// Intermediate order struct.
sol! {
    #[derive(Debug, Serialize)]
    struct Order {
        
        #[serde(serialize_with = "serialize_u256_as_u128")]
        uint256 salt;

        address maker;
        
        address signer;
        
        address taker;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 tokenId;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 makerAmount;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 takerAmount;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 expiration;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 nonce;
        
        #[serde(serialize_with = "serialize_u256_as_dec_str")]
        uint256 feeRateBps;
        
        #[serde(serialize_with = "serialize_side")]
        uint8   side;
 
        uint8   signatureType;
    }
}

// Add this serializer function
fn serialize_u256_as_dec_str<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn serialize_u256_as_u128<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u128(value.to::<u128>())
}

fn serialize_side<S>(value: &u8, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        0 => serializer.serialize_str("BUY"),
        1 => serializer.serialize_str("SELL"),
        _ => Err(serde::ser::Error::custom("invalid side, must be 0 for buy, 1 for sell"))
    }
}


// Signed intermediate order struct.
#[derive(Debug, Serialize)]
pub struct SignedOrder {
    #[serde(flatten)]
    pub order:     Order,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub success:            bool,
    pub error_msg:          String,
    #[serde(rename = "orderID")]
    pub order_id:           String,
    pub transaction_hashes: Option<Vec<String>>,
    pub status:             String,
    pub making_amount:      String,
    pub taking_amount:      String,
}

impl OrderResponse {
    pub fn check_success(&self) -> Result<(), OrderError> {
        if !self.success {
            Err(OrderError::ServerSideError(self.error_msg.clone()))
        } else if !self.error_msg.is_empty() {
            Err(OrderError::ClientSideError(self.error_msg.clone()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelResponse {
    pub canceled:     Vec<String>,
    pub not_canceled: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrders {
    pub data:        Vec<OpenOrder>,
    pub next_cursor: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrder {
    pub asset_id:         String,
    pub associate_trades: Vec<String>,
    pub created_at:       u64,
    pub expiration:       String,
    pub id:               String,
    pub maker_address:    String,
    pub market:           String,
    pub order_type:       String,
    pub original_size:    String,
    pub outcome:          String,
    pub owner:            String,
    pub price:            String,
    pub side:             String,
    pub size_matched:     String,
    pub status:           String,
}