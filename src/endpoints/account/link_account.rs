use serde::{Deserialize, Serialize};
use log::error;

use crate::{sila_signatures, Header, Signatures, SignaturesParams, Status};

pub struct LinkParams {
    pub customer_sila_handle: String,
    pub customer_eth_address: String,
    pub customer_private_key: String,
    pub sila_bank_identifier: String,
    pub sila_bank_token: String,
}

#[derive(Deserialize)]
pub struct LinkResponse {
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub account_name: Option<String>,
    pub match_score: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LinkMessage {
    pub header: Header,
    pub message: String,
    pub public_token: String,
    pub account_name: String,
}

pub async fn link_account_message() -> Result<LinkMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/getmessage?emptymessage=LinkAccountTestMessage", sila_params.gateway);

    let resp: LinkMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

pub async fn link_account(params: &LinkParams) -> Result<LinkResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/link_account", sila_params.gateway);

    let mut message: LinkMessage = link_account_message().await?;
    message.header.user_handle = params.customer_sila_handle.clone();
    message.header.auth_handle = sila_params.app_handle.clone();
    message.public_token = params.sila_bank_token.clone();
    message.account_name = params.sila_bank_identifier.clone();
    message.message = "link_account_msg".to_string();
    
    let signatures: Signatures = sila_signatures(&SignaturesParams {
        address: params.customer_eth_address.clone(),
        private_key: params.customer_private_key.clone(),
        data: serde_json::to_string(&message)?}).await?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", signatures.usersignature)
        .header("authsignature", signatures.authsignature)
        .json(&message)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response : Result<LinkResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general link_account error | text: {}", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}