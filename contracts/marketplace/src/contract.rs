use crate::state::{offerings, Offering, OFFERINGS, OFFERINGS_COUNT, increment_offerings, State};
use cosmwasm_std::{
    attr, to_binary, from_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, MessageInfo, Order, Querier,
    StdResult, Storage,
};
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, OfferingsResponse, QueryMsg, ReceiveMsg};
use cw20::Cw20CoinHuman;
use cw721::Cw721ReceiveMsg;

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        marketplace_name: msg.marketplace_name,
        owner: deps.api.canonical_address(&info.sender)?,
    };
    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::WithdrawNft { token_id } => try_withdraw(deps, info, token_id),
        HandleMsg::Receive(msg) => try_receive(deps, info, msg),
    }
}

// ============================== Message Handlers ==============================

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let msg: ReceiveMsg = match wrapper.msg {
        Some(bin) => Ok(from_binary(&bin)?),
        None => Err(ContractError::NoData {}),
    }?;
    match msg {
        ReceiveMsg::SellNft {
            list_price,
        } => try_sell_nft(deps, info, wrapper, list_price),
        ReceiveMsg::BuyNft { token_id } => try_buy_nft(deps, info, token_id),
    }
}

pub fn try_sell_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    rcvMsg: Cw721ReceiveMsg,
    list_price: Cw20CoinHuman,
) -> Result<HandleResponse, ContractError> {
    // check if same token Id form same original contract is already on sale
    
    // get OFFERING_COUNT
    let id = increment_offerings(&mut deps.storage)?.to_string();

    // save Offering
    let off = Offering {
        contract_addr: info.sender,
        token_id: rcvMsg.token_id,
        seller: rcvMsg.sender,
        list_price: list_price,
    };

    OFFERINGS.save(&mut deps.storage, &id, &off);

    let priceString = format!("{} {}", list_price.amount, list_price.address);

    Ok(HandleResponse{
        messages: Vec::new(),
        attributes: vec![
            attr("action", "sell_nft"),
            attr("original_contract", info.sender),
            attr("seller", off.seller),
            attr("list_price", priceString),
            attr("token_id", off.token_id),
        ],
        data: None,
    })
}

pub fn try_buy_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    token_id: String,
) -> Result<HandleResponse, ContractError> {
    Ok(HandleResponse::default())
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    token_id: String,
) -> Result<HandleResponse, ContractError> {
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOfferings {} => to_binary(&query_offerings(deps)?),
    }
}

// ============================== Query Handlers ==============================

fn query_offerings<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) /*-> StdResult<OfferingsResponse>*/ {
    // let offs: StdResult<Vec<Offering>> = offerings::<S>()
    //     .range(&deps.storage, None, None, Order::Ascending)
    //     .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
    //     .collect();

    // let offs: StdResult<Vec<Offering>> = offerings::<S>()
    //     .range(&deps.storage, None, None, Order::Ascending)
    //     .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
    //     .collect();

    //Ok(OfferingsResponse {})
}

// ============================== Test ==============================

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(&[]);

//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = init(&mut deps, mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(&coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = init(&mut deps, mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
