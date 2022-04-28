use serde::{Deserialize, Serialize};
use web3::{
    types::H160,
};

use crate::{header_message, Header, HeaderMessage, SignedMessageParams };
use crate::endpoints::entity::*;

pub struct UpdatePhoneMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub uuid: String,
    pub email: Option<String>,
    pub sms_opt_in: Option<bool>
}

#[derive(Deserialize, Serialize)]
pub struct UpdatePhoneResponse {
    pub success: bool,
    pub message: Option<String>,
    pub phone: Option<PhoneResponse>,
    pub status: Option<String>,
    pub response_time_ms: Option<String>,
    pub reference: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct UpdatePhoneMessage {
    pub header: Header,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sms_opt_in: Option<bool>
}

pub async fn update_phone_message(
    params: &UpdatePhoneMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let header_message: HeaderMessage = header_message();
    let mut header = header_message.header.clone();
    header.user_handle = Option::from(params.sila_handle.clone());
    header.auth_handle = sila_params.app_handle.clone();

    let message = UpdatePhoneMessage {
        header: header,
        uuid: params.uuid.clone(),
        phone: params.email.clone(),
        sms_opt_in: params.sms_opt_in.clone()
    };

    Ok(serde_json::to_string(&message)?)
}

pub async fn update_phone(params: &SignedMessageParams) -> Result<UpdatePhoneResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/email", sila_params.gateway);

    let h: UpdatePhoneMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<UpdatePhoneResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("update_phone API Error: String({})", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("JSON Decoding Error: String({})", response_text);
            Err(Box::from(e))
        }
    }
}