use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Binary, CosmosMsg, HumanAddr, StdResult, Uint128, WasmMsg};

/// Cw20ReceiveMsg should be de/serialized under `Receive()` variant in a HandleMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Cw20ReceiveMsg {
    pub sender: HumanAddr,
    pub amount: Uint128,
    pub msg: Option<Binary>,
}

impl Cw20ReceiveMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverHandleMsg::Receive(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, contract_addr: HumanAddr) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr,
            msg,
            send: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum ReceiverHandleMsg {
    Receive(Cw20ReceiveMsg),
}
