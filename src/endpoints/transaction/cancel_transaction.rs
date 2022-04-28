use crate::header_message;
use crate::HeaderMessage;
use crate::Header;
use log::error;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::{SignedMessageParams, Status};

#[derive(Deserialize, Serialize)]
pub struct CancelTransactionMessage {
    pub header: Header,
    pub transaction_id: String,
}

pub struct CancelTransactionMessageParams {
    pub sila_handle: Option<String>,
    pub ethereum_address: H160,
    pub reference: Option<String>,
    pub transaction_id: String
}

impl Default for CancelTransactionMessageParams {
    fn default() -> Self {
        CancelTransactionMessageParams {
            sila_handle: Option::from(String::new()),
            ethereum_address: H160::zero(),
            reference: Option::None,
            transaction_id: String::new()
        }
    }
}

pub async fn transfer_sila_message(
    params: &CancelTransactionMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let header_message: HeaderMessage = header_message();

    let mut message = CancelTransactionMessage {
        header: header_message.header,
        transaction_id: params.transaction_id.clone()
    };

    message.header.user_handle = params.sila_handle.clone();
    message.header.auth_handle = sila_params.app_handle.clone();

    Ok(serde_json::to_string(&message)?)
}

#[derive(Serialize, Deserialize)]
pub struct CancelTransactionResponse {
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
}

pub async fn cancel_transaction(
    params: &SignedMessageParams,
) -> Result<CancelTransactionResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/cancel_transaction", sila_params.gateway);

    let h: CancelTransactionMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<CancelTransactionResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("cancel_transaction failure: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!(
                "JSON decoding failure in cancel_transaction response: {}",
                response_text
            );
            Err(Box::from(e))
        }
    }
}
