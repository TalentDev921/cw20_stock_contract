#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg, WasmQuery, QueryRequest, CosmosMsg, Order, Addr, SubMsg, ReplyOn, Reply, Storage
};
use cw_utils::parse_reply_instantiate_data;
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse, Logo, Denom};
use cw20_base::msg::{InstantiateMsg as Cw20InstantiateMsg, InstantiateMarketingInfo};
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, ReceiveMsg
};

use crate::state::{
    Config, CONFIG, STOCKS
};
use cw20_stock::msg::{InstantiateMsg as StockInstantiateMsg, InstantiateMarketingInfo as StockInstantiateMarketingInfo};
use stknstaking::msg::{InstantiateMsg as StakingInstantiateMsg};

use crate::util;
use crate::util::{ManagerConfigResponse, ManagerQueryMsg, StockListResponse, StockInfo, StockConfigResponse, StockQueryMsg, NORMAL_DECIMAL};
// Version info, for migration info
const CONTRACT_NAME: &str = "stknmanager";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");



const INSTANTIATE_PUSD_ID: u64 = 1;
const INSTANTIATE_STAKING_ID: u64 = 2;
const INSTANTIATE_STOCK_ID: u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        stkn_address: msg.stkn_address.clone(),
        pusd_address: msg.stkn_address.clone(),
        staking_address: msg.stkn_address.clone(),

        staking_code_id: msg.staking_code_id,
        cw20_code_id: msg.cw20_code_id,
        stock_code_id: msg.stock_code_id,
        pool_code_id: msg.pool_code_id,

        shorting_code_id: msg.shorting_code_id,
        trading_code_id: msg.trading_code_id,
        providing_code_id: msg.providing_code_id,

        price: msg.price,
        max_stock_id: 0u32,
        enabled: true,

        providing_sync_interval: msg.providing_sync_interval
    };
    CONFIG.save(deps.storage, &config)?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.cw20_code_id,
            funds: vec![],
            admin: Some(info.sender.clone().into()),
            label: String::from("PUSD : stable coin of Stocken"),
            msg: to_binary(&Cw20InstantiateMsg {
                name: String::from("PUSD"),
                symbol: String::from("PUSD"),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.clone().into(),
                    cap: None
                }),
                marketing: Some(InstantiateMarketingInfo {
                    project: None,
                    description: None,
                    logo: Some(Logo::Url(msg.pusd_url)),
                    marketing: Some(env.contract.address.clone().into())
                })
            })?,
        }.into(),
        id: INSTANTIATE_PUSD_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))
}


// Reply callback triggered from cw20-base contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    
    let mut cfg: Config = CONFIG.load(deps.storage)?;
    let reply = parse_reply_instantiate_data(msg.clone()).unwrap();
    let contract_address = Addr::unchecked(reply.contract_address);

    if msg.id == INSTANTIATE_PUSD_ID {
        cfg.pusd_address = contract_address.clone();
        CONFIG.save(deps.storage, &cfg)?;

        //Instantiate Staking
        let sub_msg: Vec<SubMsg> = vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: cfg.staking_code_id,
                funds: vec![],
                admin: Some(cfg.owner.clone().into()),
                label: String::from("STKN Staking"),
                msg: to_binary(&StakingInstantiateMsg {
                    owner: cfg.owner.clone().into(),
                    manager_address: env.contract.address.clone(),
                    lock_days: vec![30, 60, 120],
                    ratios: vec![20000, 40000, 80000],
                })?,
            }.into(),
            id: INSTANTIATE_STAKING_ID,
            gas_limit: None,
            reply_on: ReplyOn::Success,
        }];

        return Ok(Response::new()
            .add_submessages(sub_msg)
            .add_attribute("action", "instantiate_pusd")
            .add_attribute("pusd_address", cfg.pusd_address.clone())
        );
    } else if msg.id == INSTANTIATE_STAKING_ID {
        
        cfg.staking_address = contract_address.clone();
        CONFIG.save(deps.storage, &cfg)?;

        return Ok(Response::new()
            .add_attribute("action", "instantiate_staking")
            .add_attribute("staking_address", cfg.staking_address.clone())
        );
    }else if msg.id == INSTANTIATE_STOCK_ID {
        cfg.max_stock_id += 1;
        CONFIG.save(deps.storage, &cfg)?;

        let stock_address = contract_address.clone();

        let stock_response: StockConfigResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: stock_address.clone().into(),
            msg: to_binary(&StockQueryMsg::Config {})?,
        }))?;

        STOCKS.save(deps.storage, cfg.max_stock_id, 
            &StockInfo {
                id: cfg.max_stock_id,
                stock_address: stock_address.clone(),
                pool_address: stock_response.pool_address.clone(),
                shorting_address: stock_response.shorting_address.clone(),
                trading_address: stock_response.trading_address.clone(),
                providing_address: stock_response.providing_address.clone()
            }
        )?;

        return Ok(Response::new()
            .add_attribute("action", "instantiate_stock")
            .add_attribute("stock_address", stock_address.clone())
        );
    } else {
        return Err(ContractError::InvalidTokenReplyId {});
    }
    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => execute_update_owner(deps, env, info, owner),
        ExecuteMsg::UpdateEnabled { enabled } => execute_update_enabled(deps, env, info, enabled),
        ExecuteMsg::UpdatePrice { price } => execute_update_price(deps, env, info, price),
        ExecuteMsg::RemoveStock {id} => execute_remove_stock(deps, env, info, id),
        ExecuteMsg::RemoveAllStocks {  } => execute_remove_all_stocks(deps, env, info),
        ExecuteMsg::AddStock{ name, symbol, url} => execute_add_stock(deps, env, info, name, symbol, url),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::MintPusd { id, recipient, amount } => execute_mint_pusd(deps, env, info.sender.clone(), id, recipient, amount),
        ExecuteMsg::MintStock { id, recipient, amount } => execute_mint_stock(deps, env, info.sender.clone(), id, recipient, amount),
        ExecuteMsg::TransferStkn { id, recipient, amount } => execute_transfer_stkn(deps, env, info.sender.clone(), id, recipient, amount),
    }
}


pub fn execute_update_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
) -> Result<Response, ContractError> {
    // authorize owner
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = owner.clone();
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_owner").add_attribute("owner", owner.clone()))
}

pub fn execute_update_enabled (
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    enabled: bool
) -> Result<Response, ContractError> {
    // authorize owner
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.enabled = enabled;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_enabled"))
}
pub fn execute_add_stock(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    symbol: String,
    url: String,
) -> Result<Response, ContractError> {

    
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
    
    let cfg = CONFIG.load(deps.storage)?;
    // Instantiate Stock Contract

    let mut sub_msg: Vec<SubMsg> = vec![];
    
    sub_msg.push(SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: cfg.stock_code_id,
            funds: vec![],
            admin: Some(cfg.owner.clone().into()),
            label: String::from("Stock Token : ") + name.as_str(),
            msg: to_binary(&StockInstantiateMsg {
                name,
                symbol,
                decimals: 6,
                initial_balances: vec![],
                mint: None,
                marketing: Some(StockInstantiateMarketingInfo {
                    project: None,
                    description: None,
                    logo: Some(Logo::Url(url)),
                    marketing: Some(cfg.owner.clone().into())
                }),

                id: cfg.max_stock_id + 1,
                owner: cfg.owner.clone(),
                pool_code_id: cfg.pool_code_id,
                shorting_code_id: cfg.shorting_code_id,
                trading_code_id: cfg.trading_code_id,
                providing_code_id: cfg.providing_code_id,
                cw20_code_id: cfg.cw20_code_id,
                
            })?,
        }.into(),
        id: INSTANTIATE_STOCK_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    });

    Ok(Response::new().add_submessages(sub_msg))

}

pub fn execute_update_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    price: Uint128
) -> Result<Response, ContractError> {
    // authorize owner
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
        
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.price = price;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_price")
    .add_attribute("price", price)
)
}

pub fn execute_remove_stock(
    deps: DepsMut,
    env: Env, 
    info: MessageInfo,
    id: u32
) -> Result<Response, ContractError>{
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
    
    STOCKS.remove(deps.storage, id);
    Ok(Response::new()
        .add_attribute("action", "remove_stock")
        .add_attribute("id", id.to_string())
    )
}

pub fn execute_remove_all_stocks(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    // authorize owner
    util::check_owner(deps.querier, env.contract.address.clone(), info.sender.clone())?;
    

    let stocks:StdResult<Vec<_>> = STOCKS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| map_stock(item))
        .collect();

    if stocks.is_err() {
        return Err(ContractError::Map2ListFailed {})
    }
    
    for item in stocks.unwrap() {
        STOCKS.remove(deps.storage, item.id);
    }
    
    Ok(Response::new().add_attribute("action", "remove_all_stock"))
}


pub fn execute_receive(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo, 
    wrapper: Cw20ReceiveMsg
) -> Result<Response, ContractError> {
    
    util::check_enabled(deps.querier, env.contract.address.clone())?;

    let cfg = CONFIG.load(deps.storage)?;
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;
    
    
    if wrapper.amount == Uint128::zero() {
        return Err(ContractError::InvalidInput {});
    }

    match msg {
        ReceiveMsg::FundStkn {} => {
            // Update Amount
            if info.sender != cfg.stkn_address {
                return Err(ContractError::UnacceptableToken {});
            }
            
            return Ok(Response::new()
                .add_attributes(vec![
                    attr("action", "fund_stkn"),
                    attr("address", user_addr),
                    attr("amount", wrapper.amount)
                ])); 
            
        },
        ReceiveMsg::Swap {expected_amount} => {
            if info.sender != cfg.stkn_address.clone() && info.sender != cfg.pusd_address.clone() {
                return Err(ContractError::UnacceptableToken {});
            }

            let mut messages:Vec<CosmosMsg> = vec![];
            if info.sender == cfg.stkn_address.clone() {
                // Store stkn and mint according pusd to user_addr
                
                let mut pusd_amount = cfg.price * wrapper.amount / Uint128::from(NORMAL_DECIMAL);

                if expected_amount * Uint128::from(NORMAL_DECIMAL) / cfg.price == wrapper.amount {
                    pusd_amount = expected_amount;
                }

                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: cfg.pusd_address.clone().into(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Mint {
                        recipient: user_addr.clone().into(),
                        amount: pusd_amount
                    })?,
                }));
                return Ok(Response::new()
                    .add_messages(messages)
                    .add_attributes(vec![
                        attr("action", "buy_pusd"),
                        attr("address", user_addr),
                        attr("amount", pusd_amount)
                    ]));

            } else {
                // Burn received pusd and send stkn
                let mut stkn_amount = wrapper.amount * Uint128::from(NORMAL_DECIMAL) / cfg.price ;

                if cfg.price * expected_amount / Uint128::from(NORMAL_DECIMAL) == wrapper.amount {
                    stkn_amount = expected_amount;
                }

                if stkn_amount > util::get_token_amount(deps.querier, Denom::Cw20(cfg.stkn_address.clone()), env.contract.address.clone())? {
                    return Err(ContractError::InsufficientStkn {});
                }
                
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: cfg.pusd_address.clone().into(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Burn {
                        amount: wrapper.amount
                    })?,
                }));
                messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: cfg.stkn_address.clone().into(),
                    funds: vec![],
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: user_addr.clone().into(),
                        amount: stkn_amount
                    })?,
                }));
                return Ok(Response::new()
                    .add_messages(messages)
                    .add_attributes(vec![
                        attr("action", "buy_stkn"),
                        attr("address", user_addr),
                        attr("amount", stkn_amount)
                ]));
            }
        }
    }
    
}

pub fn check_stock_subcontract(
    storage: &mut dyn Storage,
    id: u32,
    address: Addr
) -> Result<Response, ContractError> {
    let config = CONFIG.load(storage)?;
    if config.owner == address || config.staking_address == address || config.stkn_address == address || config.pusd_address == address {
        return Ok(Response::new().add_attribute("action", "check_stock_contracts"));
    }
    let stock_info = STOCKS.load(storage, id)?;
    
    if stock_info.providing_address == address || stock_info.trading_address == address || stock_info.shorting_address == address || stock_info.pool_address == address || stock_info.stock_address == address {
        Ok(Response::new().add_attribute("action", "check_stock_contracts"))
    } else {
        return Err(ContractError::Unauthorized {})
    }
}

pub fn execute_mint_pusd(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    id: u32,
    recipient: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {

    util::check_enabled(deps.querier, env.contract.address.clone())?;
    check_stock_subcontract(deps.storage, id, caller.clone())?;
    
    let mut messages:Vec<CosmosMsg> = vec![];

    let cfg = CONFIG.load(deps.storage)?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.pusd_address.clone().into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: recipient.clone().into(),
            amount
        })?,
    }));

    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "mint_pusd"),
            attr("id", id.to_string()),
            attr("address", recipient.clone()),
            attr("amount", amount)
    ]));
}


pub fn execute_mint_stock(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    id: u32,
    recipient: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {

    util::check_enabled(deps.querier, env.contract.address.clone())?;
    check_stock_subcontract(deps.storage, id, caller.clone())?;
    
    let stock_info = STOCKS.load(deps.storage, id)?;

    let mut messages:Vec<CosmosMsg> = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: stock_info.stock_address.clone().into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Mint {
            recipient: recipient.clone().into(),
            amount
        })?,
    }));

    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "mint_stock"),
            attr("id", id.to_string()),
            attr("address", recipient.clone()),
            attr("amount", amount)
    ]));
}

pub fn execute_transfer_stkn(
    deps: DepsMut,
    env: Env,
    caller: Addr,
    id: u32,
    recipient: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {

    util::check_enabled(deps.querier, env.contract.address.clone())?;
    check_stock_subcontract(deps.storage, id, caller.clone())?;

    let cfg = CONFIG.load(deps.storage)?;

    if util::get_token_amount(deps.querier, Denom::Cw20(cfg.stkn_address.clone()), env.contract.address.clone())? < amount {
        return Err(ContractError::NotEnoughStkn {});
    }
    
    let mut messages:Vec<CosmosMsg> = vec![];

    let cfg = CONFIG.load(deps.storage)?;
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.stkn_address.clone().into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: recipient.clone().into(),
            amount
        })?,
    }));

    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "send_stkn"),
            attr("id", id.to_string()),
            attr("address", recipient.clone()),
            attr("amount", amount)
    ]));
    

}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: ManagerQueryMsg) -> StdResult<Binary> {
    match msg {
        ManagerQueryMsg::Config {} 
            => to_binary(&query_config(deps, env)?),
        ManagerQueryMsg::Stock {id} 
            => to_binary(&query_stock(deps, id)?),
        ManagerQueryMsg::ListStocks {} 
            => to_binary(&query_list_stocks(deps)?),
        ManagerQueryMsg::CheckStockSubcontract {id, address} 
            => to_binary(&query_check_stock_subcontract(deps, id, address)?)
    }
}

pub fn query_config(deps: Deps, env: Env) -> StdResult<ManagerConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ManagerConfigResponse {
        owner: cfg.owner,
        stkn_address: cfg.stkn_address.clone(),
        pusd_address: cfg.pusd_address,
        staking_address: cfg.staking_address,
        
        cw20_code_id: cfg.cw20_code_id,
        stock_code_id: cfg.stock_code_id,
        pool_code_id: cfg.pool_code_id,

        staking_code_id: cfg.staking_code_id,

        shorting_code_id: cfg.shorting_code_id,
        trading_code_id: cfg.trading_code_id,
        providing_code_id: cfg.providing_code_id,

        price: cfg.price,
        stkn_amount: util::get_token_amount(deps.querier, Denom::Cw20(cfg.stkn_address.clone()), env.contract.address.clone()).unwrap(),
        max_stock_id: cfg.max_stock_id,
        enabled: cfg.enabled,
        providing_sync_interval: cfg.providing_sync_interval
    })
}

pub fn query_stock(deps: Deps, id: u32) -> StdResult<StockInfo> {
    
    let stock_info = STOCKS.load(deps.storage, id)?;
    Ok(stock_info)
}

pub fn query_check_stock_subcontract(deps: Deps, id: u32, address: Addr) -> StdResult<bool> {
    
    let stock_info = STOCKS.load(deps.storage, id)?;
    let cfg = CONFIG.load(deps.storage)?;
    if cfg.owner == address || stock_info.providing_address == address || stock_info.trading_address == address || stock_info.shorting_address == address || stock_info.pool_address == address {
        Ok(true)
    } else {
        return Ok(false)
    }
}

pub fn query_list_stocks(deps: Deps) 
-> StdResult<StockListResponse> {
    let stocks:StdResult<Vec<_>> = STOCKS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| map_stock(item))
        .collect();

    Ok(StockListResponse {
        list: stocks?
    })
}

fn map_stock(
    item: StdResult<(u32, StockInfo)>,
) -> StdResult<StockInfo> {
    item.map(|(_id, stock_info)| {
        stock_info
    })
}



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

