use serde::{Deserialize, Serialize};
use log::error;
use web3::types::H160;

use crate::endpoints::entity::*;

#[derive(Deserialize, Serialize)]
pub struct RegisterMessage {
    pub header: Header,
    pub message: String,
    pub address: Address,
    pub identity: Identity,
    pub contact: Contact,
    pub crypto_entry: CryptoEntry,
    pub entity: Entity,
}

pub async fn register_message_template() -> Result<RegisterMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/getmessage?emptymessage=EntityTestMessage", sila_params.gateway);

    let resp: RegisterMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

pub struct RegisterMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub birthdate: String,
    pub first_name: String,
    pub last_name: String,
    pub street_address_1: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub phone: String,
    pub email: String,
    pub ssn: String,
}

pub async fn register_message(
    params: &RegisterMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut message: RegisterMessage = register_message_template().await?;
    message.header.user_handle = Option::from(params.sila_handle.clone());
    message.header.auth_handle = sila_params.app_handle.clone();

    message.entity.relationship = Option::from("user".to_string());
    message.entity.entity_name = "default".to_string();
    message.entity.first_name = params.first_name.clone();
    message.entity.last_name = params.last_name.clone();

    message.entity.birthdate = params.birthdate.clone();
    
    message.address.address_alias = Option::from("default".to_string());
    message.address.street_address_1 = Option::from(params.street_address_1.clone());
    message.address.city = Option::from(params.city.clone());
    message.address.state = Option::from(params.state.clone());
    message.address.postal_code = Option::from(params.postal_code.clone());
    message.address.country = Option::from("US".to_string());

    message.identity.identity_value = params.ssn.clone();
    
    message.contact.phone = params.phone.clone();
    message.contact.email = params.email.clone();

    message.crypto_entry.crypto_address = format!("{:#x}", params.ethereum_address.clone());
    message.crypto_entry.crypto_code = "ETH".to_string();
    
    Ok(serde_json::to_string(&message)?)
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub message: String,
    pub reference: String,
    pub status: Status,
}

pub async fn register(params: &SignedMessageParams) -> Result<RegisterResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/register", sila_params.gateway);

    let h: RegisterMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response : Result<RegisterResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general register error | text: {}", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}