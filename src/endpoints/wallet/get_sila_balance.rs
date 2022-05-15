use serde::{Deserialize, Serialize};
use log::error;

#[derive(Serialize)]
pub struct RequestSilaBalanceParams {
    pub blockchain_address: String
}

#[derive(Deserialize)]
pub struct SilaBalanceResponse {
    pub success: bool,
    pub status: String,
    pub response_time_ms: String,
    pub address: String,
    pub sila_balance: i32,
    pub reference: String
}

pub async fn get_sila_balance(params: &RequestSilaBalanceParams) -> Result<SilaBalanceResponse, Box<dyn std::error::Error>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/get_sila_balance", sila_params.gateway);

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .json(&params)
        .send()
        .await?;
    
    let response_text = resp.text().await?;
    let response : Result<SilaBalanceResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.success != true => {
            error!("get_sila_balance error: {}", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("get_sila_balance response error: {}", response_text);
            Err(Box::from(e))
        }
    }
}