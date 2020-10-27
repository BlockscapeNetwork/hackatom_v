use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Coin};

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: CanonicalAddr,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offering {
    pub token_id: String,

    pub contract_addr: CanonicalAddr,

    pub seller: CanonicalAddr,

    pub list_price: Coin,
}

// pub const TOKENS: Map<&str, TokenInfo> = Map::new(b"tokens");
pub const OFFERINGS: Map<(&[u8], &[u8]), Offering> = Map::new(b"offerings");
