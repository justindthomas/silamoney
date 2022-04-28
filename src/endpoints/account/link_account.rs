use serde::{Deserialize, Serialize};
use log::error;
use web3::{
    types::H160,
    types::H256
};

use crate::{Header, SignedMessageParams, Status};

pub struct LinkParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub private_key: Option<H256>,
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

pub async fn link_account_template() -> Result<LinkMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/getmessage?emptymessage=LinkAccountTestMessage", sila_params.gateway);

    let resp: LinkMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

pub async fn link_account_message(
    params: &LinkParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut message: LinkMessage = link_account_template().await?;
    message.header.user_handle = Option::from(params.sila_handle.clone());
    message.header.auth_handle = sila_params.app_handle.clone();
    message.public_token = params.sila_bank_token.clone();
    message.account_name = params.sila_bank_identifier.clone();
    message.message = "link_account_msg".to_string();

    Ok(serde_json::to_string(&message)?)
}

pub async fn link_account(params: &SignedMessageParams) -> Result<LinkResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/link_account", sila_params.gateway);

    let h: LinkMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
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