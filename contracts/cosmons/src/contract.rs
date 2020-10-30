use cosmwasm_std::{
    attr, to_binary, Api, Binary, BlockInfo, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    MessageInfo, Order, Querier, StdError, StdResult, Storage, KV,
};

use cw0::maybe_canonical;
use cw2::set_contract_version;
use cw721::{
    AllNftInfoResponse, ApprovedForAllResponse, ContractInfoResponse, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse,
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, MintMsg, MinterResponse, QueryMsg};
use crate::state::{
    increment_tokens, num_tokens, tokens, Approval, TokenInfo, CONTRACT_INFO, MINTER, OPERATORS,
};
use cw_storage_plus::Bound;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_contract_version(&mut deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = ContractInfoResponse {
        name: msg.name,
        symbol: msg.symbol,
    };
    CONTRACT_INFO.save(&mut deps.storage, &info)?;
    let minter = deps.api.canonical_address(&msg.minter)?;
    MINTER.save(&mut deps.storage, &minter)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::Mint(msg) => handle_mint(deps, env, info, msg),
        HandleMsg::BattleMonster {
            attacker_id,
            defender_id,
        } => handle_battle_monster(deps, env, info, attacker_id, defender_id),
        HandleMsg::Approve {
            spender,
            token_id,
            expires,
        } => handle_approve(deps, env, info, spender, token_id, expires),
        HandleMsg::Revoke { spender, token_id } => {
            handle_revoke(deps, env, info, spender, token_id)
        }
        HandleMsg::ApproveAll { operator, expires } => {
            handle_approve_all(deps, env, info, operator, expires)
        }
        HandleMsg::RevokeAll { operator } => handle_revoke_all(deps, env, info, operator),
        HandleMsg::TransferNft {
            recipient,
            token_id,
        } => handle_transfer_nft(deps, env, info, recipient, token_id),
        HandleMsg::SendNft {
            contract,
            token_id,
            msg,
        } => handle_send_nft(deps, env, info, contract, token_id, msg),
    }
}

pub fn handle_battle_monster<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    attacker_id: String,
    defender_id: String,
) -> Result<HandleResponse, ContractError> {
    let mut info_attacker_id = tokens().load(&deps.storage, &attacker_id)?;
    let mut info_defender_id = tokens().load(&deps.storage, &defender_id)?;
    if info_attacker_id.level >= info_defender_id.level {
        info_attacker_id.level += 2;
        info_defender_id.level += 1;
    } else {
        info_attacker_id.level += 1;
        info_defender_id.level += 2;
    }
    tokens().save(&mut deps.storage, &attacker_id, &info_attacker_id)?;
    tokens().save(&mut deps.storage, &defender_id, &info_defender_id)?;
    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "battle_monster"),
            attr("attacker_id", attacker_id),
            attr("defender_id", defender_id),
        ],
        data: None,
    })
}

pub fn handle_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<HandleResponse, ContractError> {
    let minter = MINTER.load(&deps.storage)?;
    let sender_raw = deps.api.canonical_address(&info.sender)?;

    if sender_raw != minter {
        return Err(ContractError::Unauthorized {});
    }

    // create the token
    let token = TokenInfo {
        owner: deps.api.canonical_address(&msg.owner)?,
        approvals: vec![],
        name: msg.name,
        level: msg.level,
        description: msg.description.unwrap_or_default(),
        image: msg.image,
    };
    tokens().update(&mut deps.storage, &msg.token_id, |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
    })?;

    increment_tokens(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "mint"),
            attr("minter", info.sender),
            attr("token_id", msg.token_id),
        ],
        data: None,
    })
}

pub fn handle_transfer_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    recipient: HumanAddr,
    token_id: String,
) -> Result<HandleResponse, ContractError> {
    _transfer_nft(deps, &env, &info, &recipient, &token_id)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "transfer_nft"),
            attr("sender", info.sender),
            attr("recipient", recipient),
            attr("token_id", token_id),
        ],
        data: None,
    })
}

pub fn handle_send_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    contract: HumanAddr,
    token_id: String,
    msg: Option<Binary>,
) -> Result<HandleResponse, ContractError> {
    // Transfer token
    _transfer_nft(deps, &env, &info, &contract, &token_id)?;

    let send = Cw721ReceiveMsg {
        sender: info.sender.clone(),
        token_id: token_id.clone(),
        msg,
    };

    // Send message
    Ok(HandleResponse {
        messages: vec![send.into_cosmos_msg(contract.clone())?],
        attributes: vec![
            attr("action", "send_nft"),
            attr("sender", info.sender),
            attr("recipient", contract),
            attr("token_id", token_id),
        ],
        data: None,
    })
}

pub fn _transfer_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    recipient: &HumanAddr,
    token_id: &str,
) -> Result<TokenInfo, ContractError> {
    let mut token = tokens().load(&deps.storage, &token_id)?;
    // ensure we have permissions
    check_can_send(&deps, env, info, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.canonical_address(recipient)?;
    token.approvals = vec![];
    tokens().save(&mut deps.storage, &token_id, &token)?;
    Ok(token)
}

pub fn handle_approve<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    spender: HumanAddr,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<HandleResponse, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "approve"),
            attr("sender", info.sender),
            attr("spender", spender),
            attr("token_id", token_id),
        ],
        data: None,
    })
}

pub fn handle_revoke<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    spender: HumanAddr,
    token_id: String,
) -> Result<HandleResponse, ContractError> {
    _update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "revoke"),
            attr("sender", info.sender),
            attr("spender", spender),
            attr("token_id", token_id),
        ],
        data: None,
    })
}

pub fn _update_approvals<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    spender: &HumanAddr,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> Result<TokenInfo, ContractError> {
    let mut token = tokens().load(&deps.storage, &token_id)?;
    // ensure we have permissions
    check_can_approve(&deps, env, info, &token)?;

    // update the approval list (remove any for the same spender before adding)
    let spender_raw = deps.api.canonical_address(&spender)?;
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_raw)
        .collect();

    // only difference between approve and revoke
    if add {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(ContractError::Expired {});
        }
        let approval = Approval {
            spender: spender_raw,
            expires,
        };
        token.approvals.push(approval);
    }

    tokens().save(&mut deps.storage, &token_id, &token)?;

    Ok(token)
}

pub fn handle_approve_all<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    operator: HumanAddr,
    expires: Option<Expiration>,
) -> Result<HandleResponse, ContractError> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired {});
    }

    // set the operator for us
    let sender_raw = deps.api.canonical_address(&info.sender)?;
    let operator_raw = deps.api.canonical_address(&operator)?;
    OPERATORS.save(&mut deps.storage, (&sender_raw, &operator_raw), &expires)?;

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "approve_all"),
            attr("sender", info.sender),
            attr("operator", operator),
        ],
        data: None,
    })
}

pub fn handle_revoke_all<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    operator: HumanAddr,
) -> Result<HandleResponse, ContractError> {
    let sender_raw = deps.api.canonical_address(&info.sender)?;
    let operator_raw = deps.api.canonical_address(&operator)?;
    OPERATORS.remove(&mut deps.storage, (&sender_raw, &operator_raw));

    Ok(HandleResponse {
        messages: vec![],
        attributes: vec![
            attr("action", "revoke_all"),
            attr("sender", info.sender),
            attr("operator", operator),
        ],
        data: None,
    })
}

/// returns true iff the sender can execute approve or reject on the contract
fn check_can_approve<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo,
) -> Result<(), ContractError> {
    // owner can approve
    let sender_raw = deps.api.canonical_address(&info.sender)?;
    if token.owner == sender_raw {
        return Ok(());
    }
    // operator can approve
    let op = OPERATORS.may_load(&deps.storage, (&token.owner, &sender_raw))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}

/// returns true iff the sender can transfer ownership of the token
fn check_can_send<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo,
) -> Result<(), ContractError> {
    // owner can send
    let sender_raw = deps.api.canonical_address(&info.sender)?;
    if token.owner == sender_raw {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == sender_raw && !apr.expires.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
    let op = OPERATORS.may_load(&deps.storage, (&token.owner, &sender_raw))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::NftInfo { token_id } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_binary(&query_owner_of(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(
            deps,
            env,
            token_id,
            include_expired.unwrap_or(false),
        )?),
        QueryMsg::ApprovedForAll {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&query_all_approvals(
            deps,
            env,
            owner,
            include_expired.unwrap_or(false),
            start_after,
            limit,
        )?),
        QueryMsg::NumTokens {} => to_binary(&query_num_tokens(deps)?),
        QueryMsg::Tokens {
            owner,
            start_after,
            limit,
        } => to_binary(&query_tokens(deps, owner, start_after, limit)?),
        QueryMsg::AllTokens { start_after, limit } => {
            to_binary(&query_all_tokens(deps, start_after, limit)?)
        }
    }
}

fn query_minter<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MinterResponse> {
    let minter_raw = MINTER.load(&deps.storage)?;
    let minter = deps.api.human_address(&minter_raw)?;
    Ok(MinterResponse { minter })
}

fn query_contract_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ContractInfoResponse> {
    CONTRACT_INFO.load(&deps.storage)
}

fn query_num_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<NumTokensResponse> {
    let count = num_tokens(&deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_nft_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: String,
) -> StdResult<NftInfoResponse> {
    let info = tokens().load(&deps.storage, &token_id)?;
    Ok(NftInfoResponse {
        name: info.name,
        level: info.level,
        description: info.description,
        image: info.image,
    })
}

fn query_owner_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<OwnerOfResponse> {
    let info = tokens().load(&deps.storage, &token_id)?;
    Ok(OwnerOfResponse {
        owner: deps.api.human_address(&info.owner)?,
        approvals: humanize_approvals(deps.api, &env.block, &info, include_expired)?,
    })
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

fn query_all_approvals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    include_expired: bool,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
) -> StdResult<ApprovedForAllResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_canon = maybe_canonical(deps.api, start_after)?;
    let start = start_canon.map(Bound::exclusive);

    let owner_raw = deps.api.canonical_address(&owner)?;
    let res: StdResult<Vec<_>> = OPERATORS
        .prefix(&owner_raw)
        .range(&deps.storage, start, None, Order::Ascending)
        .filter(|r| include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block))
        .take(limit)
        .map(|item| parse_approval(deps.api, item))
        .collect();
    Ok(ApprovedForAllResponse { operators: res? })
}

fn parse_approval<A: Api>(api: A, item: StdResult<KV<Expiration>>) -> StdResult<cw721::Approval> {
    item.and_then(|(k, expires)| {
        let spender = api.human_address(&k.into())?;
        Ok(cw721::Approval { spender, expires })
    })
}

fn query_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let owner_raw = deps.api.canonical_address(&owner)?;
    let tokens: Result<Vec<String>, _> = tokens::<S>()
        .idx
        .owner
        .pks(&deps.storage, &owner_raw, start, None, Order::Ascending)
        .take(limit)
        .map(String::from_utf8)
        .collect();
    let tokens = tokens.map_err(StdError::invalid_utf8)?;
    Ok(TokensResponse { tokens })
}

fn query_all_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let tokens: StdResult<Vec<String>> = tokens::<S>()
        .range(&deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}

fn query_all_nft_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
    token_id: String,
    include_expired: bool,
) -> StdResult<AllNftInfoResponse> {
    let info = tokens().load(&deps.storage, &token_id)?;
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: deps.api.human_address(&info.owner)?,
            approvals: humanize_approvals(deps.api, &env.block, &info, include_expired)?,
        },
        info: NftInfoResponse {
            name: info.name,
            level: info.level,
            description: info.description,
            image: info.image,
        },
    })
}

fn humanize_approvals<A: Api>(
    api: A,
    block: &BlockInfo,
    info: &TokenInfo,
    include_expired: bool,
) -> StdResult<Vec<cw721::Approval>> {
    let iter = info.approvals.iter();
    iter.filter(|apr| include_expired || !apr.expires.is_expired(block))
        .map(|apr| humanize_approval(api, apr))
        .collect()
}

fn humanize_approval<A: Api>(api: A, approval: &Approval) -> StdResult<cw721::Approval> {
    Ok(cw721::Approval {
        spender: api.human_address(&approval.spender)?,
        expires: approval.expires,
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, CosmosMsg, WasmMsg};

    use super::*;
    use cw721::ApprovedForAllResponse;

    const MINTER: &str = "merlin";
    const CONTRACT_NAME: &str = "Magic Power";
    const SYMBOL: &str = "MGK";

    fn setup_contract<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>) {
        let msg = InitMsg {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            minter: MINTER.into(),
        };
        let info = mock_info("creator", &[]);
        let res = init(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            minter: MINTER.into(),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query_minter(&deps).unwrap();
        assert_eq!(MINTER, res.minter.as_str());
        let info = query_contract_info(&deps).unwrap();
        assert_eq!(
            info,
            ContractInfoResponse {
                name: CONTRACT_NAME.to_string(),
                symbol: SYMBOL.to_string(),
            }
        );

        let count = query_num_tokens(&deps).unwrap();
        assert_eq!(0, count.count);

        // list the token_ids
        let tokens = query_all_tokens(&deps, None, None).unwrap();
        assert_eq!(0, tokens.tokens.len());
    }

    #[test]
    fn minting() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);

        let token_id = "petrify".to_string();
        let name = "Petrify with Gaze".to_string();
        let description = "Allows the owner to petrify anyone looking at him or her".to_string();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id.clone(),
            owner: "medusa".into(),
            name: name.clone(),
            level: 1,
            description: Some(description.clone()),
            image: None,
        });

        // random cannot mint
        let random = mock_info("random", &[]);
        let err = handle(&mut deps, mock_env(), random, mint_msg.clone()).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }

        // minter can mint
        let allowed = mock_info(MINTER, &[]);
        let _ = handle(&mut deps, mock_env(), allowed, mint_msg.clone()).unwrap();

        // ensure num tokens increases
        let count = query_num_tokens(&deps).unwrap();
        assert_eq!(1, count.count);

        // unknown nft returns error
        let _ = query_nft_info(&deps, "unknown".to_string()).unwrap_err();

        // this nft info is correct
        let info = query_nft_info(&deps, token_id.clone()).unwrap();
        assert_eq!(
            info,
            NftInfoResponse {
                name: name.clone(),
                level: info.level,
                description: description.clone(),
                image: None,
            }
        );

        // owner info is correct
        let owner = query_owner_of(&deps, mock_env(), token_id.clone(), true).unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: "medusa".into(),
                approvals: vec![],
            }
        );

        // Cannot mint same token_id again
        let mint_msg2 = HandleMsg::Mint(MintMsg {
            token_id: token_id.clone(),
            owner: "hercules".into(),
            name: "copy cat".into(),
            level: 1,
            description: None,
            image: None,
        });

        let allowed = mock_info(MINTER, &[]);
        let err = handle(&mut deps, mock_env(), allowed, mint_msg2).unwrap_err();
        match err {
            ContractError::Claimed {} => {}
            e => panic!("unexpected error: {}", e),
        }

        // list the token_ids
        let tokens = query_all_tokens(&deps, None, None).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id], tokens.tokens);
    }

    #[test]
    fn transferring_nft() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "melt".to_string();
        let name = "Melting power".to_string();
        let description = "Allows the owner to melt anyone looking at him or her".to_string();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id.clone(),
            owner: "venus".into(),
            name: name.clone(),
            level: 1,
            description: Some(description.clone()),
            image: None,
        });

        let minter = mock_info(MINTER, &[]);
        handle(&mut deps, mock_env(), minter, mint_msg).unwrap();

        // random cannot transfer
        let random = mock_info("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "random".into(),
            token_id: token_id.clone(),
        };

        let err = handle(&mut deps, mock_env(), random, transfer_msg.clone()).unwrap_err();

        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }

        // owner can
        let random = mock_info("venus", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "random".into(),
            token_id: token_id.clone(),
        };

        let res = handle(&mut deps, mock_env(), random, transfer_msg.clone()).unwrap();

        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                attributes: vec![
                    attr("action", "transfer_nft"),
                    attr("sender", "venus"),
                    attr("recipient", "random"),
                    attr("token_id", token_id),
                ],
                data: None,
            }
        );
    }

    #[test]
    fn sending_nft() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "melt".to_string();
        let name = "Melting power".to_string();
        let description = "Allows the owner to melt anyone looking at him or her".to_string();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id.clone(),
            owner: "venus".into(),
            name: name.clone(),
            level: 1,
            description: Some(description.clone()),
            image: None,
        });

        let minter = mock_info(MINTER, &[]);
        handle(&mut deps, mock_env(), minter, mint_msg).unwrap();

        let msg = to_binary("You now have the melting power").unwrap();
        let target = HumanAddr::from("another_contract");
        let send_msg = HandleMsg::SendNft {
            contract: target.clone(),
            token_id: token_id.clone(),
            msg: Some(msg.clone()),
        };

        let random = mock_info("random", &[]);
        let err = handle(&mut deps, mock_env(), random, send_msg.clone()).unwrap_err();
        match err {
            ContractError::Unauthorized {} => {}
            e => panic!("unexpected error: {}", e),
        }

        // but owner can
        let random = mock_info("venus", &[]);
        let res = handle(&mut deps, mock_env(), random, send_msg).unwrap();

        let payload = Cw721ReceiveMsg {
            sender: "venus".into(),
            token_id: token_id.clone(),
            msg: Some(msg),
        };
        let expected = payload.into_cosmos_msg(target.clone()).unwrap();
        // ensure expected serializes as we think it should
        match &expected {
            CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
                assert_eq!(contract_addr, &target)
            }
            m => panic!("Unexpected message type: {:?}", m),
        }
        // and make sure this is the request sent by the contract
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![expected],
                attributes: vec![
                    attr("action", "send_nft"),
                    attr("sender", "venus"),
                    attr("recipient", "another_contract"),
                    attr("token_id", token_id),
                ],
                data: None,
            }
        );
    }

    #[test]
    fn approving_revoking() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "grow".to_string();
        let name = "Growing power".to_string();
        let description = "Allows the owner to grow anything".to_string();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id.clone(),
            owner: "demeter".into(),
            name: name.clone(),
            level: 1,
            description: Some(description.clone()),
            image: None,
        });

        let minter = mock_info(MINTER, &[]);
        handle(&mut deps, mock_env(), minter, mint_msg).unwrap();

        // Give random transferring power
        let approve_msg = HandleMsg::Approve {
            spender: "random".into(),
            token_id: token_id.clone(),
            expires: None,
        };
        let owner = mock_info("demeter", &[]);
        let res = handle(&mut deps, mock_env(), owner, approve_msg).unwrap();
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                attributes: vec![
                    attr("action", "approve"),
                    attr("sender", "demeter"),
                    attr("spender", "random"),
                    attr("token_id", token_id.clone()),
                ],
                data: None,
            }
        );

        // random can now transfer
        let random = mock_info("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "person".into(),
            token_id: token_id.clone(),
        };
        handle(&mut deps, mock_env(), random, transfer_msg).unwrap();

        // Approvals are removed / cleared
        let query_msg = QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: None,
        };
        let res: OwnerOfResponse =
            from_binary(&query(&deps, mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            res,
            OwnerOfResponse {
                owner: "person".into(),
                approvals: vec![],
            }
        );

        // Approve, revoke, and check for empty, to test revoke
        let approve_msg = HandleMsg::Approve {
            spender: "random".into(),
            token_id: token_id.clone(),
            expires: None,
        };
        let owner = mock_info("person", &[]);
        handle(&mut deps, mock_env(), owner.clone(), approve_msg).unwrap();

        let revoke_msg = HandleMsg::Revoke {
            spender: "random".into(),
            token_id: token_id.clone(),
        };
        handle(&mut deps, mock_env(), owner, revoke_msg).unwrap();

        // Approvals are now removed / cleared
        let res: OwnerOfResponse =
            from_binary(&query(&deps, mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            res,
            OwnerOfResponse {
                owner: "person".into(),
                approvals: vec![],
            }
        );
    }

    #[test]
    fn approving_all_revoking_all() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);

        // Mint a couple tokens (from the same owner)
        let token_id1 = "grow1".to_string();
        let name1 = "Growing power".to_string();
        let description1 = "Allows the owner the power to grow anything".to_string();
        let token_id2 = "grow2".to_string();
        let name2 = "More growing power".to_string();
        let description2 = "Allows the owner the power to grow anything even faster".to_string();

        let mint_msg1 = HandleMsg::Mint(MintMsg {
            token_id: token_id1.clone(),
            owner: "demeter".into(),
            name: name1.clone(),
            level: 1,
            description: Some(description1.clone()),
            image: None,
        });

        let minter = mock_info(MINTER, &[]);
        handle(&mut deps, mock_env(), minter.clone(), mint_msg1).unwrap();

        let mint_msg2 = HandleMsg::Mint(MintMsg {
            token_id: token_id2.clone(),
            owner: "demeter".into(),
            name: name2.clone(),
            level: 1,
            description: Some(description2.clone()),
            image: None,
        });

        handle(&mut deps, mock_env(), minter, mint_msg2).unwrap();

        // paginate the token_ids
        let tokens = query_all_tokens(&deps, None, Some(1)).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id1.clone()], tokens.tokens);
        let tokens = query_all_tokens(&deps, Some(token_id1.clone()), Some(3)).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id2.clone()], tokens.tokens);

        // demeter gives random full (operator) power over her tokens
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "random".into(),
            expires: None,
        };
        let owner = mock_info("demeter", &[]);
        let res = handle(&mut deps, mock_env(), owner, approve_all_msg).unwrap();
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                attributes: vec![
                    attr("action", "approve_all"),
                    attr("sender", "demeter"),
                    attr("operator", "random"),
                ],
                data: None,
            }
        );

        // random can now transfer
        let random = mock_info("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "person".into(),
            token_id: token_id1.clone(),
        };
        handle(&mut deps, mock_env(), random.clone(), transfer_msg).unwrap();

        // random can now send
        let inner_msg = WasmMsg::Execute {
            contract_addr: "another_contract".into(),
            msg: to_binary("You now also have the growing power").unwrap(),
            send: vec![],
        };
        let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

        let send_msg = HandleMsg::SendNft {
            contract: "another_contract".into(),
            token_id: token_id2.clone(),
            msg: Some(to_binary(&msg).unwrap()),
        };
        handle(&mut deps, mock_env(), random, send_msg).unwrap();

        // Approve_all, revoke_all, and check for empty, to test revoke_all
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "operator".into(),
            expires: None,
        };
        // person is now the owner of the tokens
        let owner = mock_info("person", &[]);
        handle(&mut deps, mock_env(), owner.clone(), approve_all_msg).unwrap();

        let res =
            query_all_approvals(&deps, mock_env(), "person".into(), true, None, None).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "operator".into(),
                    expires: Expiration::Never {}
                }]
            }
        );

        // second approval
        let buddy_expires = Expiration::AtHeight(1234567);
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "buddy".into(),
            expires: Some(buddy_expires),
        };
        let owner = mock_info("person", &[]);
        handle(&mut deps, mock_env(), owner.clone(), approve_all_msg).unwrap();

        // and paginate queries
        let res =
            query_all_approvals(&deps, mock_env(), "person".into(), true, None, Some(1)).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "buddy".into(),
                    expires: buddy_expires,
                }]
            }
        );
        let res = query_all_approvals(
            &deps,
            mock_env(),
            "person".into(),
            true,
            Some("buddy".into()),
            Some(2),
        )
        .unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "operator".into(),
                    expires: Expiration::Never {}
                }]
            }
        );

        let revoke_all_msg = HandleMsg::RevokeAll {
            operator: "operator".into(),
        };
        handle(&mut deps, mock_env(), owner, revoke_all_msg).unwrap();

        // Approvals are removed / cleared without affecting others
        let res =
            query_all_approvals(&deps, mock_env(), "person".into(), false, None, None).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "buddy".into(),
                    expires: buddy_expires,
                }]
            }
        );

        // ensure the filter works (nothing should be here
        let mut late_env = mock_env();
        late_env.block.height = 1234568; //expired
        let res = query_all_approvals(&deps, late_env, "person".into(), false, None, None).unwrap();
        assert_eq!(0, res.operators.len());
    }

    #[test]
    fn query_tokens_by_owner() {
        let mut deps = mock_dependencies(&[]);
        setup_contract(&mut deps);
        let minter = mock_info(MINTER, &[]);

        // Mint a couple tokens (from the same owner)
        let token_id1 = "grow1".to_string();
        let demeter = HumanAddr::from("Demeter");
        let token_id2 = "grow2".to_string();
        let ceres = HumanAddr::from("Ceres");
        let token_id3 = "sing".to_string();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id1.clone(),
            owner: demeter.clone(),
            name: "Growing power".to_string(),
            level: 1,
            description: Some("Allows the owner the power to grow anything".to_string()),
            image: None,
        });
        handle(&mut deps, mock_env(), minter.clone(), mint_msg).unwrap();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id2.clone(),
            owner: ceres.clone(),
            name: "More growing power".to_string(),
            level: 1,
            description: Some(
                "Allows the owner the power to grow anything even faster".to_string(),
            ),
            image: None,
        });
        handle(&mut deps, mock_env(), minter.clone(), mint_msg).unwrap();

        let mint_msg = HandleMsg::Mint(MintMsg {
            token_id: token_id3.clone(),
            owner: demeter.clone(),
            name: "Sing a lullaby".to_string(),
            level: 1,
            description: Some("Calm even the most excited children".to_string()),
            image: None,
        });
        handle(&mut deps, mock_env(), minter.clone(), mint_msg).unwrap();

        // get all tokens in order:
        let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];
        let tokens = query_all_tokens(&deps, None, None).unwrap();
        assert_eq!(&expected, &tokens.tokens);
        // paginate
        let tokens = query_all_tokens(&deps, None, Some(2)).unwrap();
        assert_eq!(&expected[..2], &tokens.tokens[..]);
        let tokens = query_all_tokens(&deps, Some(expected[1].clone()), None).unwrap();
        assert_eq!(&expected[2..], &tokens.tokens[..]);

        // get by owner
        let by_ceres = vec![token_id2.clone()];
        let by_demeter = vec![token_id1.clone(), token_id3.clone()];
        // all tokens by owner
        let tokens = query_tokens(&deps, demeter.clone(), None, None).unwrap();
        assert_eq!(&by_demeter, &tokens.tokens);
        let tokens = query_tokens(&deps, ceres.clone(), None, None).unwrap();
        assert_eq!(&by_ceres, &tokens.tokens);

        // paginate for demeter
        let tokens = query_tokens(&deps, demeter.clone(), None, Some(1)).unwrap();
        assert_eq!(&by_demeter[..1], &tokens.tokens[..]);
        let tokens =
            query_tokens(&deps, demeter.clone(), Some(by_demeter[0].clone()), Some(3)).unwrap();
        assert_eq!(&by_demeter[1..], &tokens.tokens[..]);
    }
}
