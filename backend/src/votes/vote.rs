#[derive(Serialize, Deserialize, Debug)]
pub struct VoteRequest {
    pub vote: i64,
    pub id: i64,
}

#[derive(PartialEq)]
pub enum VoteEnum {
    ReceivingUserNotFound,
    VoteYourOwnPost,
    VoteOnce,
    VoteSuccessful,
}

pub struct VoteResult {
    pub e: VoteEnum,
    pub vote_by_user: i64,
}
