use std::env;

use cosmwasm_std::{Deps, DepsMut, Env, IbcTimeout, Storage, Timestamp};

use crate::{error::{self, ContractError}, state::{Transaction, START_TIME}};

pub fn check_queue_top(
    queue: Vec<u32>,
    tx_id: u32,
) -> bool {
    if queue.is_empty() {
       return false;
    }
    if queue[0] == tx_id {
        return true;
    }
    false
}





pub fn remove_all_pending_elements(
    queue: &mut Vec<u32>,
    tx_id: u32,
) -> Vec<u32>{
    // remove all elements that are after the element with the given tx_id, not before the elemen
    if let Some(pos) = queue.iter().position(|tx_id| tx_id == tx_id) {
        let removed_elements = queue.split_off(pos);
        removed_elements.into_iter().map(|tx_id| tx_id).collect()
    } else {
        vec![]
    }

}


const PACKET_LIFETIME: u64 = 3600000;
pub fn get_timeout(env: &Env) -> IbcTimeout {
    let timeout = env.block.time.plus_seconds(PACKET_LIFETIME);
    IbcTimeout::with_timestamp(timeout)
}

pub fn get_seconds_diff(start: &Timestamp, end: &Timestamp) -> u64 {
    return end.seconds()-start.seconds();
} 
