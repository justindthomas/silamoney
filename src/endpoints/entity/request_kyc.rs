use serde::{Deserialize, Serialize};
use log::error;
use web3::{
    types::H160,
    types::H256
};

use crate::endpoints::entity::*;

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

pub struct RequestKycMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
}

pub async fn request_kyc_message(
    params: &RequestKycMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let mut header: HeaderMessage = header_message().await?;
    header.header.user_handle = params.sila_handle.clone();
    header.header.auth_handle = sila_params.app_handle.clone();

    Ok(serde_json::to_string(&header)?)
}

pub async fn request_kyc(params: &SignedMessageParams) -> Result<RequestKycResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/request_kyc", sila_params.gateway);

    let h: HeaderMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
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