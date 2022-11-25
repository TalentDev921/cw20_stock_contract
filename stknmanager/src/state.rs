use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use crate::util::StockInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub stkn_address: Addr,
    pub pusd_address: Addr,
    pub staking_address: Addr,

    pub cw20_code_id: u64,
    pub stock_code_id: u64,

    pub staking_code_id: u64,
    pub pool_code_id: u64, 
    pub shorting_code_id: u64,
    pub trading_code_id: u64,
    pub providing_code_id: u64,

    pub price: Uint128,

    pub max_stock_id: u32,
    pub enabled: bool,

    pub providing_sync_interval: u64
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const STOCKS_KEY: &str = "stocks";
pub const STOCKS: Map<u32, StockInfo> = Map::new(STOCKS_KEY);
