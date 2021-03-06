use crate::header_message;
use crate::Header;
use crate::HeaderMessage;
use crate::IssueProcessingType;
use log::error;
use serde::{Deserialize, Serialize};

use crate::{SignedMessageParams, Status};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Issue,
    Redeem,
    Transfer,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransactionType::Issue => write!(f, "issue"),
            TransactionType::Redeem => write!(f, "redeem"),
            TransactionType::Transfer => write!(f, "transfer"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Queued,
    Pending,
    PendingConfirmation,
    Reversed,
    Failed,
    Success,
    Rollback,
    Review,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransactionStatus::Queued => write!(f, "queued"),
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::PendingConfirmation => write!(f, "pending_confirmation"),
            TransactionStatus::Reversed => write!(f, "reversed"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Success => write!(f, "success"),
            TransactionStatus::Rollback => write!(f, "rollback"),
            TransactionStatus::Review => write!(f, "review"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TransactionSearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_timelines: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_ascending: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sila_amount: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_sila_amount: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<TransactionStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_types: Option<Vec<TransactionType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bank_account_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blockchain_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_type: Option<IssueProcessingType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_id: Option<String>,
}

impl Default for TransactionSearchFilters {
    fn default() -> Self {
        TransactionSearchFilters {
            transaction_id: Option::None,
            reference_id: Option::None,
            show_timelines: Option::from(true),
            sort_ascending: Option::from(false),
            max_sila_amount: Option::None,
            min_sila_amount: Option::None,
            statuses: Option::from(vec![
                TransactionStatus::Queued,
                TransactionStatus::Pending,
                TransactionStatus::PendingConfirmation,
                TransactionStatus::Reversed,
                TransactionStatus::Failed,
                TransactionStatus::Success,
                TransactionStatus::Rollback,
                TransactionStatus::Review,
            ]),
            start_epoch: Option::None,
            end_epoch: Option::None,
            page: Option::from(1),
            per_page: Option::from(20),
            transaction_types: Option::from(vec![
                TransactionType::Issue,
                TransactionType::Redeem,
                TransactionType::Transfer,
            ]),
            bank_account_name: Option::None,
            blockchain_address: Option::None,
            processing_type: Option::None,
            payment_method_id: Option::None,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct GetTransactionsMessage {
    pub header: Header,
    pub message: String,
    pub search_filters: Option<TransactionSearchFilters>,
}

#[derive(Clone)]
pub struct GetTransactionsMessageParams {
    pub sila_handle: Option<String>,
    pub reference: Option<String>,
    pub search_filters: Option<TransactionSearchFilters>,
}

impl Default for GetTransactionsMessageParams {
    fn default() -> Self {
        GetTransactionsMessageParams {
            sila_handle: Option::from(String::new()),
            reference: Option::None,
            search_filters: Option::from(TransactionSearchFilters {
                ..Default::default()
            }),
        }
    }
}

impl From<GetTransactionsMessageParams> for GetTransactionsMessage {
    fn from(params: GetTransactionsMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;
 
        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        if params.reference.is_some() {
            header_message.header.reference = params.reference.unwrap();
        }
         
        GetTransactionsMessage {
            header: header_message.header,
            message: "get_transactions_msg".to_string(),
            search_filters: params.search_filters.clone(),
        }
     }
 }

#[derive(Serialize, Deserialize)]
pub struct TransactionTimelineItem {
    pub date: Option<String>,
    pub date_epoch: Option<i64>,
    pub status: Option<String>,
    pub usd_status: Option<String>,
    pub token_status: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub user_handle: Option<String>,
    pub reference_id: Option<String>,
    pub transaction_id: Option<String>,
    pub transaction_hash: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub sila_amount: Option<i32>,
    pub status: Option<TransactionStatus>,
    pub usd_status: Option<String>,
    pub token_status: Option<String>,
    pub created: Option<String>,
    pub last_update: Option<String>,
    pub created_epoch: Option<i64>,
    pub last_update_epoch: Option<i64>,
    pub descriptor: Option<String>,
    pub descriptor_ach: Option<String>,
    pub ach_name: Option<String>,
    pub bank_account_name: Option<String>,
    pub processing_type: Option<String>,
    pub submitted: Option<String>,
    pub submitted_epoch: Option<i64>,
    pub trace_number: Option<String>,
    pub addenda: Option<String>,
    pub error_code: Option<String>,
    pub error_msg: Option<String>,
    pub return_code: Option<String>,
    pub return_desc: Option<String>,
    pub destination_address: Option<String>,
    pub destination_handle: Option<String>,
    pub handle_address: Option<String>,
    pub source_id: Option<String>,
    pub destination_id: Option<String>,
    pub sec_code: Option<String>,
    pub timeline: Option<Vec<TransactionTimelineItem>>,
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction (reference_id: {}, user_handle: {}, transaction_type: {}, sila_amount: {})", 
            self.reference_id.clone().unwrap(), 
            self.user_handle.clone().unwrap(), 
            self.transaction_type.clone().unwrap(), 
            self.sila_amount.unwrap() )
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionPagination {
    pub returned_count: Option<i32>,
    pub total_count: Option<i32>,
    pub current_page: Option<i32>,
    pub total_pages: Option<i32>,
}

impl std::fmt::Display for TransactionPagination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetTransactionsResponse {
    pub message: Option<String>,
    pub reference: Option<String>,
    pub status: Status,
    pub success: bool,
    pub page: Option<i32>,
    pub returned_count: Option<i32>,
    pub total_count: Option<i32>,
    pub pagination: Option<TransactionPagination>,
    pub transactions: Option<Vec<Transaction>>,
}

impl std::fmt::Display for GetTransactionsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub async fn get_transactions(
    params: &SignedMessageParams,
) -> Result<GetTransactionsResponse, Box<dyn std::error::Error>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/get_transactions", sila_params.gateway);


    let h: GetTransactionsMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::ClientBuilder::new()
        .build()
        .unwrap();

    let resp: reqwest::Response;

    match &params.usersignature {
        Some(x) => {
            resp = client
                .post(&_url.to_owned())
                .header("usersignature", x)
                .header("authsignature", &params.authsignature)
                .json(&h)
                .send()
                .await
                .unwrap();
        }
        None => {
            resp = client
                .post(&_url.to_owned())
                .header("authsignature", &params.authsignature)
                .json(&h)
                .send()
                .await
                .unwrap();
        }
    }

    let response_text = resp.text().await.unwrap();
    let response: Result<GetTransactionsResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("get_transactions failure: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(_) => {
            error!(
                "JSON decoding failure in cancel_transaction response: {}",
                response_text
            );
            Err(Box::from("JSON decoding failure"))
        }
    }
}
