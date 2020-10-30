use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_binary, to_vec, Api, Binary, CanonicalAddr, ContractResult, CosmosMsg, Empty,
    HumanAddr, Querier, QueryRequest, StdError, StdResult, SystemResult, WasmMsg, WasmQuery,
};

use crate::msg::Cw4HandleMsg;
use crate::{member_key, AdminResponse, Cw4QueryMsg, Member, MemberListResponse, TOTAL_KEY};

/// Cw4Contract is a wrapper around HumanAddr that provides a lot of helpers
/// for working with cw4 contracts
///
/// If you wish to persist this, convert to Cw4CanonicalContract via .canonical()
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw4Contract(pub HumanAddr);

impl Cw4Contract {
    pub fn addr(&self) -> HumanAddr {
        self.0.clone()
    }

    /// Convert this address to a form fit for storage
    pub fn canonical<A: Api>(&self, api: &A) -> StdResult<Cw4CanonicalContract> {
        let canon = api.canonical_address(&self.0)?;
        Ok(Cw4CanonicalContract(canon))
    }

    fn encode_msg(&self, msg: Cw4HandleMsg) -> StdResult<CosmosMsg> {
        Ok(WasmMsg::Execute {
            contract_addr: self.addr(),
            msg: to_binary(&msg)?,
            send: vec![],
        }
        .into())
    }

    pub fn update_admin<T: Into<HumanAddr>>(
        &self,
        admin: Option<HumanAddr>,
    ) -> StdResult<CosmosMsg> {
        let msg = Cw4HandleMsg::UpdateAdmin { admin };
        self.encode_msg(msg)
    }

    pub fn update_members(&self, remove: Vec<HumanAddr>, add: Vec<Member>) -> StdResult<CosmosMsg> {
        let msg = Cw4HandleMsg::UpdateMembers { remove, add };
        self.encode_msg(msg)
    }

    fn encode_smart_query(&self, msg: Cw4QueryMsg) -> StdResult<QueryRequest<Empty>> {
        Ok(WasmQuery::Smart {
            contract_addr: self.addr(),
            msg: to_binary(&msg)?,
        }
        .into())
    }

    fn encode_raw_query<T: Into<Binary>>(&self, key: T) -> StdResult<QueryRequest<Empty>> {
        Ok(WasmQuery::Raw {
            contract_addr: self.addr(),
            key: key.into(),
        }
        .into())
    }

    /// Read the admin
    pub fn admin<Q: Querier>(&self, querier: &Q) -> StdResult<Option<HumanAddr>> {
        let query = self.encode_smart_query(Cw4QueryMsg::Admin {})?;
        let res: AdminResponse = querier.query(&query)?;
        Ok(res.admin)
    }

    /// Read the total weight
    pub fn total_weight<Q: Querier>(&self, querier: &Q) -> StdResult<u64> {
        let query = self.encode_raw_query(TOTAL_KEY)?;
        querier.query(&query)
    }

    // TODO: implement with raw queries
    /// Check if this address is a member, and if so, with which weight
    pub fn is_member<Q: Querier, T: Into<CanonicalAddr>>(
        &self,
        querier: &Q,
        addr: T,
    ) -> StdResult<Option<u64>> {
        let path = member_key(&addr.into());
        let query = self.encode_raw_query(path)?;

        // We have to copy the logic of Querier.query to handle the empty case, and not
        // try to decode empty result into a u64.
        // TODO: add similar API on Querier - this is not the first time I came across it
        let raw = to_vec(&query)?;
        match querier.raw_query(&raw) {
            SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            ))),
            SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(
                format!("Querier contract error: {}", contract_err),
            )),
            SystemResult::Ok(ContractResult::Ok(value)) => {
                // This is the only place we customize
                if value.is_empty() {
                    Ok(None)
                } else {
                    from_slice(&value)
                }
            }
        }
    }

    pub fn list_members<Q: Querier>(
        &self,
        querier: &Q,
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Member>> {
        let query = self.encode_smart_query(Cw4QueryMsg::ListMembers { start_after, limit })?;
        let res: MemberListResponse = querier.query(&query)?;
        Ok(res.members)
    }
}

/// This is a respresentation of Cw4Contract for storage.
/// Don't use it directly, just translate to the Cw4Contract when needed.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw4CanonicalContract(pub CanonicalAddr);

impl Cw4CanonicalContract {
    /// Convert this address to a form fit for usage in messages and queries
    pub fn human<A: Api>(&self, api: &A) -> StdResult<Cw4Contract> {
        let human = api.human_address(&self.0)?;
        Ok(Cw4Contract(human))
    }
}
