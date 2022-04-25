#![feature(async_closure)]

pub mod endpoints;

pub use endpoints::account::link_account::*;
pub use endpoints::entity::check_kyc::*;
pub use endpoints::entity::register::*;
pub use endpoints::entity::request_kyc::*;
pub use endpoints::entity::update::address::*;
pub use endpoints::entity::update::email::*;
pub use endpoints::entity::update::identity::*;
pub use endpoints::entity::update::phone::*;
pub use endpoints::entity::*;
pub use endpoints::wallet::get_sila_balance::*;

use eth_checksum;
use lazy_static::lazy_static;
use reqwest;
use secp256k1::{Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::convert::TryInto;
use std::env;
use std::future::Future;
use std::str::FromStr;
use web3::{types::H160, types::H256};

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
            app_private_key: env::var("SILA_APP_KEY").expect("SILA_APP_KEY must be set"),
        }
    };
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum Status {
    SUCCESS,
    FAILURE,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Status::SUCCESS => write!(f, "SUCCESS"),
            Status::FAILURE => write!(f, "FAILURE"),
        }
    }
}

#[derive(Clone)]
pub struct SignedMessageParams {
    pub sila_handle: String,
    pub ethereum_address: H160,
    pub message: String,
    pub usersignature: Option<String>,
    pub authsignature: String,
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

pub fn hash_message(message: String) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&message);
    hasher
        .finalize()
        .as_slice()
        .try_into()
        .expect("Wrong length")
}

pub fn checksum(address: &str) -> String {
    eth_checksum::checksum(address)
}

#[derive(Deserialize, Serialize)]
pub struct Signatures {
    pub usersignature: Option<String>,
    pub authsignature: String,
}

// #[derive(Copy, Clone)]
// pub struct SignData {
//     pub address: [u8;20],
//     pub data: [u8;32],
//     pub private_key: Option<[u8;32]>
// }

#[derive(Copy, Clone)]
pub struct SignData {
    pub address: [u8; 20],
    pub data: [u8; 32],
    pub private_key: Option<[u8; 32]>,
}

#[derive(Clone)]
pub struct Signature {
    pub data: String,
}

#[derive(Copy, Clone)]
pub struct Signer<F, Fut>
where
    F: FnOnce(&mut SignData) -> Fut,
    Fut: Future<Output = Signature>,
{
    pub sign_func: F,
}

impl<F, Fut> Signer<F, Fut>
where
    F: FnOnce(&mut SignData) -> Fut,
    Fut: Future<Output = Signature>,
{
    pub fn new(signer: F) -> Signer<F, Fut> {
        Signer { sign_func: signer }
    }

    pub fn sign(self, sign_data: &mut SignData) -> Fut {
        (self.sign_func)(sign_data)
    }
}

pub async fn default_sign(user_data: Option<SignData>, mut app_data: SignData) -> Signatures {
    let user_signer = Signer::new(async move |&mut x| {
        let message = secp256k1::Message::from_slice(&x.data).unwrap();

        let secret_key = SecretKey::from_slice(&x.private_key.unwrap()).unwrap();
        let secp = Secp256k1::new();
        let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

        let (id, bytes) = signature.serialize_compact();

        // https://github.com/pubkey/eth-crypto/blob/master/src/sign.js
        let recovery_id = match id.to_i32() {
            1 => 0x1c,
            _ => 0x1b,
        };

        let mut eth_array = [0; 65];
        eth_array[0..64].copy_from_slice(&bytes[0..64]);
        eth_array[64] = recovery_id;

        Signature {
            data: hex::encode(eth_array),
        }
    });

    let app_signer = Signer::new(async move |&mut x| {
        let message = secp256k1::Message::from_slice(&x.data).unwrap();

        let secret_key = SecretKey::from_slice(&x.private_key.unwrap()).unwrap();
        let secp = Secp256k1::new();
        let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

        let (id, bytes) = signature.serialize_compact();

        // https://github.com/pubkey/eth-crypto/blob/master/src/sign.js
        let recovery_id = match id.to_i32() {
            1 => 0x1c,
            _ => 0x1b,
        };

        let mut eth_array = [0; 65];
        eth_array[0..64].copy_from_slice(&bytes[0..64]);
        eth_array[64] = recovery_id;

        Signature {
            data: hex::encode(eth_array),
        }
    });

    match user_data {
        Some(mut x) => Signatures {
            usersignature: Option::from(user_signer.sign(&mut x).await.data),
            authsignature: app_signer.sign(&mut app_data).await.data,
        },
        None => Signatures {
            usersignature: Option::None,
            authsignature: app_signer.sign(&mut app_data).await.data,
        },
    }
}

// message must be pre-hashed for this data field
pub struct SignaturesParams {
    pub address: H160,
    pub private_key: Option<H256>,
    pub data: [u8; 32],
}

pub async fn header_message() -> Result<HeaderMessage, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let _url: String = format!(
        "{}/getmessage?emptymessage=HeaderTestMessage",
        sila_params.gateway
    );
    let resp: HeaderMessage = reqwest::get(&_url.to_owned()).await?.json().await?;

    Ok(resp)
}

// debatable whether this is useful - in the default_signer maybe, but otherwise probable not
pub async fn sila_app_signature(
    message_hash: [u8; 32],
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    Ok(sign_sila(&SignParams {
        message_hash: message_hash.clone(),
        address: sila_params.app_address.clone(),
        private_key: sila_params.app_private_key.clone(),
    })
    .await?)
}

pub async fn sila_signatures(
    params: &SignaturesParams,
) -> Result<Signatures, Box<dyn std::error::Error + Sync + Send>> {
    let sila_params = &*crate::SILA_PARAMS;

    let pvk = format!("{:#x}", params.private_key.clone().unwrap());

    let u_sig: String = sign_sila(&SignParams {
        message_hash: params.data.clone(),
        address: params.address.clone().to_string(),
        private_key: pvk.to_string(),
    })
    .await?;

    let a_sig: String = sign_sila(&SignParams {
        message_hash: params.data.clone(),
        address: sila_params.app_address.clone(),
        private_key: sila_params.app_private_key.clone(),
    })
    .await?;

    let resp: Signatures = Signatures {
        usersignature: Option::from(u_sig),
        authsignature: a_sig,
    };

    Ok(resp)
}

pub struct SignParams {
    pub message_hash: [u8; 32],
    pub address: String,
    pub private_key: String,
}

pub async fn sign_sila(
    params: &SignParams,
) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
    let message = secp256k1::Message::from_slice(&params.message_hash).unwrap();

    let secret_key =
        SecretKey::from_slice(&hex::decode(&params.private_key[2..].to_string()).unwrap()).unwrap();
    let secp = Secp256k1::new();
    let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);

    let (id, bytes) = signature.serialize_compact();

    // https://github.com/pubkey/eth-crypto/blob/master/src/sign.js
    let recovery_id = match id.to_i32() {
        1 => 0x1c,
        _ => 0x1b,
    };

    let mut eth_array = [0; 65];
    eth_array[0..64].copy_from_slice(&bytes[0..64]);
    eth_array[64] = recovery_id;

    Ok(hex::encode(eth_array))
}
