use log::error;
use serde::{Deserialize, Serialize};
use web3::types::H160;

use crate::endpoints::entity::{Address, Contact, CryptoEntry, Entity, Identity, IdentityAlias};
use crate::{header_message, Header, HeaderMessage, SignedMessageParams, Status};

#[derive(Deserialize, Serialize)]
pub struct RegisterMessage {
    pub header: Header,
    pub message: String,
    pub address: Address,
    pub identity: Identity,
    pub contact: Contact,
    pub crypto_entry: CryptoEntry,
    pub entity: Entity,
}

#[derive(Clone)]
pub struct RegisterMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub birthdate: String,
    pub first_name: String,
    pub last_name: String,
    pub street_address_1: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub phone: String,
    pub email: String,
    pub ssn: String,
}

impl From<RegisterMessageParams> for RegisterMessage {
    fn from(params: RegisterMessageParams) -> Self {
        let sila_params = &*crate::SILA_PARAMS;

        let mut header_message: HeaderMessage = header_message();
        header_message.header.user_handle = Option::from(params.sila_handle.clone());
        header_message.header.auth_handle = sila_params.app_handle.clone();

        RegisterMessage {
            header: header_message.header,
            entity: Entity {
                relationship: Option::from("user".to_string()),
                entity_name: "default".to_string(),
                first_name: params.first_name.clone(),
                last_name: params.last_name.clone(),
                birthdate: params.birthdate.clone(),
            },
            address: Address {
                address_alias: Option::from("default".to_string()),
                street_address_1: Option::from(params.street_address_1.clone()),
                city: Option::from(params.city.clone()),
                state: Option::from(params.state.clone()),
                postal_code: Option::from(params.postal_code.clone()),
                country: Option::from("US".to_string()),
                ..Default::default()
            },
            identity: Identity {
                identity_alias: IdentityAlias::Ssn,
                identity_value: params.ssn.clone(),
            },
            contact: Contact {
                contact_alias: "default".to_string(),
                phone: params.phone.clone(),
                email: params.email.clone(),
            },
            crypto_entry: CryptoEntry {
                crypto_alias: "default".to_string(),
                crypto_status: Option::None,
                crypto_address: format!("{:#x}", params.ethereum_address.clone()),
                crypto_code: "ETH".to_string(),
            },
            message: "entity_msg".to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub message: String,
    pub reference: String,
    pub status: Status,
}

pub async fn register(
    params: &SignedMessageParams,
) -> Result<RegisterResponse, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    let _url: String = format!("{}/register", sila_params.gateway);

    let h: RegisterMessage = serde_json::from_str(&params.message.clone()).unwrap();

    let client = reqwest::Client::new();
    let resp = client
        .post(&_url.to_owned())
        .header("usersignature", params.usersignature.clone().unwrap())
        .header("authsignature", params.authsignature.clone())
        .json(&h)
        .send()
        .await?;

    let response_text = resp.text().await?;
    let response: Result<RegisterResponse, serde_json::Error> =
        serde_json::from_str(&response_text);

    match response {
        Ok(x) if x.status == Status::FAILURE => {
            error!("general register error | text: {}", response_text);
            Ok(x)
        }
        Ok(x) => Ok(x),
        Err(e) => {
            error!("decoding error | text: {}", response_text);
            Err(Box::from(e))
        }
    }
}
