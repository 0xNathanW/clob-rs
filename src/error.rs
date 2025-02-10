use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {

    #[error("invalid private key")]
    InvalidPrivateKey,

    #[error("invalid secret, unable to base64 decode")]
    InvalidSecret,

    #[error("invalid or unsupported chain id, must be a number")]
    InvalidChainId,

    #[error("invalid signature type")]
    InvalidSignatureType,

    #[error("invalid proxy address")]
    InvalidProxyAddress,

    
    #[error("environment variable error: {0}")]
    EnvVariableError(#[from] std::env::VarError),

    #[error("requires signer")]
    SignerRequired,

    #[error("requires creds")]
    CredsRequired,

    #[error("signing error: {0}")]
    SigningError(#[from] alloy::signers::Error),

    #[error("invalid request header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("API error: {status_code} - {msg}")]
    ApiError {
        status_code: u16,
        msg:         String,
    },

    #[error("request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("alloy primitive parse error: {0}")]
    ParseError(#[from] alloy::primitives::ruint::ParseError),

    #[error("API key deletion failed")]
    ApiKeyDeleteFailed,
}

// API responds with an error to order request.
#[derive(Error, Debug)]
pub enum OrderError {
    
    #[error("server side error: {0}")]
    ServerSideError(String),

    #[error("client side error: {0}")]
    ClientSideError(String),
}

#[derive(Error, Debug)]
pub enum MarketError {
    
    #[error("{0} token not found")]
    TokenNotFound(String),

    #[error("asset id could not be parsed to U256")]
    InvalidAssetId,

    #[error("could not parse date: {0}")]
    InvalidDate(String),
}

