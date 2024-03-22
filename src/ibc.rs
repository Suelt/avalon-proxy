use std::fmt::format;

use cosmwasm_std::{from_json, from_slice, to_binary, to_json_binary, Binary, ContractResult, Event, IbcMsg, IbcPacket, IbcTimeout, Storage, SubMsg, Timestamp};
use cosmwasm_std::{entry_point, DepsMut, Env, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Never, Response, StdResult};

use crate::error::ContractError;
use crate::msg::{AcknowledgementMsg, MsgQueueResponse, Op, WhoAmIResponse};
use crate::state::*;
use crate::utils::{check_queue_top, get_timeout, remove_all_pending_elements};



#[entry_point]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelOpenMsg,        
) -> StdResult<()>{

    Ok(())
}


#[entry_point]
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,        
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let channel_id=&channel.endpoint.channel_id;
    let mut state: State = STATE.load(deps.storage)?;
    state.node_number+=1;
    state.channel_ids.push(channel_id.to_string());
    STATE.save(deps.storage, &state)?;
    let packet = Op::WhoAmI {
        chain_id: state.chain_id,
    };
    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_json_binary(&packet)?,
        timeout: get_timeout(&env)
    };

    Ok(IbcBasicResponse::new()
        .add_message(msg)
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", channel_id))
}

#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,        
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let channel_id = &channel.endpoint.channel_id;
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", channel_id))
}


#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,        
) -> StdResult<IbcReceiveResponse> {
    // let packet = msg.packet;
    // do_ibc_packet_receive(deps, env, packet).or_else(
    //     |e|{
    //         let error = encode_ibc_error(format!("Failed to process IBC packet: {}", e));
    //         Ok(IbcReceiveResponse::new()
    //         .set_ack(error)
    //         .add_attribute("action", "error"))
    //     }
    // )


    (|| {
        let packet = msg.packet;
        let dest_channel_id = packet.dest.channel_id;
        let msg: Op = from_json(&packet.data)?;
        // which local channel did this packet come on
        // let dest_channel_id = packet.dest.channel_id;
        match msg{
            Op::DirtySuccess{value} => {
                // ...
                handle_dirty_success(deps.storage, env, value)
            },
            Op::Abortion{value} => {
                // ...
                handle_abortion(deps.storage, env, value)
            },
            Op::PrepareSuccess{value} => {
                // ...
                handle_prepare_success(deps.storage, env, value)
            },
            Op::WhoAmI { chain_id } => {
                // ...
                receive_who_am_i(deps ,dest_channel_id,chain_id)
               
            }
    
       
        }
    }
    )()
    .or_else(|e| {
        // we try to capture all app-level errors and convert them into
        // acknowledgement packets that contain an error code.
        let acknowledgement = encode_ibc_error(format!("invalid packet: {}", e));
        Ok(IbcReceiveResponse::new()
            .set_ack(acknowledgement)
            .add_event(Event::new("ibc").add_attribute("packet", "receive")))
    })
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,        
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}




fn do_ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    packet: IbcPacket,
) -> StdResult<IbcReceiveResponse> {

    let msg: Op = from_slice(&packet.data)?;
    let dest_channel_id = packet.dest.channel_id;
    match msg{
        Op::DirtySuccess{value} => {
            // ...
            handle_dirty_success(deps.storage, env, value)
        },
        Op::Abortion{value} => {
            // ...
            handle_abortion(deps.storage, env, value)
        },
        Op::PrepareSuccess{value} => {
            // ...
            handle_prepare_success(deps.storage, env, value)
        },
        Op::WhoAmI { chain_id } => {
            // ...
            receive_who_am_i(deps, dest_channel_id,chain_id)
           
        }

   
    }
}

fn receive_who_am_i(
    deps: DepsMut,
    channel_id: String,
    chain_id: u32,
) -> StdResult<IbcReceiveResponse> {
   

    let action = |_| -> StdResult<String> { Ok(channel_id.to_string()) };
    CHANNELS.update(deps.storage, chain_id, action)?;

    // initialize the highest_request of that chain
    // let action = |_| -> StdResult<u32> { Ok(0) };
    // HIGHEST_REQ.update(deps.storage, chain_id, action)?;
    // initialize the highest_request of that chain
    HIGHEST_ABORT.save(deps.storage, chain_id, &-1)?;

    let response = WhoAmIResponse {};
    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(response))?;
    // and we are golden
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_who_am_i")
        .add_attribute("chain_id", chain_id.to_string()))
}


fn handle_dirty_success(
    store: & mut dyn Storage,
    env: Env,
    value: u32,
) -> StdResult<IbcReceiveResponse> {
    // ...
    // check if dirty votes map for the key value is empty

    let mut state = STATE.load(store)?;
    state.dirty_votes+=1;
    STATE.save(store, &state)?;
    
    let msgs = check_dirty(store, env.clone(), value, get_timeout(&env)).unwrap();
    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(MsgQueueResponse { }))?; 
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_messages(msgs)
        .add_attribute("action", "handle_dirty_success"))
}


fn handle_prepare_success(
    store: &mut dyn Storage,
    env: Env,
    value: u32,
) -> StdResult<IbcReceiveResponse> {
    // ...

    
    let mut state = STATE.load(store)?;
    state.prepare_votes+=1;
    STATE.save(store, &state)?;
    let mut msgs = handle_deadlock(store, env.clone(), value).unwrap();



    let mut state = STATE.load(store)?;
    // if the number of votes is equal to the number of nodes, remove the transaction from the prepare queue and add it to the commit queue
    // moreover, if a subsequent transaction exists in the prepare_tx_queue, add the votes by 1 and send the prepare success message to other blockchains.
    if state.prepare_votes == state.node_number{
        state.prepare_tx_queue.remove(0);
        state.commit_tx_queue.push(value);
        // if !state.prepare_tx_queue.is_empty(){
        //     let value = state.prepare_tx_queue[0].tx_id;
        //     if PREPARE_VOTES_MAP.may_load(store, value)?.is_none(){
        //         PREPARE_VOTES_MAP.save(store, value, &1)?;
        //     }else{
        //         let votes = PREPARE_VOTES_MAP.load(store, value)?;
        //         PREPARE_VOTES_MAP.save(store, value, &(votes+1))?;
        //     }
        //     for channel_id in state.channel_ids.iter() {
        //         let packet = Op::PrepareSuccess { value };
        //         let ibc_msg = IbcMsg::SendPacket {
        //             channel_id: channel_id.clone(),
        //             data: to_binary(&packet).unwrap(),
        //             timeout: get_timeout(&env),
        //         };
        //         // 将新创建的消息添加到msgs向量中
        //         msgs.push(ibc_msg);
        //     }
        // }

        state.committed=true;
        state.end_time=Some(env.block.time);
        STATE.save(store, &state)?;

    }



    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(MsgQueueResponse { }))?; 
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_messages(msgs)
        .add_attribute("action", "handle_prepare_success"))
}






fn handle_abortion(
    store: &mut dyn Storage,
    env: Env,
    value: u32,
) -> StdResult<IbcReceiveResponse> {
    // ...
    let mut state = STATE.load(store)?;
    // remove_all_pending_elements(&mut state.prepare_tx_queue, value);
    // let mut msgs :Vec<IbcMsg> = Vec::new();
    // if !removed_elements_prepare.is_empty(){
    //     for tx_id in removed_elements_prepare{
    //         msgs.append(& mut upon_failure(store, env.block.time.clone(), tx_id, get_timeout(&env)).unwrap());
    //     }
    // }
    // remove_all_pending_elements(&mut state.dirty_tx_queue, value);
    STATE.save(store, &state)?;

    let acknowledgement = to_binary(&AcknowledgementMsg::Ok(MsgQueueResponse { }))?; 
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        // .add_messages(msgs)
        .add_attribute("action", "handle_failure"))
}




fn encode_ibc_error(msg: impl Into<String>) -> Binary {
    // this cannot error, unwrap to keep the interface simple
    to_binary(&ContractResult::<()>::Err(msg.into())).unwrap()
}


fn handle_deadlock(
    store: &mut dyn Storage,
    env: Env,
    tx_id: u32,
) -> Result<Vec<IbcMsg>, ContractError>{
    // ...
    let mut state = STATE.load(store)?;
    // if the transaction is not at the top of the prepare queue, remove it and abort all subsequent transactions in the prepare queue, including it, since it indicates a deadlock
    // however, if the transaction even does not exist in the prepare queue, needs to check whether it is the top element of the dirty queue, if so, do not see it as a deadlock
    // if the transaction is not at the top of the dirty queue, remove it since even though it is promoted to the prepare queue, it is not the top element 
    let is_top_prepare = check_queue_top(state.prepare_tx_queue.clone(), tx_id);
    let is_top_dirty=check_queue_top(state.dirty_tx_queue.clone(), tx_id);
    let mut msgs: Vec<IbcMsg> = Vec::new();
    if !is_top_prepare && !is_top_dirty{
        let removed_elements = remove_all_pending_elements(&mut state.prepare_tx_queue, tx_id);
        STATE.save(store, &state)?;
        for tx_id in removed_elements{
            msgs = upon_failure(store, env.block.time.clone(), tx_id, get_timeout(&env))?;
        }
        // check if dirty_tx_queue is empty
        if !state.dirty_tx_queue.is_empty(){
            let tx_id = state.dirty_tx_queue[0];
            let removed_elements = remove_all_pending_elements(& mut state.prepare_tx_queue, tx_id);
            for tx_id in removed_elements{
                msgs.append(& mut upon_failure(store, env.block.time.clone(), tx_id, get_timeout(&env))?);
            }   
        }
        
    }
    Ok(msgs)

}

pub fn check_dirty(
    store: &mut dyn Storage,
    env: Env,
    tx_id: u32,
    timeout: IbcTimeout,
) -> Result<Vec<IbcMsg>, ContractError> {
   
    let mut state = STATE.load(store)?;
    // check if dirty votes map for the key value is empty, if so, use mayload
    let mut msgs:Vec<IbcMsg> = Vec::new();
    let dirty_votes = state.dirty_votes;
    if dirty_votes == state.node_number{
        let is_top = check_queue_top(state.dirty_tx_queue.clone(), tx_id);
        if is_top{
            // if the transaction is at the top of the dirty queue, remove it and move it to the prepare queue
           // if not, remove it and abort all subsequent transactions in the dirty queue, including it
            state.prepare_tx_queue.push(state.dirty_tx_queue.get(0).unwrap().clone());
            STATE.save(store, &state)?;
            msgs = upon_dirty_success(store, tx_id, timeout.clone()).unwrap();

        } else {
            let removed_elements = remove_all_pending_elements(&mut state.dirty_tx_queue, tx_id);
            STATE.save(store, &state)?;
            for tx_id in removed_elements{
                msgs.append(&mut upon_failure(store, env.block.time.clone(), tx_id, timeout.clone()).unwrap());
            }

            
        }
    }
    Ok(msgs)
  
}




pub fn upon_dirty_success(
    store: &mut dyn Storage,
    tx_id: u32,
    timeout: IbcTimeout,
) ->  Result<Vec<IbcMsg>, ContractError>  {
    let mut state = STATE.load(store)?;
    let is_top =  check_queue_top(state.prepare_tx_queue.clone(), tx_id);
    let mut msgs: Vec<IbcMsg> = Vec::new();
    if is_top{
        // let state = STATE.load(store)?;

        for channel_id in state.channel_ids.iter() {
            let packet = Op::PrepareSuccess { value: tx_id };
            let ibc_msg = IbcMsg::SendPacket {
                channel_id: channel_id.clone(),
                data: to_binary(&packet).unwrap(),
                timeout: timeout.clone(),
            };
            // 将新创建的消息添加到msgs向量中
            msgs.push(ibc_msg);
        }
        state.prepare_votes+=1;
        STATE.save(store, &state)?;
    
    }
    Ok(msgs)
  
}

pub fn upon_failure(
    store: &mut dyn Storage,
    time: Timestamp,
    tx_id: u32,
    timeout: IbcTimeout,
) ->  Result<Vec<IbcMsg>, ContractError> {
    
    // instantiate a vector containing IbcMsg
    let mut state = STATE.load(store)?;
    let mut msgs: Vec<IbcMsg> = Vec::new();
    for channel_id in state.channel_ids.iter() {
        let packet = Op::Abortion { value: tx_id };
        let ibc_msg = IbcMsg::SendPacket {
            channel_id: channel_id.clone(),
            data: to_binary(&packet).unwrap(),
            timeout: timeout.clone(),
        };
        // 将新创建的消息添加到msgs向量中
        msgs.push(ibc_msg);
    }

    state.end_time=Some(time);
    state.aborted=true;
    STATE.save(store, &state)?;
    Ok(msgs)
}
