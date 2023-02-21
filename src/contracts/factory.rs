use crate::{d, QueryResponseHandler, FACTORY_CRONTRACT, HandleResponseTypes};
use cosmrs::proto::cosmwasm::wasm::v1::{
    QuerySmartContractStateRequest, QuerySmartContractStateResponse,
};
use cosmrs::proto::prost::Message;
use serde::Serialize;
use eyre::Report;
use serde_json::json;
use tendermint_rpc::Method;
use crate::utils::{create_json_rpc_request, get_request_params, RequestValueAndHandler};

// Factory
use croncat_sdk_factory::msg::Config as FactoryConfigResponse;
use croncat_sdk_factory::msg::FactoryQueryMsg::Config as FactoryConfigRequest;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum FactoryQueryMethod {
    Config,
    LatestContracts,
    // There are more factory query methods
    // We'll add here if they'll be called
}

/// Handlers can optionally return a value
impl FactoryQueryMethod {
    pub(crate) async fn handle(&self, value: &Vec<u8>) -> Option<HandleResponseTypes> {
        match self {
            FactoryQueryMethod::Config => Some(HandleResponseTypes::FactoryQueryConfig(
                handle_factory_config(value).expect("Couldn't handle factory config"),
            )),
            _ => None,
        }
    }
}

pub fn handle_factory_config(
    response_value: &Vec<u8>,
) -> Result<FactoryConfigResponse, Report> {
    let decoded_response = QuerySmartContractStateResponse::decode(response_value.as_slice());
    // d("decoded_response", decoded_response.clone());
    let data = decoded_response.unwrap().data;
    let factory_config_response: FactoryConfigResponse = serde_json::from_slice(data.as_slice())?;
    d("factory_config_response", factory_config_response.clone());
    Ok(factory_config_response)
}

pub fn create_factory_config_request(json_rpc_id: u32) -> Result<RequestValueAndHandler, Report> {
    // Manager Config
    let query_method_plain = json!(&FactoryConfigRequest {}).to_string();
    let query_msg = QuerySmartContractStateRequest {
        address: FACTORY_CRONTRACT.to_string(),
        query_data: query_method_plain.into_bytes(),
    };

    let request_params =
        get_request_params(query_msg).expect("Issue getting factory config query params");

    // TODO see if we can pass the id by reference
    let query_request = create_json_rpc_request(
        json_rpc_id.clone(),
        Method::AbciQuery,
        request_params.clone(),
    );
    let response_handler = QueryResponseHandler::Factory {
        method: FactoryQueryMethod::Config,
    };
    Ok(RequestValueAndHandler {
        val: query_request.into(),
        handler: response_handler,
    })
}
