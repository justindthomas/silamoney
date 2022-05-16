use crate::Header;
use log::error;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::{header_message, HeaderMessage, SignedMessageParams, Status};

#[derive(Deserialize, Serialize, Clone)]
pub enum RedeemProcessingType {
    StandardAch,
    SameDayAch,
}

impl std::fmt::Display for RedeemProcessingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RedeemProcessingType::StandardAch => write!(f, "STANDARD_ACH"),
            RedeemProcessingType::SameDayAch => write!(f, "SAME_DAY_ACH"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct RedeemSilaMessage {
    pub header: Header,
    pub amount: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descriptor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_type: Option<RedeemProcessingType>,
}

#[derive(Clone)]
pub struct RedeemSilaMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub amount: i32,
    pub account_name: Option<String>,
    pub descriptor: Option<String>,
    pub business_uuid: Option<String>,
    pub processing_type: Option<RedeemProcessingType>,
    pub reference: Option<String>,
}

impl Default for RedeemSilaMessageParams {
    fn default() -> Self {
        RedeemSilaMessageParams {
            sila_handle: String::new(),
            ethereum_address: H160::zero(),
            amount: 0,
            account_name: Option::from("default".to_string()),
            descriptor: Option::None,
            business_uuid: Option::None,
            processing_type: Option::from(RedeemProcessingType::StandardAch),
            reference: Option::None
        }
    }
}

impl From<RedeemSilaMessageParams> for RedeemSilaMessage {
    fn from(params: RedeemSilaMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        if params.reference.is_some() {
            header_message.header.reference = params.reference.unwrap();
        }

        RedeemSilaMessage {
            header: header_message.header,
            message: "redeem_msg".to_string(),
            amount: params.amount,
            account_name: params.account_name.clone(),
            descriptor: params.descriptor.clone(),
            business_uuid: params.business_uuid.clone(),
            processing_type: params.processing_type.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RedeemSilaResponse {
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
    pub transaction_id: Option<String>,
    pub descriptor: Option<String>,
}

impl std::fmt::Display for RedeemSilaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RedeemSilaResponse(message: {}, reference: {}, transaction_id: {}, status: {})",
            self.message,
            self.reference.as_ref().unwrap_or(&"none".to_string()),
            self.transaction_id.as_ref().unwrap_or(&"none".to_string()),
            self.status
        )
    }
}

pub async fn redeem_sila(
    params: &SignedMessageParams,
) -> Result<RedeemSilaResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/redeem_sila", sila_params.gateway);

    let h: RedeemSilaMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<RedeemSilaResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("redeem_sila error: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("redeem_sila response error: {}", response_text);
            Err(Box::from(e))
        }
    }
}
