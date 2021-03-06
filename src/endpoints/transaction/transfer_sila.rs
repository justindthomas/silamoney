use log::error;
use serde::{Deserialize, Serialize};

use crate::{header_message, Header, HeaderMessage, SignedMessageParams, Status};

#[derive(Deserialize, Serialize)]
pub struct TransferSilaMessage {
    pub header: Header,
    pub amount: i32,
    pub message: String,
    pub destination_handle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_wallet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_id: Option<String>,
}

#[derive(Clone)]
pub struct TransferSilaMessageParams {
    pub sila_handle: String,
    pub amount: i32,
    pub descriptor: Option<String>,
    pub destination_handle: String,
    pub destination_address: Option<String>,
    pub destination_wallet: Option<String>,
    pub destination_id: Option<String>,
    pub source_id: Option<String>,
    pub reference: Option<String>,
}

impl Default for TransferSilaMessageParams {
    fn default() -> Self {
        TransferSilaMessageParams {
            sila_handle: String::new(),
            amount: 0,
            descriptor: Option::None,
            destination_handle: String::new(),
            destination_address: Option::None,
            destination_wallet: Option::None,
            destination_id: Option::None,
            source_id: Option::None,
            reference: Option::None,
        }
    }
}

impl From<TransferSilaMessageParams> for TransferSilaMessage {
    fn from(params: TransferSilaMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        if params.reference.is_some() {
            header_message.header.reference = params.reference.unwrap();
        }

        TransferSilaMessage {
            header: header_message.header,
            message: "transfer_msg".to_string(),
            amount: params.amount,
            descriptor: params.descriptor.clone(),
            destination_handle: params.destination_handle.clone(),
            destination_address: params.destination_address.clone(),
            destination_wallet: params.destination_wallet.clone(),
            destination_id: params.destination_id.clone(),
            source_id: params.source_id.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransferSilaResponse {
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
    pub transaction_id: Option<String>,
    pub descriptor: Option<String>,
}

impl std::fmt::Display for TransferSilaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TransferSilaResponse(message: {}, reference: {}, transaction_id: {}, status: {})",
            self.message,
            self.reference.as_ref().unwrap_or(&"none".to_string()),
            self.transaction_id.as_ref().unwrap_or(&"none".to_string()),
            self.status
        )
    }
}

pub async fn transfer_sila(
    params: &SignedMessageParams,
) -> Result<TransferSilaResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/transfer_sila", sila_params.gateway);

    let h: TransferSilaMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<TransferSilaResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("transfer_sila error: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("transfer_sila response error: {}", response_text);
            Err(Box::from(e))
        }
    }
}
