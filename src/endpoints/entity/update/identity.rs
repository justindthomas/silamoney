use serde::{Deserialize, Serialize};
use log::error;
use web3::types::H160;

use crate::{header_message, Header, HeaderMessage};
use crate::endpoints::entity::*;

#[derive(Clone)]
pub struct UpdateIdentityMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub uuid: String,
    pub identity_alias: String,
    pub identity_value: String
}

#[derive(Deserialize, Serialize)]
pub struct UpdateIdentityResponse {
    pub success: bool,
    pub message: Option<String>,
    pub identity: Option<IdentityResponse>,
    pub status: Option<String>,
    pub response_time_ms: Option<String>,
    pub reference: Option<String>
}

#[derive(Deserialize, Serialize)]
pub struct UpdateIdentityMessage {
    pub header: Header,
    pub uuid: String,
    pub identity_alias: String,
    pub identity_value: String
}

impl From<UpdateIdentityMessageParams> for UpdateIdentityMessage {
    fn from(params: UpdateIdentityMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        UpdateIdentityMessage {
            header: header_message.header,
            uuid: params.uuid.clone(),
            identity_alias: params.identity_alias.clone(),
            identity_value: params.identity_value.clone()
        }
    }
}

pub async fn update_identity(params: &SignedMessageParams) -> Result<UpdateIdentityResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/update/identity", sila_params.gateway);

    let h: UpdateIdentityMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<UpdateIdentityResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("update_identity API Error: String({})", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("JSON Decoding Error: String({})", response_text);
            Err(Box::from(e))
        }
    }
}