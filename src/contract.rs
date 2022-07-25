use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Decimal};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, GasPrice, ConfigResponse, GAS_MAP};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:gas-price-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => try_update_config(deps, info, owner),
        ExecuteMsg::UpdateGasPrice { token, value } => try_update_gas_price(deps, info, env, token, value),
    }
}

pub fn try_update_config(deps: DepsMut, info: MessageInfo, owner: String) -> Result<Response, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    };
    state.owner = deps.api.addr_validate(&owner)?;
    CONFIG.save(deps.storage, &state)?;
    Ok(Response::new())
}

pub fn try_update_gas_price(deps: DepsMut, info: MessageInfo, env: Env, token: String, value: String) -> Result<Response, ContractError> {
    let state = CONFIG.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    };
    GAS_MAP.save(deps.storage, token, &GasPrice {
        gas_price: Decimal::from_str(&value)?,
        last_updated: env.block.time.seconds(),
    })?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::GasPrice { token } => to_binary(&query_gas_price(deps, env, token)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner.to_string(),
    })
}

fn query_gas_price(deps: Deps, env: Env, token: String) -> StdResult<GasPrice> {
    let gas = GAS_MAP.load(deps.storage, token)?;
    Ok(GasPrice {
        gas_price: gas.gas_price,
        last_updated: env.block.time.seconds()
    })
}
