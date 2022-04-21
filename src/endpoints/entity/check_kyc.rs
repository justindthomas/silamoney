use crate::endpoints::entity::*;

pub async fn check_kyc(params: &CheckParams) -> Result<CheckResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/check_kyc", sila_params.gateway);

    let mut header: HeaderMessage = header_message().await?;
    header.header.user_handle = params.customer_sila_handle.clone();
    header.header.auth_handle = sila_params.app_handle.clone();

    let signatures: Signatures = sila_signatures(&SignaturesParams {
        address: params.customer_eth_address.clone(), 
        private_key: params.customer_private_key.clone(),
        data: serde_json::to_string(&header)? }).await?;

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", signatures.usersignature)
        .header("authsignature", signatures.authsignature)
        .json(&header)
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