use serde::{Deserialize, Serialize};

use crate::{header_message, sila_signatures, Header, HeaderMessage, Signatures, SignaturesParams};
use crate::endpoints::entity::*;

#[derive(Deserialize, Serialize)]
pub struct UpdatePhoneParams {
    pub customer_sila_handle: String,
    pub customer_eth_address: String,
    pub customer_private_key: String,
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

pub async fn update_phone(params: &UpdatePhoneParams) -> Result<UpdatePhoneResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/email", sila_params.gateway);

    let header_message: HeaderMessage = header_message().await?;
    let mut header = header_message.header.clone();
    header.user_handle = params.customer_sila_handle.clone();
    header.auth_handle = sila_params.app_handle.clone();

    let message = UpdatePhoneMessage {
        header: header,
        uuid: params.uuid.clone(),
        phone: params.email.clone(),
        sms_opt_in: params.sms_opt_in.clone()
    };
    
    let signatures: Signatures = sila_signatures(&SignaturesParams {
        address: params.customer_eth_address.clone(),
        private_key: params.customer_private_key.clone(),
        data: serde_json::to_string(&message)? }).await?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", signatures.usersignature)
        .header("authsignature", signatures.authsignature)
        .json(&message)
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