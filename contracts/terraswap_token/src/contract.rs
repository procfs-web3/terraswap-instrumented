#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult};

use cw2::set_contract_version;
use cw20_base::contract::{create_accounts, execute as cw20_execute, query as cw20_query};
use cw20_base::msg::{ExecuteMsg, QueryMsg};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};
use cw20_base::ContractError;

use terraswap::token::InstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;

    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(ContractError::Std(StdError::generic_err(
                "Initial supply greater than cap",
            )));
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };

    TOKEN_INFO.save(deps.storage, &data)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    cw20_execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_query(deps, env, msg)
}

mod memory {
    use std::convert::TryFrom;
    use std::mem;
    use std::vec::Vec;

    #[repr(C)]
    pub struct Region {
        /// The beginning of the region expressed as bytes from the beginning of the linear memory
        pub offset: u32,
        /// The number of bytes available in this region
        pub capacity: u32,
        /// The number of bytes used in this region
        pub length: u32,
    }

    pub fn release_buffer(buffer: Vec<u8>) -> *mut Region {
        let region = build_region(&buffer);
        mem::forget(buffer);
        Box::into_raw(region)
    }

    pub fn build_region(data: &[u8]) -> Box<Region> {
        let data_ptr = data.as_ptr() as usize;
        build_region_from_components(
            u32::try_from(data_ptr).expect("pointer doesn't fit in u32"),
            u32::try_from(data.len()).expect("length doesn't fit in u32"),
            u32::try_from(data.len()).expect("length doesn't fit in u32"),
        )
    }

    fn build_region_from_components(offset: u32, capacity: u32, length: u32) -> Box<Region> {
        Box::new(Region {
            offset,
            capacity,
            length,
        })
    }
}

mod coverage {
    use super::memory::release_buffer;
    use minicov::capture_coverage;
    #[no_mangle]
    extern "C" fn dump_coverage() -> u32 {
        let coverage = capture_coverage();
        release_buffer(coverage) as u32
    }
}
