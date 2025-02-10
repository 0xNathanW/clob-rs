use std::{str::FromStr, env::var};
use alloy::signers::{local::PrivateKeySigner, Signer};
use alloy::primitives::{Address, U256};
use crate::{auth, contracts::SUPPORTED_CHAIN_IDS, Error, Result};

mod markets;
mod orders;
mod user;

pub use orders::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiCreds {
    #[serde(rename = "apiKey")]
    pub api_key:    String,
    pub secret:     String,
    pub passphrase: String,
}

impl ApiCreds {
    pub fn from_env() -> Result<Self> {
        Ok(Self { 
            api_key:    var("API_KEY")?, 
            secret:     var("SECRET")?, 
            passphrase: var("PASSPHRASE")? 
        })
    }
}

#[derive(Debug, Clone)]
struct RequestArgs<'a> {
    pub method:     reqwest::Method,
    pub path:       &'a str,
    pub queries:    Option<&'a [(&'a str, &'a str)]>,
    pub body:       Option<String>,
    pub auth_level: AuthLevel,
}

#[derive(Debug, Clone, Copy)]
enum AuthLevel {
    None,
    // Private key authentication.
    L1 {
        nonce: U256,
    },
    // API authentication.
    L2,
}

#[derive(Debug, Clone)]
pub struct ClobClient {
    // HTTP client.
    client:   reqwest::Client,
    // Base HTTP url.
    base_url: String,
    // Chain signer.
    signer:   Option<PrivateKeySigner>,
    // Polymarket API credentials.
    creds:    Option<ApiCreds>,
    // If none sig type is EOA.
    proxy:    Option<Proxy>,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub address:  Address,
    pub sig_type: SignatureType,
}

impl ClobClient {

    pub fn new(base_url: &str) -> Self {
        Self { 
            client:   reqwest::Client::new(),
            base_url: base_url.to_string(),
            signer:   None,
            creds:    None,
            proxy:    None,
        }
    }

    pub fn with_signer(mut self, private_key: &str, chain_id: u64) -> Result<Self> {
        if !SUPPORTED_CHAIN_IDS.contains(&chain_id) {
            return Err(Error::InvalidChainId);
        }
        self.signer = Some(
            PrivateKeySigner::from_str(private_key)
                .map_err(|_| Error::InvalidPrivateKey)?
                .with_chain_id(Some(chain_id))
        );
        Ok(self)
    }

    pub fn with_creds(mut self, creds: ApiCreds) -> Self {
        self.creds = Some(creds);
        self
    }

    pub fn with_proxy(mut self, address: &str, sig_type: SignatureType) -> Result<Self> {
        if sig_type == SignatureType::EOA {
            return Err(Error::InvalidSignatureType);
        }
        let address = Address::from_str(address).map_err(|_| Error::InvalidProxyAddress)?;
        self.proxy = Some(Proxy { address, sig_type });
        Ok(self)
    }

    pub fn get_signer(&self) -> Result<&PrivateKeySigner> {
        self.signer
            .as_ref()
            .ok_or(Error::SignerRequired)
    }

    pub fn get_creds(&self) -> Result<&ApiCreds> {
        self.creds
            .as_ref()
            .ok_or(Error::CredsRequired)
    }

    // Create a new client from environment variables.
    pub fn from_env() -> Result<Self> {

        let chain_id = var("CHAIN_ID")?.parse::<u64>().map_err(|_| Error::InvalidChainId)?;
        let sig_type = match var("SIG_TYPE")?.parse::<u8>().map_err(|_| Error::InvalidSignatureType)? {
            0 => SignatureType::EOA,
            1 => SignatureType::PolyProxy,
            2 => SignatureType::PolyGnosisSafe,
            _ => return Err(Error::InvalidSignatureType),
        };

        Self::new(&var("CLOB_URL")?)
            .with_signer(&var("PRIVATE_KEY")?, chain_id)?
            .with_creds(ApiCreds::from_env()?)
            .with_proxy(&var("PROXY")?, sig_type)
    }

    #[tracing::instrument(skip(self))]
    async fn request<T: serde::de::DeserializeOwned + std::fmt::Debug>(&self, args: RequestArgs<'_>) -> Result<T> {
        
        let headers = match args.auth_level {
            AuthLevel::None => None,
            AuthLevel::L1 { nonce } => {
                let signer = self.get_signer()?;
                Some(auth::l1_headers(signer, nonce)?)
            },
            AuthLevel::L2 => {
                let signer = self.get_signer()?;
                let creds = self.get_creds()?;
                Some(auth::l2_headers(signer, creds, &args.method, args.path, args.body.as_deref())?)
            },
        };

        let mut req = self.client.request(args.method, format!("{}{}", &self.base_url, args.path));
        if let Some(headers) = headers {
            req = req.headers(headers);
        }
        if let Some(body) = args.body {
            req = req.body(body);
        }
        if let Some(queries) = args.queries {
            req = req.query(queries);
        }
        tracing::debug!("request: {:#?}", req);

        let resp = req.send().await?;

        if !resp.status().is_success() {
            let status_code = resp.status().as_u16();
            let msg: String = resp.text().await?;
            return Err(Error::ApiError { status_code, msg });
        }

        let out = resp.json().await?;
        tracing::debug!("response: {:#?}", out);
        Ok(out)
    }
}