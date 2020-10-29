use crate::state::{increment_offerings, Offering, CONTRACT_INFO, OFFERINGS};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse,
    MessageInfo, Querier, StdResult, Storage,
};
use cw20::Cw20ReceiveMsg;
use cw721::{ContractInfoResponse, Cw721ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{
    BuyNft, HandleMsg, InitMsg, OfferingsResponse, QueryMsg, ReceiveMsgWrapper, SellNft,
};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let info = ContractInfoResponse {
        name: msg.marketplace_name,
        symbol: msg.symbol,
    };
    CONTRACT_INFO.save(&mut deps.storage, &info)?;
    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
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
    wrapper: ReceiveMsgWrapper,
) -> Result<HandleResponse, ContractError> {
    match wrapper {
        ReceiveMsgWrapper::Cw20Rcv(msg) => try_buy_nft(deps, info, msg),
        ReceiveMsgWrapper::Cw721Rcv(msg) => try_sell_nft(deps, info, msg),
    }
}

pub fn try_sell_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let msg: SellNft = match rcv_msg.msg {
        Some(bin) => Ok(from_binary(&bin)?),
        None => Err(ContractError::NoData {}),
    }?;

    // check if same token Id form same original contract is already on sale
    // get OFFERING_COUNT
    let id = increment_offerings(&mut deps.storage)?.to_string();

    // save Offering
    let off = Offering {
        contract_addr: deps.api.canonical_address(&info.sender)?,
        token_id: rcv_msg.token_id,
        seller: deps.api.canonical_address(&rcv_msg.sender)?,
        list_price: msg.list_price.clone(),
    };

    OFFERINGS.save(&mut deps.storage, &id, &off)?;

    let price_string = format!("{} {}", msg.list_price.amount, msg.list_price.address);

    Ok(HandleResponse {
        messages: Vec::new(),
        attributes: vec![
            attr("action", "sell_nft"),
            attr("original_contract", info.sender),
            attr("seller", off.seller),
            attr("list_price", price_string),
            attr("token_id", off.token_id),
        ],
        data: None,
    })
}

pub fn try_buy_nft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _info: MessageInfo,
    rcv_msg: Cw20ReceiveMsg,
) -> Result<HandleResponse, ContractError> {
    let _msg: BuyNft = match rcv_msg.msg {
        Some(bin) => Ok(from_binary(&bin)?),
        None => Err(ContractError::NoData {}),
    }?;

    // check if offering exists
    // check for enough coins
    // if everything is fine transfer cw20 to seller
    // transfer nft to buyer
    //delete offering

    Ok(HandleResponse::default())
}

pub fn try_withdraw<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    token_id: String,
) -> Result<HandleResponse, ContractError> {
    // check if token_id is currently sold by the requesting address
    let off = OFFERINGS.load(&deps.storage, &token_id)?;
    if off.seller == deps.api.canonical_address(&info.sender)? {
        return Ok(HandleResponse {
            messages: Vec::new(),
            attributes: vec![
                attr("action", "withdraw_nft"),
                attr("seller", info.sender),
                attr("token_id", token_id),
            ],
            data: None,
        });
    }
    Err(ContractError::Unauthorized {})
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
    _deps: &Extern<S, A, Q>,
) -> StdResult<OfferingsResponse> {
    // let offs: StdResult<Vec<Offering>> = offerings::<S>()
    //     .range(&deps.storage, None, None, Order::Ascending)
    //     .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
    //     .collect();

    // let offs: StdResult<Vec<Offering>> = offerings::<S>()
    //     .range(&deps.storage, None, None, Order::Ascending)
    //     .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
    //     .collect();

    Ok(OfferingsResponse {
        offerings: Vec::new(), // Placeholder
    })
}

// ============================== Test ==============================

#[cfg(test)]
mod tests {
    use cosmwasm_std::Uint128;
use cosmwasm_std::HumanAddr;
use crate::msg::ReceiveMsgWrapper::Cw721Rcv;
use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

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

        #[test]
        fn post_offering() {
            let mut deps = mock_dependencies(&coins(2, "token"));

            let msg = InitMsg { 
                marketplace_name: String::from("test market"),
             };
            let info = mock_info("creator", &coins(2, "token"));
            let _res = init(&mut deps, mock_env(), info, msg).unwrap();

            // beneficiary can release it
            let info = mock_info("anyone", &coins(2, "token"));

             let sellMsg = SellNft {
                 list_price: Cw20CoinHuman {
                    address: HumanAddr::from("cw20ContractAddr"),
                    amount: Uint128::from(5),
                 }
             };

            let msg = HandleMsg::Receive(Cw721Rcv(Cw721ReceiveMsg{
                sender: HumanAddr::from("seller"),
                token_id: String::from("SellableNFT"),
                msg: to_binary(&sellMsg).ok(),
            }));
            let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

            // Offering should be listed
            let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
            let value: CountResponse = from_binary(&res).unwrap();
            assert_eq!(18, value.count);
        }

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
 }
