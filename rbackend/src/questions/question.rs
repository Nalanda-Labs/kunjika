#[cfg(any(feature = "postgres"))]
type SqlID = i64;

type SqlDateTime = chrono::NaiveDateTime;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: SqlID,
    title: String,
    description: String,
    visible: String,
    votes: i64,
    slug: String,
    views: i64,
    answer_accepted: bool,
    answer_count: i64,
    op_id: SqlID,
    posted_by_id: SqlID,
    reply_to_id: SqlID,
    updated_by_id: SqlID,
    pub create_dt: SqlDateTime,
    pub update_dt: SqlDateTime
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct DbQuestion {
    #[validate(length(min = 6, max = 256))]
    pub title: String,
    #[validate(length(min = 20, max = 100000))]
    pub description: String,
    pub tag_list: Vec<String>,
    pub slug: String,
    pub op_id: SqlID,
    pub posted_by_id: SqlID,
    pub updated_by_id: SqlID
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct QuestionRequest {
    #[validate(length(min = 6, max = 256))]
    pub title: String,
    #[validate(length(min = 20, max = 100000))]
    pub description: String,
    pub tag_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AskResponse {
    pub id: String,
    pub slug: String
}