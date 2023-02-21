use crate::contracts::agents::{create_agents_config_request, AgentsQueryMethod};
use crate::contracts::factory::{create_factory_config_request, FactoryQueryMethod};
use crate::contracts::manager::{create_manager_config_request, ManagerQueryMethod};
use crate::contracts::tasks::{create_tasks_config_request, TasksQueryMethod};
use crate::{QueryResponseHandler, SMART_CONTRACT_STATE};
use eyre::{eyre, Report};
use serde_json::{json, Value};
use std::fmt::Debug;
use cosmrs::proto::cosmwasm::wasm::v1::QuerySmartContractStateRequest;
use cosmrs::proto::prost::Message;
use serde::Serialize;
use tendermint_rpc::Method;
use tendermint_rpc::endpoint::abci_query::Request;

#[derive(Clone, Debug, Serialize)]
pub struct RequestValueAndHandler {
    pub(crate) val: Value,
    pub(crate) handler: QueryResponseHandler
}

pub fn create_config_request(
    json_rpc_id: u32,
    handler: &QueryResponseHandler,
) -> Result<RequestValueAndHandler, Report> {
    match handler {
        QueryResponseHandler::Factory { method } => match method {
                FactoryQueryMethod::Config => create_factory_config_request(json_rpc_id),
                _ => Err(eyre!("nope")),
        }
        QueryResponseHandler::Manager { method } => match method {
            ManagerQueryMethod::Config => create_manager_config_request(json_rpc_id),
            _ => Err(eyre!("nope")),
        },
        QueryResponseHandler::Tasks { method } => match method {
            TasksQueryMethod::Config => create_tasks_config_request(json_rpc_id),
            _ => Err(eyre!("nope")),
        },
        QueryResponseHandler::Agents { method } => match method {
            AgentsQueryMethod::Config => create_agents_config_request(json_rpc_id),
            _ => Err(eyre!("nope")),
        },
    }
}

// Creates JSON RPC params given a query message
// Undergoes protobuf encoding and whatnot
pub fn get_request_params(query_msg: QuerySmartContractStateRequest) -> Result<Request, Report> {
    // Encode the CronCat message calling Factory's `config` (does protobuf stuff)
    let mut my_buf = Vec::new();
    // This assigns the mutable into the encoded value
    QuerySmartContractStateRequest::encode(&query_msg, &mut my_buf)?;
    // d("my_buf", my_buf.clone());

    // In the JSON RPC request, we'll have a `params` key
    // Set that up here
    let request_params = Request {
        path: Some(SMART_CONTRACT_STATE.to_string()),
        data: my_buf.clone(),
        height: None, // can specify block height
        prove: false, // include the proof in the response
    };
    // d("request_params", request_params.clone());
    Ok(request_params)
}

/// Returns a JSON RPC response given the method and parameters
pub fn create_json_rpc_request(id: u32, method: Method, params: Request) -> Value {
    json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": method,
      "params": params
    })
}

/// Simple debugging thing
pub fn d<T>(msg: &str, o: T)
where
    T: Debug,
{
    println!("{}\n", format!("{}: {:?}", msg, o));
}
