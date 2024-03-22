
use cosmwasm_std::{entry_point, to_binary, to_json_binary, Binary, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Reply, Response, StdResult, Storage, Timestamp};

use crate::{error::ContractError, ibc::check_dirty, msg::{ExecuteMsg, GetTxResponse, InstantiateMsg, Op, QueryMsg}, state::{self, State, Transaction, ABORTED_MAP, COMMITTED_MAP, DIRTY_VOTES_MAP, END_TIME, PREPARE_VOTES_MAP, START_TIME, STATE}, utils::{check_queue_top, get_seconds_diff, get_timeout, remove_all_pending_elements}};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:simple-storage";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // ...
    let state = State::new(msg.chain_id, env.block.time);
    STATE.save(deps.storage, &state)?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg{
        ExecuteMsg::Input {value} => {
            handle_execute_input(deps, env, info, value)
        },
    }

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps, 
    env: Env, 
    msg: QueryMsg,
) -> StdResult<Binary> {
    // ...
    match msg{
        QueryMsg::GetTx {}=> to_json_binary(&handle_query_gettx(deps)?),
        QueryMsg::GetChainId {} => to_json_binary(&handle_query_getchainid(deps)?),
        QueryMsg::GetNodeNumber{} => to_json_binary(&handle_query_getnodenumber(deps)?),
        QueryMsg::GetDirtyTx {} => to_json_binary(&handle_query_getdirtytx(deps)?),
        QueryMsg::GetPrepareTx {} => to_json_binary(&handle_query_getpreparetx(deps)?),

    }

}

fn handle_query_getchainid(
    deps: Deps,
)-> StdResult<u32>{
    let state = STATE.load(deps.storage)?;
    Ok(state.chain_id)
    
}

fn handle_query_getnodenumber(
    deps: Deps,
)-> StdResult<u32>{
    let state = STATE.load(deps.storage)?;
    Ok(state.node_number)
    
}


fn handle_query_getpreparetx(
    deps: Deps,
)-> StdResult<Vec<u32>>{
    let state = STATE.load(deps.storage)?;
    Ok(state.prepare_tx_queue)
    
}

fn handle_query_getdirtytx(
    deps: Deps,
)-> StdResult<Vec<u32>>{
    let state = STATE.load(deps.storage)?;
    Ok(state.dirty_tx_queue)
    
}

fn handle_query_gettx(
    deps:Deps,
)-> StdResult<GetTxResponse>{
    // let start_time = START_TIME.load(deps.storage, val)?;
    // let end_time = END_TIME.load(deps.storage, val)?;
    let state = STATE.load(deps.storage)?;
    // load all the fields in gettxresponse in the contract storage, don't load from the State struct, but from the cosmwasmstd states
   
    // let time_in_seconds = get_seconds_diff(&state.start_time, &state.end_time.unwrap());

    Ok(
        GetTxResponse{
            dirty_votes: Some(state.dirty_votes),
            prepare_votes: Some(state.prepare_votes),
            committed: Some(state.committed),
            aborted: Some(state.aborted),
            start_time: state.start_time,
            end_time: state.end_time,
            time: Some(state.time),

            // time: Some(time_in_seconds),
        }
    )
    
    // Ok(
    //     GetTxResponse{
    //         dirty_votes: Some(state.dirty_votes),
    //         prepare_votes: Some(state.prepare_votes),
    //         committed: Some(state.committed),
    //         aborted: Some(state.aborted),
    //         start_time: state.start_time,
    //         end_time: state.end_time,
    //         time: Some(state.time),

    //         // time: Some(time_in_seconds),
    //     }
    
    // )

}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut, 
    env: Env,
    msg: Reply
) -> StdResult<Response> {
    // ...
    Ok(Response::default())
}


pub fn handle_execute_input (
    
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    input: Transaction,
) -> Result<Response, ContractError>{
    let timeout: IbcTimeout = get_timeout(&env);
    // if the txid index in the start time is empty, fill it as the current block time

   
    // push the initial transaction into the dirty tx queue
    let mut state = STATE.load(deps.storage)?;
    state.dirty_tx_queue.push(input.tx_id);
    state.start_time=env.block.time;
    state.dirty_votes+=1;
    STATE.save(deps.storage, &state)?;


    // generate a dirty success message
    let mut msgs: Vec<IbcMsg> = Vec::new();
        for channel_id in state.channel_ids.iter() {
            let packet = Op::DirtySuccess { value: input.tx_id };
            let ibc_msg = IbcMsg::SendPacket {
                channel_id: channel_id.to_string(),
                data: to_json_binary(&packet)?,
                timeout: timeout.clone(),
            };
            // 将新创建的消息添加到msgs向量中
            msgs.push(ibc_msg);
        }
    
    let response = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "handle_execute_input")
        .add_attribute("tx_id", input.tx_id.to_string());
    Ok(response)

    // let msgs = check_dirty(deps.storage, env, input.tx_id, timeout).unwrap();
    // // check if messages contains no ibcmsg
    // if msgs.is_empty(){
    //     return Ok(response);
    // }else{
    //     return Ok(response.add_messages(msgs));

    // }    

}
