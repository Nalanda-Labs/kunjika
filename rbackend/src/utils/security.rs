use std::time::Duration;
use crate::state::AppState;
use itsdangerous::{default_builder, IntoTimestampSigner, Signer, TimestampSigner};

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
    unsigned
        .value_if_not_expired(Duration::from_secs(state.config.email_verification_expiry_time))
        .expect("Signature was expired").to_string()
}
