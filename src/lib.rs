
mod error;
mod auth;
mod client;
mod contracts;
pub mod schema;

pub use error::{Error, Result};
pub use client::{ClobClient, OrderArgs, OrderType, SignatureType, ApiCreds};

/* 
String formatting is much faster than using serde_json.
We can make signing async
*/
