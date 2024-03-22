

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Timestamp;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State{
    pub node_number: u32,
    pub chain_id: u32,
    pub channel_ids: Vec<String>,


    pub dirty_tx_queue: Vec<u32>,
    pub prepare_tx_queue: Vec<u32>,
    pub commit_tx_queue: Vec<u32>,


    pub dirty_votes: u32,
    pub prepare_votes: u32,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub committed: bool,
    pub aborted: bool,
    pub time: u64,

}

impl State {
    //new
    pub fn new(chain_id: u32, start: Timestamp) -> Self {
        State {
            node_number: 1,
            chain_id,
            channel_ids: vec![],
            dirty_tx_queue: vec![],
            prepare_tx_queue: vec![],
            commit_tx_queue: vec![],
            dirty_votes: 0,
            prepare_votes: 0,
            start_time: start,
            end_time: None, 
            committed: false,
            aborted: false,
            time: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Hash)]
#[serde(rename_all = "snake_case")]
pub struct Transaction{
    pub tx_id: u32,
}
pub const STATE: Item<State> = Item::new("state");

// the start time and commit time represent the time when the transaction is added to submitted to the contract 
pub const START_TIME: Map<u32, Timestamp> = Map::new("start_time");
pub const END_TIME: Map<u32, Timestamp> = Map::new("end_time");

// the first element of the value tuple is the index of transactions, while the second is the number of votes received so far
// for simplicity, we implement no reentrancy checks for the votes
pub const DIRTY_VOTES_MAP : Map<u32, u32> = Map::new("dirty_votes_map");
pub const PREPARE_VOTES_MAP : Map<u32, u32> = Map::new("prepare_votes_map");

pub const COMMITTED_MAP:Map<u32, bool> = Map::new("committed_map");
pub const ABORTED_MAP:Map<u32, bool> = Map::new("aborted_map");


pub const CHANNELS: Map<u32, String> = Map::new("channels");
pub const HIGHEST_ABORT: Map<u32, i32> = Map::new("highest_abort");