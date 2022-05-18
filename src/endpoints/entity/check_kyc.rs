use crate::endpoints::entity::*;

#[derive(Clone)]
pub struct CheckKycMessageParams {
    pub sila_handle: String,
}

impl From<CheckKycMessageParams> for HeaderMessage {
    fn from(params: CheckKycMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header: HeaderMessage = header_message();
        header.header.user_handle = Option::from(params.sila_handle.clone());
        header.header.auth_handle = sila_params.app_handle.clone();

        header
    }
}

pub async fn check_kyc(
    params: &SignedMessageParams,
) -> Result<CheckResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/check_kyc", sila_params.gateway);

    let h: HeaderMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(&_url.to_owned())
        .header("usersignature", &params.usersignature.clone().unwrap())
        .header("authsignature", &params.authsignature)
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<CheckResponse, serde_json::Error> = serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general check_kyc error | text: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}
