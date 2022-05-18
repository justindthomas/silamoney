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
pub use endpoints::transaction::cancel_transaction::*;
pub use endpoints::transaction::get_transactions::*;
pub use endpoints::transaction::issue_sila::*;
pub use endpoints::transaction::redeem_sila::*;
pub use endpoints::transaction::transfer_sila::*;
pub use endpoints::wallet::get_sila_balance::*;
use std::str::FromStr;

use eth_checksum;
use lazy_static::lazy_static;
use secp256k1::{Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::convert::TryInto;
use std::env;
use std::future::Future;
use std::time::SystemTime;
use uuid::Uuid;
use web3::{types::H160, types::H256};

#[derive(Clone)]
pub struct SilaParams {
    pub gateway: String,
    pub app_handle: String,
    pub app_address: String,
    pub app_private_key: Option<String>,
}

lazy_static! {
    static ref SILA_PARAMS: SilaParams = {
        SilaParams {
            gateway: env::var("SILA_GATEWAY").expect("SILA_GATEWAY must be set"),
            app_handle: env::var("SILA_APP_HANDLE").expect("SILA_APP_HANDLE must be set"),
            app_address: env::var("SILA_APP_ADDRESS").expect("SILA_APP_ADDRESS must be set"),
            app_private_key: Option::from(
                env::var("SILA_APP_KEY").expect("SILA_APP_KEY must be set"),
            ),
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
    pub sila_handle: Option<String>,
    pub message: String,
    pub usersignature: Option<String>,
    pub authsignature: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Header {
    pub reference: String,
    pub created: u64,
    pub user_handle: Option<String>,
    pub auth_handle: String,
    pub version: String,
    pub crypto: String,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            reference: Uuid::new_v4().to_string(),
            created: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("could not calculate current time")
                .as_secs(),
            user_handle: Option::None,
            auth_handle: String::new(),
            version: "0.2".to_string(),
            crypto: "ETH".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HeaderMessage {
    pub header: Header,
    pub message: String,
}

fn hash_message(message: String) -> [u8; 32] {
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

#[derive(Deserialize, Serialize, Clone)]
pub struct KeyParams {
    pub address: String,
    pub private_key: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct SignDataParams {
    pub message: String,
    pub user_params: Option<KeyParams>,
    pub app_params: KeyParams,
}

#[derive(Copy, Clone)]
pub struct SignData {
    pub address: [u8; 20],
    pub message_hash: [u8; 32],
    pub private_key: Option<[u8; 32]>,
}

#[derive(Copy, Clone)]
pub struct SignDataPair {
    pub user: Option<SignData>,
    pub app: SignData,
}

impl From<SignDataParams> for SignDataPair {
    fn from(params: SignDataParams) -> Self {
        let hash = hash_message(params.message);
        let mut user = Option::None;

        if params.user_params.is_some() {
            let up = params.user_params.clone().unwrap();

            let mut user_private_key = Option::None;
        
            if up.private_key.is_some() {
                user_private_key = Option::from(*H256::from_str(&up.private_key.unwrap())
                .unwrap()
                .as_fixed_bytes());
            }

            user = Option::from(SignData {
                address: *H160::from_str(&params.user_params.clone().unwrap().address)
                    .unwrap()
                    .as_fixed_bytes(),
                message_hash: hash,
                private_key: user_private_key,
            })
        };

        let mut app_private_key = Option::None;

        if params.app_params.private_key.is_some() {
            app_private_key = Option::from(*H256::from_str(&params.app_params.private_key.unwrap())
            .unwrap()
            .as_fixed_bytes());
        }

        SignDataPair {
            user,
            app: SignData {
                address: *H160::from_str(&params.app_params.address)
                    .unwrap()
                    .as_fixed_bytes(),
                message_hash: hash,
                private_key: app_private_key,
            },
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Signatures {
    pub usersignature: Option<String>,
    pub authsignature: String,
}

#[derive(Clone)]
pub struct Signature {
    pub data: String,
}

#[derive(Clone)]
pub struct Signer<F, Fut>
where
    F: Fn(SignData) -> Fut,
    Fut: Future<Output = Signature>,
{
    pub sign_func: F,
}

impl<F, Fut> Signer<F, Fut>
where
    F: Fn(SignData) -> Fut,
    Fut: Future<Output = Signature>,
{
    pub fn new(signer: F) -> Signer<F, Fut> {
        Signer { sign_func: signer }
    }

    pub fn sign(self, sign_data: SignData) -> Fut {
        (self.sign_func)(sign_data)
    }
}

pub async fn default_sign(user_data: Option<SignData>, app_data: SignData) -> Signatures {
    let closure = async move |x: SignData| {
        let message = secp256k1::Message::from_slice(&x.message_hash).unwrap();

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
    };

    let user_signer = Signer::new(closure);
    let app_signer = Signer::new(closure);

    match user_data {
        Some(x) => Signatures {
            usersignature: Option::from(user_signer.sign(x).await.data),
            authsignature: app_signer.sign(app_data).await.data,
        },
        None => Signatures {
            usersignature: Option::None,
            authsignature: app_signer.sign(app_data).await.data,
        },
    }
}

pub fn header_message() -> HeaderMessage {
    HeaderMessage {
        header: Header {
            ..Default::default()
        },
        message: "header_msg".to_string(),
    }
}

pub struct SignParams {
    pub message_hash: [u8; 32],
    pub address: String,
    pub private_key: String,
}
