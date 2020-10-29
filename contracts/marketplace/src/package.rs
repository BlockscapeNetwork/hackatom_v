use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractInfoResponse {
    pub marketplace_name: String,
}

// THIS FILE SHOULD BE EXTRACTED TO ITS OWN PACKAGE PROJECT LIKE CW20 OR CW721
