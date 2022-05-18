# Sila Banking API for Rust

This crate is intended to reduce the effort necessary by Rust developers to integrate their applications with the [Sila Banking API](https://docs.silamoney.com).

This is an early work-in-progress. Work is currently focused on establishing the framework for interacting with the API and beginning to build modules specific to functionality provided by the Sila endpoints.

## Example

A call to Sila's `check_handle` endpoint can be accomplished in the following way.

All of the `silamoney::` functions are `async` so you'll need to use an appropriate runtime like `tokio`.

```rust
#[tokio::main]
async fn main() {
```

This section defines the addresses used in the call to Sila's API.

```rust
    // you'll need your registered application's address and key; we define those here with these
    // variable names to streamline the KeyParams constructor further down

    let address = "0x...".to_string();
    let private_key = Option::from("0x...".to_string());
```

These `fn` references in `silamoney::*` build the JSON object that will be sent to Sila based on Sila's API expectations.

```rust
    let message = serde_json::to_string(&HeaderMessage::from(CheckHandleMessageParams {
        sila_handle: "handle-to-check".to_string(),
    })).unwrap();
```

This function prepares the data to be signed by whatever `Signer` you're using. We'll use `default_sign`.

Because `check_handle` does not require a `usersignature`, `Option::None` is provided to the function to skip the creation of that Signature. For `KeyParams` we only need to specify the variable names previously defined because they match the expectations of the `struct`.

```rust
    let sdp = SignDataPair::from(SignDataParams {
        message: message.clone(),
        user_params: Option::None,
        app_params: KeyParams { address, private_key },
    });
```

Provisions exist in the `silamoney` crate to specify a custom signer. This `default_sign` function builds a Signer that requires the application to have direct access to customer private keys.
    

```rust
    let signatures = default_sign(sdp.user, sdp.app).await;
```

This struct is is in `silamoney::SignedMessageParams`. It is used to send the request to the `check_handle` endpoint.

```rust
    let smp = SignedMessageParams {
        ethereum_address: address.clone(),
        sila_handle: handle.to_string(),
        message: message.clone(),
        usersignature: Option::None,
        authsignature: signatures.authsignature,
    };
```

The `check_handle` function is in `silamoney::*` and executes the request to the Sila API and waits for a response.

```rust
    let response = check_handle(&smp).await;

    println!("Response: {:?}", serde_json::to_string(&response.unwrap()));
}
```

## Authentication

The Sila API uses an authentication mechanism that leverages key generation and message signing standards established by the Ethereum project. This crate has a number of structs and functions to enable you to use user private keys to sign messages and use the API in that way.

You can also choose to define your own `Signer` and provide it a closure to produce the signatures needed to execute that authentication. For example, you might have a separate service that provides a `/sign` endpoint where you can provide a user address and a message to be signed and have that service handle the sensitive operations necessary to produce that signature. Here is an example of how that could be accomplished

```rust
#[derive(Deserialize, Serialize)]
struct CustomResponse {
    signature: String
}

#[derive(Deserialize, Serialize)]
struct SilaSignServiceParams {
    address: String,
    message: String
}

let app_address = H160::from_str("0x...").unwrap();
        
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
```

In that way you can establish some sensible architectural security boundaries around your services.
