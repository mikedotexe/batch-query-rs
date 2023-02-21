use crate::{HandleResponseTypes, QueryResponseHandler, TASKS_CRONTRACT};
use cosmrs::proto::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use cosmrs::proto::prost::Message;
use eyre::Report;
use serde_json::json;
use tendermint_rpc::Method;
use serde::Serialize;
use crate::utils::{create_json_rpc_request, d, get_request_params, RequestValueAndHandler};

// Tasks
use croncat_sdk_tasks::msg::TasksQueryMsg::Config as TasksConfigRequest;
use croncat_sdk_tasks::types::Config as TasksConfigResponse;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum TasksQueryMethod {
    Config,
    TasksTotal,
    // There are more factory query methods
    // We'll add here if they'll be called
}

impl TasksQueryMethod {
    pub(crate) async fn handle(&self, value: &Vec<u8>) -> Option<HandleResponseTypes> {
        match self {
            TasksQueryMethod::Config => {
                Some(HandleResponseTypes::TasksQueryConfig(
                    handle_tasks_config(value).expect("Couldn't handle tasks config"),
                ))
            },
            _ => None,
        }
    }
}

pub fn handle_tasks_config(
    response_value: &Vec<u8>,
) -> Result<TasksConfigResponse, Report> {
    let decoded_response = QuerySmartContractStateResponse::decode(response_value.as_slice());
    // d("decoded_response", decoded_response.clone());
    let data = decoded_response.unwrap().data;
    let tasks_config_response: TasksConfigResponse = serde_json::from_slice(data.as_slice())?;
    d("tasks_config_response", tasks_config_response.clone());
    Ok(tasks_config_response)
}

pub fn create_tasks_config_request(json_rpc_id: u32) -> Result<RequestValueAndHandler, Report> {
    // Tasks Config
    let query_method_plain = json!(&TasksConfigRequest {}).to_string();
    let query_msg = QuerySmartContractStateRequest {
        address: TASKS_CRONTRACT.to_string(),
        query_data: query_method_plain.into_bytes(),
    };

    let request_params =
        get_request_params(query_msg).expect("Issue getting tasks config query params");

    // TODO see if we can pass the id by reference
    let query_request = create_json_rpc_request(
        json_rpc_id.clone(),
        Method::AbciQuery,
        request_params.clone(),
    );
    let response_handler = QueryResponseHandler::Tasks {
        method: TasksQueryMethod::Config,
    };
    Ok(RequestValueAndHandler {
        val: query_request.into(),
        handler: response_handler,
    })
}
