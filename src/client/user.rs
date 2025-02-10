use alloy::primitives::U256;
use reqwest::Method;
use serde::Deserialize;
use crate::{Error, Result};
use super::{ApiCreds, AuthLevel, ClobClient, RequestArgs};

impl ClobClient {

    pub async fn get_api_keys(&self) -> Result<Vec<String>> {

        #[derive(Debug, Deserialize)]
        struct Resp {
            #[serde(rename = "apiKeys")]
            api_keys: Vec<String>,
        }

        Ok(
            self.request::<Resp>(RequestArgs {
                method: Method::GET,
                path: "/auth/api-keys",
                queries: None,
                body: None,
                auth_level: AuthLevel::L2,
            })
            .await?
            .api_keys
        )
    }

    pub async fn derive_creds_from_nonce(&self, nonce: U256) -> Result<ApiCreds> {
        self.request(RequestArgs {
            method: Method::GET,
            path: "/auth/derive-api-key",
            queries: None,
            body: None,
            auth_level: AuthLevel::L1 { nonce },
        }).await
    }

    pub async fn create_creds_with_nonce(&self, nonce: U256) -> Result<ApiCreds> {
        self.request(RequestArgs {
            method: Method::POST,
            path: "/auth/api-key",
            queries: None,
            body: None,
            auth_level: AuthLevel::L1 { nonce },
        }).await
    }

    // Deletes the API key used to authenticate the request.
    pub async fn delete_api_key(&self) -> Result<()> {

        let resp = self.request::<String>(RequestArgs {
            method: Method::DELETE,
            path: "/auth/api-key",
            queries: None,
            body: None,
            auth_level: AuthLevel::L2,
        }).await?;

        if resp != "OK" {
            Err(Error::ApiKeyDeleteFailed)
        } else {
            Ok(())
        }
    }

    pub async fn require_cert(&self) -> Result<bool> {
        let address = self.get_signer()?.address().to_string();

        #[derive(Deserialize, Debug)]
        struct Resp {
            cert_required: bool,
        }

        Ok(
            self.request::<Resp>(RequestArgs {
                method: reqwest::Method::GET,
                path: "/auth/ban-status/cert-required",
                queries: Some(&[("address", &address)]),
                body: None,
                auth_level: AuthLevel::None,
            }).await?
            .cert_required
        )
    }

    pub async fn get_balance_allowance(&self) -> Result<()> {
        let resp = self.request::<String>(RequestArgs {
            method: Method::GET,
            path: "/balance-allowance",
            queries: None,
            body: None,
            auth_level: AuthLevel::L2,
        }).await?;

        println!("{}", resp);
        Ok(())
    }
    // TODO: Get trades.
}
