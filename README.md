# clob-rs
A PolyMarket CLOB API client in rust.

## Usage
2 options:
1. Build a client using:
- `fn new(base_url: &str) -> Self`
- `fn with_signer(mut self, private_key: &str, chain_id: u64) -> Result<Self>` for Level 1 access.
- `fn with_creds(mut self, creds: ApiCreds) -> Self` for Level 2 access.
2. Use `fn from_env() -> Result<Self>` where environment variables required are as follows:
- `CLOB_URL`: base http url.
- `PRIVATE_KEY`: wallet private key for signing.
- `CHAIN_ID`: associated blockchain id, 137 for polygon.
- `PROXY`: address of proxy wallet.
- 'SIG_TYPE': signature type, 1 for EOA, 2 for PolyProxy, 3 for PolyGnosisSafe.

All endpoint methods use the `pub async fn request<T: serde::de::DeserializeOwned + std::fmt::Debug>(&self, args: RequestArgs<'_>) -> Result<T>` function.  Therefore future or changed endpoints can be accessed easily.

Find example usage in the Examples directory.
