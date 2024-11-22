use crate::state::AppState;
use itsdangerous::{default_builder, IntoTimestampSigner, TimestampSigner};
use std::time::Duration;

pub async fn sign(text: &str, state: &AppState) -> String {
    let signer = default_builder(state.config.secret_key.clone())
        .build()
        .into_timestamp_signer();

    signer.sign(text)
}

pub async fn check_signature(text: &str, state: &AppState) -> String {
    let signer = default_builder(state.config.secret_key.clone())
        .build()
        .into_timestamp_signer();

    let unsigned = signer.unsign(&text).expect("Signature was not valid");
    unsigned.value().to_string()
}
