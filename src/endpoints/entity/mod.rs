pub mod check_kyc;
pub mod register;
pub mod request_kyc;
pub mod update;

use crate::{
    header_message, Header, HeaderMessage,
    SignedMessageParams, Status,
};

use log::error;
use serde::{Deserialize, Serialize};
use web3::{types::H160, types::H256};

#[derive(Deserialize, Serialize)]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
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
    pub country: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Identity {
    pub identity_alias: String,
    pub identity_value: String,
}

#[derive(Deserialize, Serialize)]
pub struct Contact {
    pub phone: String,
    pub contact_alias: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct CryptoEntry {
    pub crypto_alias: String,
    pub crypto_status: String,
    pub crypto_address: String,
    pub crypto_code: String,
}

#[derive(Deserialize, Serialize)]
pub struct Entity {
    pub birthdate: String,
    pub entity_name: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct IdentityResponse {
    pub added_epoch: Option<i64>,
    pub modified_epoch: Option<i64>,
    pub uuid: Option<String>,
    pub identity_type: Option<String>,
    pub identity: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EmailResponse {
    pub added_epoch: Option<i64>,
    pub modified_epoch: Option<i64>,
    pub uuid: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct PhoneResponse {
    pub added_epoch: Option<i64>,
    pub modified_epoch: Option<i64>,
    pub uuid: Option<String>,
    pub phone: Option<String>,
    pub sms_confirmation_requested: Option<bool>,
    pub sms_confirmed: Option<bool>,
    pub primary: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct DeviceResponse {
    pub added_epoch: Option<i64>,
    pub modified_epoch: Option<i64>,
    pub uuid: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct MembershipResponse {
    pub business_handle: Option<String>,
    pub entity_name: Option<String>,
    pub role: Option<String>,
    pub details: Option<String>,
    pub ownership_stake: Option<f32>,
    pub certification_token: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct GetEntityResponse {
    pub success: bool,
    pub status: String,
    pub reference: Option<String>,
    pub response_time_ms: Option<String>,
    pub user_handle: Option<String>,
    pub entity_type: Option<String>,
    pub entity: Option<Entity>,
    pub addresses: Option<Vec<Address>>,
    pub identities: Option<Vec<IdentityResponse>>,
    pub emails: Option<Vec<EmailResponse>>,
    pub phones: Option<Vec<PhoneResponse>>,
    pub devices: Option<Vec<DeviceResponse>>,
    pub memberships: Option<Vec<MembershipResponse>>,
}

pub struct RequestEntityParams {
    pub customer_sila_handle: String,
    pub customer_eth_address: H160,
    pub private_key: Option<H256>,
}

pub struct RequestEntityMessageParams {
    pub sila_handle: String,
}

pub async fn get_entity_message(
    params: &RequestEntityMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut message: HeaderMessage = header_message();
    message.header.user_handle = Option::from(params.sila_handle.clone());
    message.header.auth_handle = sila_params.app_handle.clone();

    Ok(serde_json::to_string(&message)?)
}

pub async fn get_entity(
    params: &SignedMessageParams,
) -> Result<GetEntityResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/get_entity", sila_params.gateway);
    let h: HeaderMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response;

    match &params.usersignature {
        Some(x) => {
            resp = client
                .post(&_url.to_owned())
                .header("usersignature", x)
                .header("authsignature", params.authsignature.clone())
                .json(&h)
                .send()
                .await?;
        }
        None => {
            resp = client
                .post(&_url.to_owned())
                .header("authsignature", params.authsignature.clone())
                .json(&h)
                .send()
                .await?;
        }
    }

    let response_text = resp.text().await?;
    let response: Result<GetEntityResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("get_entity API Error: String({})", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("JSON Decoding Error: String({})", response_text);
            Err(Box::from(e))
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct CheckResponse {
    pub message: Option<String>,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
}

pub struct CheckHandleMessageParams {
    pub sila_handle: String,
}

pub async fn check_handle_message(
    params: &CheckHandleMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut header: HeaderMessage = header_message();
    header.header.user_handle = Option::from(params.sila_handle.clone());
    header.header.auth_handle = sila_params.app_handle.clone();

    Ok(serde_json::to_string(&header)?)
}

pub async fn check_handle(
    params: &SignedMessageParams,
) -> Result<CheckResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/check_handle", sila_params.gateway);

    let h: HeaderMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response;

    match &params.usersignature {
        Some(x) => {
            resp = client
                .post(&_url.to_owned())
                .header("usersignature", x)
                .header("authsignature", params.authsignature.clone())
                .json(&h)
                .send()
                .await?;
        }
        None => {
            resp = client
                .post(&_url.to_owned())
                .header("authsignature", params.authsignature.clone())
                .json(&h)
                .send()
                .await?;
        }
    }

    let response_text = resp.text().await?;
    let response: Result<CheckResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("check_handle failed: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("check_handle json decode failed: {}", response_text);
            Err(Box::from(e))
        }
    }
}
