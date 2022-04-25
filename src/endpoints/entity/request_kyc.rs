use serde::{Deserialize, Serialize};
use log::error;
use web3::{
    types::H160,
    types::H256
};

use crate::endpoints::entity::*;
use crate::hash_message;

pub struct RequestKycParams {
    pub customer_sila_handle: String,
    pub customer_eth_address: H160,
    pub customer_private_key: Option<H256>
}

#[derive(Serialize, Deserialize)]
pub struct RequestKycResponse {
    pub message: String,
    pub reference: String,
    pub status: Status,
}

pub async fn request_kyc(params: &RequestKycParams) -> Result<RequestKycResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/request_kyc", sila_params.gateway);

    let mut message: HeaderMessage = header_message().await?;
    message.header.user_handle = params.customer_sila_handle.clone();
    message.header.auth_handle = sila_params.app_handle.clone();

    let signatures: Signatures = sila_signatures(&SignaturesParams {
        address: params.customer_eth_address.clone(),
        private_key: params.customer_private_key.clone(),
        data: hash_message(serde_json::to_string(&message)?)
    }).await?;

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", signatures.usersignature.unwrap())
        .header("authsignature", signatures.authsignature)
        .json(&message)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<RequestKycResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general request_kyc error | text: {}", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}