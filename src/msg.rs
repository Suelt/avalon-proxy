

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use cosmwasm_std::{ContractResult, Timestamp};

use crate::state::Transaction;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Hash)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg{
    pub chain_id: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Hash)]
pub enum ExecuteMsg{
    Input {value: Transaction},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Hash)]
pub enum QueryMsg{
    GetTx{},
    GetChainId{},
    GetNodeNumber{},
    GetDirtyTx{},
    GetPrepareTx{},

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetTxResponse{
    pub dirty_votes: Option<u32>,
    pub prepare_votes: Option<u32>,
    pub committed: Option<bool>,
    pub aborted: Option<bool>,
    pub time: Option<u64>,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Op{
    // sent after processing an input
    DirtySuccess{value: u32},
    //sent after acquiring all dirty votes but fail or find out that a previous prepared state fails
    Abortion{value: u32},
    //sent after acquiring all dirty votes and succeed
    PrepareSuccess{value: u32},    

    WhoAmI{chain_id: u32},
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WhoAmIResponse {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MsgQueueResponse{

}
pub type AcknowledgementMsg<T> = ContractResult<T>;