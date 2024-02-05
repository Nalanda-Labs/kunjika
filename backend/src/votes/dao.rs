use super::vote::*;
use crate::state::AppStateRaw;

#[async_trait]
pub trait IVote: std::ops::Deref<Target = AppStateRaw> {
    async fn handle_vote(&self, data: &VoteRequest) -> sqlx::Result<bool>;
}


#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IVote for &AppStateRaw {
    async fn handle_vote(&self, data: &VoteRequest) -> sqlx::Result<bool> {
        Ok(true)
    }
}