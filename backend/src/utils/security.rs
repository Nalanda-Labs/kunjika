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

/// Why a signed token was rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum SignatureError {
    /// Not produced by us, or tampered with.
    Invalid,
    /// Genuine but older than `email_verification_expiry_time`.
    Expired,
}

/// The signed value (an email) if the token is genuine and unexpired.
pub async fn check_signature(text: &str, state: &AppState) -> Result<String, SignatureError> {
    let signer = default_builder(state.config.secret_key.clone())
        .build()
        .into_timestamp_signer();

    let unsigned = signer.unsign(text).map_err(|_| SignatureError::Invalid)?;
    unsigned
        .value_if_not_expired(Duration::from_secs(
            state.config.email_verification_expiry_time,
        ))
        .map(|v| v.to_string())
        .map_err(|_| SignatureError::Expired)
}

/// The signed value (an email) if the token is genuine, ignoring its age.
pub async fn extract_email(text: &str, state: &AppState) -> Result<String, SignatureError> {
    let signer = default_builder(state.config.secret_key.clone())
        .build()
        .into_timestamp_signer();

    signer
        .unsign(text)
        .map(|u| u.value().to_string())
        .map_err(|_| SignatureError::Invalid)
}
