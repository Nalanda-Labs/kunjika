use crate::state::AppState;
use itsdangerous::{default_builder, Signer, TimestampSigner, IntoTimestampSigner};

pub async fn sign(text: &str, state: &AppState) -> String{
    let signer = default_builder(state.config.secret_key.clone())
                .build()
                .into_timestamp_signer();

    signer.sign(text)
}