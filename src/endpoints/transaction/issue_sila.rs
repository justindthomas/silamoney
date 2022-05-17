use crate::{header_message, Header, HeaderMessage, SignedMessageParams, Status};
use log::error;
use serde::{Deserialize, Serialize};
use web3::types::H160;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all="SCREAMING_SNAKE_CASE")]
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

#[derive(Clone)]
pub struct IssueSilaMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub amount: i32,
    pub account_name: Option<String>,
    pub descriptor: Option<String>,
    pub business_uuid: Option<String>,
    pub processing_type: Option<IssueProcessingType>,
    pub reference: Option<String>
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
            reference: Option::None,
        }
    }
}

impl From<IssueSilaMessageParams> for IssueSilaMessage {
    fn from(params: IssueSilaMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        if params.reference.is_some() {
            header_message.header.reference = params.reference.unwrap();
        }

        IssueSilaMessage {
            header: header_message.header,
            message: "issue_msg".to_string(),
            amount: params.amount,
            account_name: params.account_name.clone(),
            descriptor: params.descriptor.clone(),
            business_uuid: params.business_uuid.clone(),
            processing_type: params.processing_type.clone(),
        }
    }
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

impl std::fmt::Display for IssueSilaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IssueSilaResponse(message: {}, reference: {}, transaction_id: {}, status: {})",
            self.message,
            self.reference.as_ref().unwrap_or(&"none".to_string()),
            self.transaction_id.as_ref().unwrap_or(&"none".to_string()),
            self.status
        )
    }
}

pub async fn issue_sila(
    params: &SignedMessageParams,
) -> Result<IssueSilaResponse, Box<dyn std::error::Error>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/issue_sila", sila_params.gateway);

    let h: IssueSilaMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::ClientBuilder::new()
        .build()
        .unwrap();
        
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
            error!("issue_sila error: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(_) => {
            error!("issue_sila response error: {}", response_text);
            Err(Box::from("issue_sila response error"))
        }
    }
}
