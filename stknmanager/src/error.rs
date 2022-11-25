use cosmwasm_std::{StdError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("Disabled")]
    Disabled {},

    // #[error("PoolAndTokenMismatch")]
    // PoolAndTokenMismatch {},

    // #[error("NativeInputZero")]
    // NativeInputZero {},

    // #[error("TokenTypeMismatch")]
    // TokenTypeMismatch {},
    
    // #[error("Cw20InputZero")]
    // Cw20InputZero {},

    
    
    #[error("InvalidTokenReplyId")]
    InvalidTokenReplyId {},
    
    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidInput")]
    InvalidInput {},

    #[error("Not STKN or PUSD")]
    UnacceptableToken {},

    #[error("InsufficientStkn")]
    InsufficientStkn {},

    #[error("Not enough STKN")]
    NotEnoughStkn {},

    #[error("Map2List failed")]
    Map2ListFailed {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    
}
