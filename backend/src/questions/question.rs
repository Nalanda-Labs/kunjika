use chrono::{Utc, DateTime};

#[cfg(any(feature = "postgres"))]
type SqlID = i64;

type SqlDateTime = DateTime<Utc>;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Question {
    pub id: SqlID,
    pub title: String,
    pub description: String,
    pub visible: bool,
    pub votes: i64,
    pub slug: String,
    pub views: i64,
    pub answer_accepted: bool,
    pub answer_count: i64,
    pub op_id: SqlID,
    pub posted_by_id: SqlID,
    pub reply_to_id: SqlID,
    pub updated_by_id: SqlID,
    pub created_at: SqlDateTime,
    pub updated_at: SqlDateTime
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct DbQuestion {
    #[validate(length(min = 6, max = 256))]
    pub title: String,
    #[validate(length(min = 20, max = 100000))]
    pub description: String,
    pub tag_list: Vec<String>,
    pub slug: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub visible: bool,
    pub votes: i64,
    pub views: i64,
    pub op_id: String,
    pub posted_by_id: String,
    pub updated_by_id: String,
    pub created_at: SqlDateTime,
    pub updated_at: SqlDateTime,
    pub tags: Vec<String>,
    pub username: String,
    pub image_url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionsReq {
    pub updated_at: String
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QR {
    pub id: String,
    pub title: String,
    pub visible: bool,
    pub votes: i64,
    pub views: i64,
    pub slug: String,
    pub posted_by_id: String,
    pub created_at: SqlDateTime,
    pub updated_at: SqlDateTime,
    pub username: String,
    pub image_url: String,
    pub uid: String,
    pub tid: String,
    pub tags: String,
    pub answers: i64,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QuestionsResponse {
    pub questions: Vec<QR>
}

#[derive(Deserialize)]
pub struct AnswersQuery {
    pub time: chrono::DateTime<Utc>,
    pub limit: i64
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AR {
    pub question_id: String,
    pub description: String,
    pub visible: bool,
    pub votes: i64,
    pub posted_by_id: String,
    pub created_at: SqlDateTime,
    pub updated_at: SqlDateTime,
    pub username: String,
    pub image_url: String,
    pub answer_accepted: bool
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AnswersResponse {
    pub questions: Vec<AR>
}

#[derive(Deserialize)]
pub struct AnswerReq {
    pub id: String,
    pub value: String,
    pub reply_to: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostResponse {
    pub title: String,
    pub description: String,
    pub tags: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct EditRequest {
    #[validate(length(min = 6, max = 256))]
    pub title: Option<String>,
    #[validate(length(min = 20, max = 100000))]
    pub description: String,
    pub tag_list: Option<Vec<String>>,
}
