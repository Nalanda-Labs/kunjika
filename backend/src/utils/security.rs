use crate::state::AppState;
use itsdangerous::{default_builder, IntoTimestampSigner, TimestampSigner};
use rand::distr::Alphanumeric;
use rand::prelude::*;
use std::time::Duration;

pub fn sign_in_code(len: &usize) -> String {
    let rng = rand::rng(); // previously thread_rng()
    rng.sample_iter(&Alphanumeric)
        .take(*len)
        .map(char::from)
        .collect()
}

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

    let unsigned = match signer.unsign(&text) {
        Ok(u) => u,
        Err(_e) => return String::from("Invalid signature"),
    };
    unsigned
        .value_if_not_expired(Duration::from_secs(
            state.config.email_verification_expiry_time,
        ))
        .expect("Signature was expired")
        .to_string()
}

pub async fn extract_email(text: &str, state: &AppState) -> String {
    let signer = default_builder(state.config.secret_key.clone())
        .build()
        .into_timestamp_signer();

    let unsigned = match signer.unsign(&text) {
        Ok(u) => u,
        Err(_e) => return String::from("Invalid signature"),
    };
    unsigned.value().to_string()
}
