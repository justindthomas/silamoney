use crate::Header;
use log::error;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::{SignedMessageParams, Status};

#[derive(Deserialize, Serialize, Clone)]
pub enum IssueProcessingType {
    StandardAch,
    SameDayAch,
    InstantAch,
    InstantSettlement,
}

impl std::fmt::Display for IssueProcessingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IssueProcessingType::StandardAch => write!(f, "STANDARD_ACH"),
            IssueProcessingType::SameDayAch => write!(f, "SAME_DAY_ACH"),
            IssueProcessingType::InstantAch => write!(f, "INSTANCE_ACH"),
            IssueProcessingType::InstantSettlement => write!(f, "INSTANT_SETTLEMENT"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct IssueSilaMessage {
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
    pub processing_type: Option<IssueProcessingType>,
}

pub async fn issue_sila_template(
) -> Result<IssueSilaMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!(
        "{}/getmessage?emptymessage=IssueTestMessage",
        sila_params.gateway
    );

    let resp: IssueSilaMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

pub struct IssueSilaMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub amount: i32,
    pub account_name: Option<String>,
    pub descriptor: Option<String>,
    pub business_uuid: Option<String>,
    pub processing_type: Option<IssueProcessingType>,
}

impl Default for IssueSilaMessageParams {
    fn default() -> Self {
        IssueSilaMessageParams {
            sila_handle: String::new(),
            ethereum_address: H160::zero(),
            amount: 0,
            account_name: Option::from("default".to_string()),
            descriptor: Option::None,
            business_uuid: Option::None,
            processing_type: Option::from(IssueProcessingType::StandardAch),
        }
    }
}

pub async fn issue_sila_message(
    params: &IssueSilaMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut message: IssueSilaMessage = issue_sila_template().await?;
    message.header.user_handle = Option::from(params.sila_handle.clone());
    message.header.auth_handle = sila_params.app_handle.clone();

    message.message = "issue_msg".to_string();
    message.amount = params.amount;
    message.account_name = params.account_name.clone();
    message.descriptor = params.descriptor.clone();
    message.business_uuid = params.business_uuid.clone();
    message.processing_type = params.processing_type.clone();
    Ok(serde_json::to_string(&message)?)
}

#[derive(Serialize, Deserialize)]
pub struct IssueSilaResponse {
    pub message: String,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
    pub transaction_id: Option<String>,
    pub descriptor: Option<String>,
}

pub async fn issue_sila(
    params: &SignedMessageParams,
) -> Result<IssueSilaResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/issue_sila", sila_params.gateway);

    let h: IssueSilaMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<IssueSilaResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("issue_sila failure: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!(
                "JSON decoding failure in issue_sila response: {}",
                response_text
            );
            Err(Box::from(e))
        }
    }
}
