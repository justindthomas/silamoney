# Sila Banking API for Rust

This crate is intended to reduce the effort necessary by Rust developers to integrate their applications with the [Sila Banking API](https://docs.silamoney.com).

This is an early work-in-progress. Work is currently focused on establishing the framework for interacting with the API and beginning to build modules specific to functionality provided by the Sila endpoints.

## Example

A call to Sila's `check_kyc` endpoint can be accomplished in the following way.

~~~
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::stdin;
use std::str::FromStr;
use web3::types::{H160, H256};

use silamoney::{
    check_handle, check_handle_message, default_sign, hash_message, CheckHandleMessageParams,
    SignData, SignedMessageParams,
};

#[derive(Deserialize, Serialize)]
struct SilaSignServiceParams {
    address: String,
    message: String,
    hashed: Option<bool>,
}

#[tokio::main]
async fn main() {

    // This section will capture a handle to check for existence from stdin
    println!("Handle:");
    let mut handle_line = String::new();
    stdin().read_line(&mut handle_line).unwrap();

    let handle = handle_line.trim_end();

    // This section defines the addresses used in the call to Sila's API
    // address: the user's ethereum address
    let address = H160::from_str("0x...")
        .expect("couldn't parse address");

    // app_address: the address of your application as registered with Sila
    let app_address = H160::from_str("0x...")
        .expect("failed to parse app_address");
    
    // app_private_key: the private key associated with your application's registered address
    let app_private_key = H256::from_str("0x...)
        .expect("failed to parse app_private_key");
    
    // this struct is in silamoney::CheckHandleMessageParams
    let check_params = CheckHandleMessageParams {
        sila_handle: handle.to_string(),
        ethereum_address: address,
    };

    // this fn in silamoney::* builds the JSON object that will be sent to Sila based on Sila's
    // API expectations
    let message = check_handle_message(&check_params)
        .await
        .expect("check_handle_message failed");

    // this is a call to silamoney::hash_message that begins to set up the structure necessary
    // to authenticate the request to the Sila API
    let hash = hash_message(message.clone());

    // this struct is in silamoney::SignData and is used by the Signer function to sign the
    // Sila API request for authentication against your registered application
    let app_data = SignData {
        address: *app_address.as_fixed_bytes(),
        data: hash,
        private_key: Option::from(*app_private_key.as_fixed_bytes()),
    };

    // provisions exist in the silamoney crate to specify a custom signer
    // this default_sign function builds Signer that requires the application to have direct access
    // to customer private keys
    
    // because check_handle does not require a usersignature, Option::None is provided to the function
    // to skip the creation of that Signature
    let signatures = default_sign(Option::None, app_data).await;

    // this struct is is in silamoney::SignedMessageParams
    // it is used to finally send the request to the check_handle endpoint
    let smp = SignedMessageParams {
        ethereum_address: address.clone(),
        sila_handle: handle.to_string(),
        message: message.clone(),
        usersignature: Option::None,
        authsignature: signatures.authsignature,
    };

    // the check_handle function is in silamoney::* and executes the request to the Sila API and waits
    // for a response
    let response = check_handle(&smp).await;

    println!("Response: {:?}", serde_json::to_string(&response.unwrap()));
}
~~~

## Authentication

The Sila API uses an authentication mechanism that leverages key generation and message signing standards established by the Ethereum project. This crate has a number of structs and functions to enable you to use user private keys to sign messages and use the API in that way.

You can also choose to define your own `Signer` and provide it a closure to produce the signatures needed to execute that authentication. For example, you might have a separate service that provides a `/sign` endpoint where you can provide a user address and a message to be signed and have that service handle the sensitive operations necessary to produce that signature. Here is an example of how that could be accomplished

~~~
#[derive(Deserialize, Serialize)]
struct CustomResponse {
    signature: String
}

#[derive(Deserialize, Serialize)]
struct SilaSignServiceParams {
    address: String,
    message: String
}

let app_address = H160::from_str("0x...")
    .expect("failed to parse app_address");
        
let mut app_data = SignData {
    address: *app_address.as_fixed_bytes(),
    data: hash,
    private_key: Option::None,
};

let signer = Signer::new(async move |&mut x| { 
    let url = format!("{}/sila/sign", env::var("SIGN_SVC_URL").expect("SIGN_SVC_URL must be set"));
          
    let address = &format!("{:#x}", H160::from_slice(&x.address));
    let message = hex::encode(&x.data);
    let params = SilaSignServiceParams { address, message };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&params)
        .send()
        .await
        .unwrap()
        .json::<CustomResponse>()
        .await
        .unwrap();
        
    Signature { data: resp.text.unwrap() } 
});

let signature = signer.sign(&mut app_data).await;
~~~

In that way you can establish some sensible architectural security boundaries around your services.
