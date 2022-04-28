use serde::{Deserialize, Serialize};
use log::error;
use web3::types::H160;

use crate::{header_message, Header, HeaderMessage, SignedMessageParams};
use crate::endpoints::entity::*;

#[derive(Deserialize, Serialize)]
pub struct UpdateAddressResponse {
    pub success: bool,
    pub message: Option<String>,
    pub address: Option<Address>,
    pub status: Option<String>,
    pub reference: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct UpdateAddressMessage {
    pub header: Header,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address_1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address_2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>
}

pub struct UpdateAddressMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub uuid: String,
    pub address_alias: Option<String>,
    pub street_address_1: Option<String>,
    pub street_address_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>
}

impl Default for UpdateAddressMessageParams {
    fn default() -> Self { 
        UpdateAddressMessageParams {  
            sila_handle: String::new(),
            ethereum_address: H160::zero(),
            uuid: String::new(),
            address_alias: None,
            street_address_1: None,
            street_address_2: None,
            city: None,
            state: None,
            postal_code: None,
            country: None 
        } 
    } 
}

pub async fn update_address_message(
    params: &UpdateAddressMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let header_message: HeaderMessage = header_message();
    let mut header = header_message.header.clone();
    header.user_handle = Option::from(params.sila_handle.clone());
    header.auth_handle = sila_params.app_handle.clone();

    let message = UpdateAddressMessage {
        header: header,
        uuid: params.uuid.clone(),
        address_alias: params.address_alias.clone(),
        street_address_1: params.street_address_1.clone(),
        street_address_2: params.street_address_2.clone(),
        city: params.city.clone(),
        state: params.state.clone(),
        postal_code: params.postal_code.clone(),
        country: params.country.clone()
    };

    Ok(serde_json::to_string(&message)?)
}

pub async fn update_address(params: &SignedMessageParams) -> Result<UpdateAddressResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/address", sila_params.gateway);

    let h: UpdateAddressMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<UpdateAddressResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("update_address API Error: String({})", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("JSON Decoding Error: String({})", response_text);
            Err(Box::from(e))
        }
    }
}