use serde_json;
use std::io::stdin;

use silamoney::*;

#[tokio::main]
async fn main() {
    let address = "0x...".to_string();
    let private_key = Option::from("0x...".to_string());

    println!("Handle:");
    let mut handle_line = String::new();
    stdin().read_line(&mut handle_line).unwrap();

    let message = serde_json::to_string(&HeaderMessage::from(CheckHandleMessageParams {
        sila_handle: handle_line.trim_end().to_string(),
    })).unwrap();

    let sdp = SignDataPair::from(SignDataParams {
        message: message.clone(),
        user_params: Option::None,
        app_params: KeyParams { address, private_key },
    });

    let signatures = default_sign(sdp.user, sdp.app).await;

    let smp = SignedMessageParams {
        ethereum_address: Option::None,
        sila_handle: Option::None,
        message: message.clone(),
        usersignature: Option::None,
        authsignature: signatures.authsignature,
    };

    let response = check_handle(&smp).await;

    println!("Response: {:?}", serde_json::to_string(&response.unwrap()));
}