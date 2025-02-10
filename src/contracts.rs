use alloy::primitives::Address;
use crate::{Result, Error};

pub const SUPPORTED_CHAIN_IDS: [u64; 2] = [137, 80002];

#[allow(unused)]
pub struct ContractConfig {
    pub exchange:           Address,
    pub neg_risk_adapter:   Address,
    pub neg_risk_exchange:  Address,
    pub collateral:         Address,
    pub conditional_tokens: Address,
}

pub const MATIC_CONTRACTS: ContractConfig = ContractConfig {
    exchange:           Address::new([75, 251, 65, 213, 179, 87, 13, 239, 208, 60, 57, 169, 164, 216, 222, 107, 216, 184, 152, 46]),
    neg_risk_adapter:   Address::new([217, 30, 128, 207, 46, 123, 226, 225, 98, 198, 81, 60, 237, 6, 241, 221, 13, 163, 82, 150]),
    neg_risk_exchange:  Address::new([197, 213, 99, 163, 106, 231, 129, 69, 196, 90, 80, 19, 77, 72, 161, 33, 82, 32, 248, 10]),
    collateral:         Address::new([39, 145, 188, 161, 242, 222, 70, 97, 237, 136, 163, 12, 153, 167, 169, 68, 154, 168, 65, 116]),
    conditional_tokens: Address::new([77, 151, 220, 217, 126, 201, 69, 244, 12, 246, 95, 135, 9, 122, 206, 94, 160, 71, 96, 69]),
};

pub const AMOY_CONTRACTS: ContractConfig = ContractConfig {
    exchange:           Address::new([223, 224, 46, 182, 115, 53, 56, 248, 234, 53, 213, 133, 175, 141, 229, 149, 138, 217, 158, 64]),
    neg_risk_adapter:   Address::new([217, 30, 128, 207, 46, 123, 226, 225, 98, 198, 81, 60, 237, 6, 241, 221, 13, 163, 82, 150]),
    neg_risk_exchange:  Address::new([197, 213, 99, 163, 106, 231, 129, 69, 196, 90, 80, 19, 77, 72, 161, 33, 82, 32, 248, 10]),
    collateral:         Address::new([156, 78, 23, 3, 71, 110, 135, 80, 112, 238, 37, 181, 106, 88, 176, 8, 207, 184, 250, 120]),
    conditional_tokens: Address::new([105, 48, 143, 181, 18, 81, 142, 57, 249, 177, 97, 18, 250, 141, 153, 79, 78, 43, 248, 187]),
};

pub fn get_contracts(chain_id: u64) -> Result<ContractConfig> {
    match chain_id {
        137   => Ok(MATIC_CONTRACTS),
        80002 => Ok(AMOY_CONTRACTS),
        _     => Err(Error::InvalidChainId),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn test_matic_contract_correctness() {
        assert_eq!(MATIC_CONTRACTS.exchange,           Address::from_str("0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E").unwrap());
        assert_eq!(MATIC_CONTRACTS.neg_risk_adapter,   Address::from_str("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296").unwrap());
        assert_eq!(MATIC_CONTRACTS.neg_risk_exchange,  Address::from_str("0xC5d563A36AE78145C45a50134d48A1215220f80a").unwrap());
        assert_eq!(MATIC_CONTRACTS.collateral,         Address::from_str("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174").unwrap());
        assert_eq!(MATIC_CONTRACTS.conditional_tokens, Address::from_str("0x4D97DCd97eC945f40cF65F87097ACe5EA0476045").unwrap());
    }

    #[test]
    fn test_amoy_contract_correctness() {
        assert_eq!(AMOY_CONTRACTS.exchange,           Address::from_str("0xdFE02Eb6733538f8Ea35D585af8DE5958AD99E40").unwrap());
        assert_eq!(AMOY_CONTRACTS.neg_risk_adapter,   Address::from_str("0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296").unwrap());
        assert_eq!(AMOY_CONTRACTS.neg_risk_exchange,  Address::from_str("0xC5d563A36AE78145C45a50134d48A1215220f80a").unwrap());
        assert_eq!(AMOY_CONTRACTS.collateral,         Address::from_str("0x9c4e1703476e875070ee25b56a58b008cfb8fa78").unwrap());
        assert_eq!(AMOY_CONTRACTS.conditional_tokens, Address::from_str("0x69308FB512518e39F9b16112fA8d994F4e2Bf8bB").unwrap());
    }
}
