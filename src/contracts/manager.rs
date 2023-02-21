use crate::{QueryResponseHandler, MANAGER_CRONTRACT, HandleResponseTypes};
use cosmrs::proto::cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse};
use cosmrs::proto::prost::Message;
use eyre::Report;
use serde_json::json;
use tendermint_rpc::Method;
use serde::Serialize;
use crate::utils::{create_json_rpc_request, d, get_request_params, RequestValueAndHandler};

// Manager
use croncat_sdk_manager::msg::ManagerQueryMsg::Config as ManagerConfigRequest;
use croncat_sdk_manager::types::Config as ManagerConfigResponse;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum ManagerQueryMethod {
    Config,
    TreasuryBalance,
    // There are more factory query methods
    // We'll add here if they'll be called
}

impl ManagerQueryMethod {
    pub(crate) async fn handle(&self, value: &Vec<u8>) -> Option<HandleResponseTypes> {
        match self {
            ManagerQueryMethod::Config => {
                Some(HandleResponseTypes::ManagerQueryConfig(
                    handle_manager_config(value).expect("Couldn't handle agents config"),
                ))
            },
            _ => None,
        }
    }
}

pub fn handle_manager_config(
    response_value: &Vec<u8>,
) -> Result<ManagerConfigResponse, Report> {
    let decoded_response = QuerySmartContractStateResponse::decode(response_value.as_slice());
    // d("decoded_response", decoded_response.clone());
    let data = decoded_response.unwrap().data;
    let manager_config_response: ManagerConfigResponse = serde_json::from_slice(data.as_slice())?;
    d("manager_config_response", manager_config_response.clone());
    Ok(manager_config_response)
}

pub fn create_manager_config_request(json_rpc_id: u32) -> Result<RequestValueAndHandler, Report> {
    // Manager Config
    let query_method_plain = json!(&ManagerConfigRequest {}).to_string();
    let query_msg = QuerySmartContractStateRequest {
        address: MANAGER_CRONTRACT.to_string(),
        query_data: query_method_plain.into_bytes(),
    };

    let request_params =
        get_request_params(query_msg).expect("Issue getting manager config query params");

    // TODO see if we can pass the id by reference
    let query_request = create_json_rpc_request(
        json_rpc_id.clone(),
        Method::AbciQuery,
        request_params.clone(),
    );
    let response_handler = QueryResponseHandler::Manager {
        method: ManagerQueryMethod::Config,
    };
    Ok(RequestValueAndHandler {
        val: query_request.into(),
        handler: response_handler,
    })
}
