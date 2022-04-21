pub mod endpoints;

pub use endpoints::entity::update::address::*;
pub use endpoints::entity::update::phone::*;
pub use endpoints::entity::update::email::*;
pub use endpoints::entity::update::identity::*;
pub use endpoints::entity::request_kyc::*;
pub use endpoints::wallet::get_sila_balance::*;
pub use endpoints::entity::register::*;
pub use endpoints::entity::check_kyc::*;
pub use endpoints::entity::*;
pub use endpoints::account::link_account::*;

use reqwest;
use sha3::{ Digest, Keccak256 };
use secp256k1::{ Secp256k1, SecretKey };
use serde::{ Deserialize, Serialize };
use lazy_static::lazy_static;
use std::env;

pub struct SilaParams {
    pub gateway: String,
    pub app_handle: String,
    pub app_address: String,
    pub app_private_key: String,
}

lazy_static! {
    static ref SILA_PARAMS: SilaParams = {
        SilaParams { 
            gateway: env::var("SILA_GATEWAY").expect("SILA_GATEWAY must be set"),
            app_handle: env::var("SILA_APP_HANDLE").expect("SILA_APP_HANDLE must be set"),
            app_address: env::var("SILA_APP_ADDRESS").expect("SILA_APP_ADDRESS must be set"),
            app_private_key: env::var("SILA_APP_KEY").expect("SILA_APP_KEY must be set")
        }
    };
}

#[derive(Deserialize, Serialize)]
#[derive(PartialEq)]
pub enum Status {
    SUCCESS,
    FAILURE
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Status::SUCCESS => write!(f, "SUCCESS"),
            Status::FAILURE => write!(f, "FAILURE"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Header {
    pub reference: String,
    pub created: i64,
    pub user_handle: String,
    pub auth_handle: String,
    pub version: String,
    pub crypto: String,
}

#[derive(Deserialize, Serialize)]
pub struct HeaderMessage {
    pub header: Header,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct Signatures {
    pub usersignature: String,
    pub authsignature: String,
}

pub struct SignaturesParams {
    pub address: String,
    pub private_key: String,
    pub data: String,
}

pub async fn header_message() -> Result<HeaderMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let _url: String = format!("{}/getmessage?emptymessage=HeaderTestMessage", sila_params.gateway);

    let resp: HeaderMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

pub async fn sila_signatures(params: &SignaturesParams) -> Result<Signatures, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;
    
    let u_sig: String = sign_sila(&SignParams { message: params.data.clone(), address: params.address.clone(), private_key: params.private_key.clone() }).await?;
    let a_sig: String = sign_sila(&SignParams { message: params.data.clone(), address: sila_params.app_address.clone(), private_key: sila_params.app_private_key.clone() }).await?;

    let resp: Signatures = Signatures { 
        usersignature: u_sig,
        authsignature: a_sig
    };

    Ok(resp)
}

#[derive(Serialize)]
pub struct SignParams {
    pub message: String,
    pub address: String,
    pub private_key: String
}

pub async fn sign_sila(params: &SignParams) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let mut hasher = Keccak256::new();
    hasher.update(&params.message);
    let hash = hasher.finalize();

    let message = secp256k1::Message::from_slice(&hash).unwrap();

    let secret_key = SecretKey::from_slice(&hex::decode(&params.private_key[2..].to_string()).unwrap()).unwrap();
    let secp = Secp256k1::new();
    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

    let (id, bytes) = signature.serialize_compact();

    // https://github.com/pubkey/eth-crypto/blob/master/src/sign.js
    let recovery_id = match id.to_i32() { 1 => { 0x1c }, _ => { 0x1b }};

    let mut eth_array = [0; 65];
    eth_array[0..64].copy_from_slice(&bytes[0..64]);
    eth_array[64] = recovery_id;

    Ok(hex::encode(eth_array))
}