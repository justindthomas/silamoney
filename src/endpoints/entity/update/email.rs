use serde::{Deserialize, Serialize};
use log::error;
use web3::types::H160;

use crate::{header_message, Header, HeaderMessage, SignedMessageParams};
use crate::endpoints::entity::*;

#[derive(Clone)]
pub struct UpdateEmailMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub uuid: String,
    pub email: String
}

#[derive(Deserialize, Serialize)]
pub struct UpdateEmailResponse {
    pub success: bool,
    pub message: Option<String>,
    pub email: Option<EmailResponse>,
    pub status: Option<String>,
    pub response_time_ms: Option<String>,
    pub reference: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct UpdateEmailMessage {
    pub header: Header,
    pub uuid: String,
    pub email: String
}

impl From<UpdateEmailMessageParams> for UpdateEmailMessage {
    fn from(params: UpdateEmailMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        UpdateEmailMessage {
            header: header_message.header,
            uuid: params.uuid.clone(),
            email: params.email.clone()
        }
    }
}

pub async fn update_email(params: &SignedMessageParams) -> Result<UpdateEmailResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/email", sila_params.gateway);

    let h: UpdateEmailMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<UpdateEmailResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("update_email API Error: String({})", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("JSON Decoding Error: String({})", response_text);
            Err(Box::from(e))
        }
    }
}