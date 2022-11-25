use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg};
use cosmwasm_std::{Uint128, Addr};

// use marble_collection::msg::{InstantiateMsg as CollectionInstantiateMsg, ExecuteMsg as CollectionExecuteMsg};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub stkn_address: Addr,

    pub cw20_code_id: u64,
    pub stock_code_id: u64,
    pub pool_code_id: u64,
    
    pub staking_code_id: u64,
    pub shorting_code_id: u64,
    pub trading_code_id: u64,
    pub providing_code_id: u64,
    
    pub price: Uint128,
    pub pusd_url: String,

    pub providing_sync_interval: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr
    },
    UpdateEnabled {
        enabled: bool
    },
    UpdatePrice {
        price: Uint128
    },
    RemoveStock {
        id: u32
    },
    RemoveAllStocks {
    },
    AddStock {
        name: String,
        symbol: String,
        url: String
    },
    Receive(Cw20ReceiveMsg),
    MintPusd {
        id: u32,
        recipient: Addr,
        amount: Uint128
    },
    MintStock {
        id: u32,
        recipient: Addr,
        amount: Uint128
    },
    TransferStkn {
        id: u32,
        recipient: Addr,
        amount: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    FundStkn {},
    Swap {
        expected_amount: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
