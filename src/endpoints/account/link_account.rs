use log::error;
use serde::{Deserialize, Serialize};

use crate::{header_message, Header, HeaderMessage, SignedMessageParams, Status};

#[derive(Clone)]
pub struct LinkMessageParams {
    pub sila_handle: String,
    pub sila_bank_identifier: String,
    pub sila_bank_token: String,
    pub selected_account_id: String,
    pub account_name: Option<String>
}

impl std::fmt::Display for LinkMessageParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinkParams ( sila_handle: {}, bank_identifier: {}, bank_token: {}, selected_account_id: {}", 
            self.sila_handle,
            self.sila_bank_identifier,
            self.sila_bank_token,
            self.selected_account_id)
    }
}

#[derive(Deserialize, Serialize)]
pub struct LinkMessage {
    pub header: Header,
    pub plaid_token: String,
    pub account_name: String,
    pub selected_account_id: String,
}

impl From<LinkMessageParams> for LinkMessage {
    fn from(params: LinkMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header: HeaderMessage = header_message();
        header.header.user_handle = Option::from(params.sila_handle.clone());
        header.header.auth_handle = sila_params.app_handle.clone();

        let mut account_name = "default".to_string();
        if params.account_name.is_some() {
            account_name = params.account_name.unwrap();
        }
        
        LinkMessage {
            header: header.header,
            plaid_token: params.sila_bank_token.clone(),
            account_name,
            selected_account_id: params.selected_account_id.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct LinkResponse {
    pub success: bool,
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub account_name: Option<String>,
    pub match_score: Option<f32>,
    pub web_debit_verified: Option<bool>,
}

pub async fn link_account(
    params: &SignedMessageParams,
) -> Result<LinkResponse, Box<dyn std::error::Error + Sync + Send>> {
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
    let response: Result<LinkResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("link_account error: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("link_account response error: {}", response_text);
            Err(Box::from(e))
        }
    }
}
