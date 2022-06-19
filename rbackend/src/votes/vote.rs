#[derive(Serialize, Deserialize, Debug)]
pub struct VoteRequest {
    pub vote: i64,
    pub id: i64
}