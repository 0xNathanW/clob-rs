use alloy::{hex::ToHexExt, primitives::U256, signers::{k256::sha2, SignerSync}, sol, sol_types::eip712_domain};
use alloy_signer_local::PrivateKeySigner;
use hmac::Mac;
use reqwest::header::HeaderMap;
use chrono::prelude::*;
use base64::prelude::*;
use crate::{client::ApiCreds, contracts::get_contracts, schema::{Order, SignedOrder}, Error};
use super::Result;

const ATTEST_MSG: &str = "This message attests that I control the given wallet";
const PROTOCOL_NAME: &str = "Polymarket CTF Exchange";
const PROTOCOL_VERSION: &str = "1";

sol! {
    struct ClobAuth {
        address address;
        string  timestamp;
        uint256 nonce;
        string  message;
    }
}

pub fn l1_headers(signer: &PrivateKeySigner, nonce: U256) -> Result<HeaderMap> {
    
    let timestamp = Utc::now().timestamp();
    let sig = sign_attest_msg(signer, timestamp, nonce)?;
    
    let mut headers = HeaderMap::new();
    headers.insert("POLY_ADDRESS",   signer.address().to_string().parse()?);
    headers.insert("POLY_SIGNATURE", sig.parse()?);
    headers.insert("POLY_TIMESTAMP", timestamp.to_string().parse()?);
    headers.insert("POLY_NONCE",     nonce.to_string().parse()?);
    
    Ok(headers)
}

fn sign_attest_msg(signer: &PrivateKeySigner, timestamp: i64, nonce: U256) -> Result<String> {

    let domain = eip712_domain! {
        name:     "ClobAuthDomain",
        version:  "1",
        chain_id: signer.chain_id().unwrap_or(137),
    };

    let data = ClobAuth {
        address:    signer.address(),
        timestamp:  timestamp.to_string(),
        message:    ATTEST_MSG.to_string(),
        nonce,
    };

    Ok(
        signer
            .sign_typed_data_sync(&data, &domain)?
            .as_bytes()
            .encode_hex_with_prefix()
    )
}

pub fn l2_headers(
    signer: &PrivateKeySigner, 
    creds:  &ApiCreds,
    method: &reqwest::Method,
    path:   &str,
    body:   Option<&str>,
) -> Result<HeaderMap> {

    let timestamp = Utc::now().timestamp();
    let body = body.unwrap_or("");
    let pre_hash = format!("{timestamp}{method}{path}{body}");
    let sig = hmac_signature(&pre_hash, &creds.secret)?;

    let mut headers = HeaderMap::new();
    headers.insert("POLY_ADDRESS",    signer.address().to_string().parse()?);
    headers.insert("POLY_SIGNATURE",  sig.parse()?);
    headers.insert("POLY_TIMESTAMP",  timestamp.to_string().parse()?);
    headers.insert("POLY_API_KEY",    creds.api_key.parse()?);
    headers.insert("POLY_PASSPHRASE", creds.passphrase.parse()?);
    Ok(headers)
}

fn hmac_signature(pre_hash: &str, secret: &str) -> Result<String> {
    let decoded_secret = BASE64_URL_SAFE.decode(secret).map_err(|_| Error::InvalidSecret)?;
    // Cannot fail, SHA-256 can accept key of any length.
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&decoded_secret).unwrap();
    mac.update(pre_hash.as_bytes());
    let result = mac.finalize();
    Ok(BASE64_URL_SAFE.encode(result.into_bytes()))
}

pub fn sign_order(signer: &PrivateKeySigner, order: Order, neg_risk: bool) -> Result<SignedOrder> {

    let chain_id = signer.chain_id().unwrap_or(137);
    let verifying_contract = if neg_risk {
        get_contracts(chain_id)?.neg_risk_exchange
    } else {
        get_contracts(chain_id)?.exchange
    };
    let domain = eip712_domain! {
        name:               PROTOCOL_NAME,
        version:            PROTOCOL_VERSION,
        chain_id:           chain_id,
        verifying_contract: verifying_contract,
    };

    let signature = signer
        .sign_typed_data_sync(&order, &domain)?
        .as_bytes()
        .encode_hex_with_prefix();

    Ok(SignedOrder {
        order,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use alloy::{primitives::Address, signers::Signer};
    use super::*;

    // Known private key for testing.
    const PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const CHAIN_ID: u64 = 80002;

    fn get_signer() -> PrivateKeySigner {
        PrivateKeySigner::from_str(PRIVATE_KEY).unwrap().with_chain_id(Some(CHAIN_ID))
    }

    #[test]
    fn test_sign_attest_msg() {
        let signer = get_signer();
        let sig = sign_attest_msg(&signer, 10000000, U256::from(23)).unwrap();
        assert_eq!(sig, "0xf62319a987514da40e57e2f4d7529f7bac38f0355bd88bb5adbb3768d80de6c1682518e0af677d5260366425f4361e7b70c25ae232aff0ab2331e2b164a1aedc1b");
    }

    #[test]
    fn test_hmac_sig() {
        let timestamp = 1000000;
        let method = "test-sign";
        let path = "/orders";
        let body = "{'hash': \"0x123\"}".to_string();
        let body = body.replace("'", "\""); 
        let prehash = format!("{timestamp}{method}{path}{body}");
        let secret = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let sig = hmac_signature(&prehash, &secret).unwrap();
        assert_eq!(sig, "ZwAdJKvoYRlEKDkNMwd5BuwNNtg93kNaR_oU2HrfVvc=");
    }

    #[test]
    fn test_sign_order() {
        let signer = get_signer();
        let address = signer.address();
        let order = Order {
            salt:           U256::from(479249096354_u64),
            maker:          address,
            signer:         address,
            taker:          Address::ZERO,
            tokenId:        U256::from(1234),
            makerAmount:    U256::from(100000000),
            takerAmount:    U256::from(50000000),
            expiration:     U256::ZERO,
            nonce:          U256::ZERO,
            feeRateBps:     U256::from(100),
            side:           0,
            signatureType:  0,
        };
        let signed_order = sign_order(&signer, order, false).unwrap();
        assert_eq!(signed_order.signature, "0x302cd9abd0b5fcaa202a344437ec0b6660da984e24ae9ad915a592a90facf5a51bb8a873cd8d270f070217fea1986531d5eec66f1162a81f66e026db653bf7ce1c");
    }
}