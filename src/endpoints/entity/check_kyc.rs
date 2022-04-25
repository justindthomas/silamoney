use crate::endpoints::entity::*;
use crate::hash_message;

pub struct CheckKycMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
}

pub async fn check_kyc_message(
    params: &CheckKycMessageParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!("{}/check_kyc", sila_params.gateway);

    let mut header: HeaderMessage = header_message().await?;
    header.header.user_handle = params.sila_handle.clone();
    header.header.auth_handle = sila_params.app_handle.clone();

    Ok(serde_json::to_string(&header)?)
}

pub async fn check_kyc(params: &SignedMessageParams) ->  Result<CheckResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/check_kyc", sila_params.gateway);
    
    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&params.message)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response : Result<CheckResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general check_kyc error | text: {}", response_text);
            Ok(x)
        },
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}