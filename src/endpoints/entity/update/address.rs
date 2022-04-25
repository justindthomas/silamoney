use serde::{Deserialize, Serialize};
use log::error;
use web3::{
    types::H160,
    types::H256
};

use crate::{header_message, sila_signatures, hash_message, Header, HeaderMessage, Signatures, SignaturesParams};
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

pub struct UpdateAddressParams {
    pub customer_sila_handle: String,
    pub customer_eth_address: H160,
    pub private_key: Option<H256>,
    pub uuid: String,
    pub address_alias: Option<String>,
    pub street_address_1: Option<String>,
    pub street_address_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>
}

impl Default for UpdateAddressParams {
    fn default() -> Self { 
        UpdateAddressParams {  
            customer_sila_handle: String::new(),
            customer_eth_address: H160::zero(),
            private_key: Option::from(H256::zero()),
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

pub async fn update_address(params: &UpdateAddressParams) -> Result<UpdateAddressResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/address", sila_params.gateway);

    let header_message: HeaderMessage = header_message().await?;
    let mut header = header_message.header.clone();
    header.user_handle = params.customer_sila_handle.clone();
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
    
    let cb = || {};

    let signatures: Signatures = sila_signatures(&SignaturesParams {
        address: params.customer_eth_address.clone(),
        private_key: params.private_key.clone(),
        data: hash_message(serde_json::to_string(&message)?)
    }).await?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", signatures.usersignature.unwrap())
        .header("authsignature", signatures.authsignature)
        .json(&message)
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