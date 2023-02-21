/// CronCat contract logic
pub mod contracts;
pub mod utils;

use crate::contracts::factory::FactoryQueryMethod;
use eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use tendermint_rpc::endpoint::abci_query::Response;
use crate::contracts::agents::AgentsQueryMethod;
use crate::contracts::manager::ManagerQueryMethod;
use crate::contracts::tasks::TasksQueryMethod;
use crate::utils::{create_config_request, d, RequestValueAndHandler};
use croncat_sdk_factory::msg::Config as FactoryConfigResponse;
use croncat_sdk_agents::types::Config as AgentConfigResponse;
use croncat_sdk_tasks::types::Config as TasksConfigResponse;
use croncat_sdk_manager::types::Config as ManagerConfigResponse;

/// Note: there is a Wrapper available in tendermint-rs
/// But it forces randomized uuids, and we'd rather set them ourselves
/// See: https://github.com/informalsystems/tendermint-rs/blob/5e46d85d33b453ab471ac82dae890d20ce4bef30/rpc/src/request.rs#L40-L76
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
struct JsonRpcQueryAbciResponse {
    jsonrpc: String,
    id: u32,
    result: Response,
}

/// Path to query a smart contract method via abci
/// cosmjs-types could be a resource to find other paths:
/// https://github.com/confio/cosmjs-types/blob/3127b418321df165177e0f220a0ba34c6cec6ec6/src/cosmwasm/wasm/v1/query.ts#L1655
/// also see cosmos-rust here:
/// https://github.com/cosmos/cosmos-rust/blob/c55ff8daf8254dd3bb1a8b49c3ea23206499c6ca/cosmos-sdk-proto/src/prost/wasmd/cosmwasm.wasm.v1.rs#L811
const SMART_CONTRACT_STATE: &str = "/cosmwasm.wasm.v1.Query/SmartContractState";
/// Current CronCat Factory address on Juno testnet
const FACTORY_CRONTRACT: &str = "juno1x82wr3jkfurkgm8za3vayjr5ty932vn8nsmauvkxe4n35aj632tq5lguvl";
const TASKS_CRONTRACT: &str = "juno1mw4pe0qn3e2hlyca3keh6jw958ssxkzhzhs2pxursudxnrfhzkysk0930y";
const MANAGER_CRONTRACT: &str = "juno167qx4hlrmfweuxt53cnw76jtf0l2ctw5eujg4c89qre0glj79pfqeljykk";
const AGENTS_CRONTRACT: &str = "juno1lfh09uejwqrjx376gzsqfcdwf2pydnf0jp3gxxvxalceqphe2n7sqwr4k0";
/// Juno testnet endpoint
const RPC_ENDPOINT: &str = "https://rpc.uni.junonetwork.io";

/// Query methods, per contract
#[derive(Clone, Debug, Serialize)]
pub enum QueryResponseHandler {
    Factory { method: FactoryQueryMethod },
    Manager { method: ManagerQueryMethod },
    Tasks { method: TasksQueryMethod },
    Agents { method: AgentsQueryMethod },
}

/// Arranging proper response type (from croncat-sdk-*)
#[derive(Clone, Debug)]
pub enum HandleResponseTypes {
    FactoryQueryConfig(FactoryConfigResponse),
    ManagerQueryConfig(ManagerConfigResponse),
    TasksQueryConfig(TasksConfigResponse),
    AgentsQueryConfig(AgentConfigResponse),
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Keep track of JSON RPC id » query method handler
    let mut id_to_type: HashMap<u32, RequestValueAndHandler> = HashMap::new();

    // Query config on Manager, Tasks, and Agents to see if they're paused
    let manager_config_handler = QueryResponseHandler::Manager {
        method: ManagerQueryMethod::Config,
    };
    let tasks_config_handler = QueryResponseHandler::Tasks {
        method: TasksQueryMethod::Config,
    };
    let agents_config_handler = QueryResponseHandler::Agents {
        method: AgentsQueryMethod::Config,
    };
    let config_requests: Vec<QueryResponseHandler> = vec![
        manager_config_handler,
        tasks_config_handler,
        agents_config_handler,
    ];

    // Add JSON RPC ids, mapping the id to the handler, ultimately
    for json_rpc_id in 0..config_requests.len() {
        let handler = config_requests.get(json_rpc_id).expect("Couldn't get ID from config requests");
        let id_32 = json_rpc_id as u32; // Convenience
        let manager_config = create_config_request(id_32, handler).expect("Issue getting config payload");

        // Add to HashMap: JSON RPC id » ResponseHandler
        id_to_type.insert(id_32, manager_config);
    }

    // Get all the JSON into an array, essentially
    let all_config_req_vals: Vec<Value> = id_to_type.iter().map(|c| c.1.val.clone()).collect();
    // d("all_config_req_vals", all_config_req_vals.clone());
    let multiple_reqs_readable = serde_json::to_string(&all_config_req_vals)?;
    d("multiple_reqs_readable", multiple_reqs_readable.clone());

    // Now do the POST to the RPC endpoint
    let client = reqwest::Client::new();

    let res: reqwest::Response = client
        .post(RPC_ENDPOINT)
        .body(multiple_reqs_readable) // Here's where we send the array of JSON RPC queries
        .send()
        .await?;
    // We're going to sort results, so it's mutable
    let mut json_responses: Vec<JsonRpcQueryAbciResponse> = res.json::<Vec<JsonRpcQueryAbciResponse>>().await?;

    // Sort results by JSON RPC `id`, ascending
    json_responses.sort_by(|a, b| a.id.cmp(&b.id));
    d("sorted json_results", &json_responses.clone());

    // Loop through the responses, not worrying about return values
    let mut response_value: Vec<u8> = vec![];
    for json_response in json_responses.iter() {
        response_value = json_response.clone().result.response.value;
        match &id_to_type.get(&json_response.id).unwrap().handler {
            QueryResponseHandler::Factory { method } => {
                method.handle(&response_value).await;
            }
            QueryResponseHandler::Manager { method } => {
                method.handle(&response_value).await;
            }
            QueryResponseHandler::Tasks { method } => {
                method.handle(&response_value).await;
            }
            QueryResponseHandler::Agents { method } => {
                method.handle(&response_value).await;
            }
        }
    }

    Ok(())
}
