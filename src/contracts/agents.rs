use crate::{QueryResponseHandler, AGENTS_CRONTRACT, HandleResponseTypes};
use cosmrs::proto::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use cosmrs::proto::prost::Message;
use eyre::Report;
use serde_json::json;
use tendermint_rpc::Method;
use serde::Serialize;
use crate::utils::{create_json_rpc_request, d, get_request_params, RequestValueAndHandler};

// Agents
use croncat_sdk_agents::msg::QueryMsg::Config as AgentsConfigRequest;
use croncat_sdk_agents::types::Config as AgentsConfigResponse;

#[derive(Clone, Debug, Serialize)]
pub enum AgentsQueryMethod {
    Config,
    GetAgent,
    // There are more factory query methods
    // We'll add here if they'll be called
}

impl AgentsQueryMethod {
    pub(crate) async fn handle(&self, value: &Vec<u8>) -> Option<HandleResponseTypes> {
        match self {
            AgentsQueryMethod::Config => {
                Some(HandleResponseTypes::AgentsQueryConfig(
                    handle_agents_config(value).expect("Couldn't handle agents config"),
                ))
            },
            _ => None,
        }
    }
}

pub fn handle_agents_config(
    response_value: &Vec<u8>,
) -> Result<AgentsConfigResponse, Report> {
    let decoded_response = QuerySmartContractStateResponse::decode(response_value.as_slice());
    // d("decoded_response", decoded_response.clone());
    let data = decoded_response.unwrap().data;
    let agents_config_response: AgentsConfigResponse = serde_json::from_slice(data.as_slice())?;
    d("agents_config_response", agents_config_response.clone());
    Ok(agents_config_response)
}

pub fn create_agents_config_request(json_rpc_id: u32) -> Result<RequestValueAndHandler, Report> {
    // Tasks Config
    let query_method_plain = json!(&AgentsConfigRequest {}).to_string();
    let query_msg = QuerySmartContractStateRequest {
        address: AGENTS_CRONTRACT.to_string(),
        query_data: query_method_plain.into_bytes(),
    };

    let request_params =
        get_request_params(query_msg).expect("Issue getting agents config query params");

    // TODO see if we can pass the id by reference
    let query_request = create_json_rpc_request(
        json_rpc_id.clone(),
        Method::AbciQuery,
        request_params.clone(),
    );
    let response_handler = QueryResponseHandler::Agents {
        method: AgentsQueryMethod::Config,
    };
    Ok(RequestValueAndHandler {
        val: query_request.into(),
        handler: response_handler,
    })
}
