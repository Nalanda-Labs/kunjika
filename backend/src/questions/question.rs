use time;

#[cfg(any(feature = "postgres"))]
type SqlID = i64;

pub type SqlDateTime = time::OffsetDateTime;

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
    pub updated_at: SqlDateTime,
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
    pub updated_by_id: SqlID,
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
    pub slug: String,
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
    pub image_url: String,
    pub vote_by_current_user: i64,
    pub cat: i64,
    pub uat: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionsReq {
    pub updated_at: String,
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
    pub uat: i64,
    pub cat: i64,
    pub answer_accepted: bool,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QR1 {
    pub id: i64,
    pub title: Option<String>,
    pub visible: bool,
    pub votes: i64,
    pub views: i64,
    pub slug: Option<String>,
    pub posted_by_id: i64,
    pub created_at: SqlDateTime,
    pub updated_at: SqlDateTime,
    pub username: String,
    pub image_url: String,
    pub uid: i64,
    pub tags: Option<Vec<String>>,
    pub answer_accepted: bool,
    pub answer_count: i64,
    pub tag_id: Option<Vec<i64>>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct QuestionsResponse {
    pub questions: Vec<QR>,
}

#[derive(Deserialize)]
pub struct AnswersReq {
    pub cat: String,
}

#[derive(Deserialize)]
pub struct AnswersQuery {
    pub limit: i64,
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
    pub answer_accepted: bool,
    pub reply_to_id: Option<i64>,
    pub rusername: String,
    pub rimage_url: String,
    pub vote_by_current_user: i64,
    pub cat: i64,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AR1 {
    pub answer_id: i64,
    pub question_id: i64,
    pub title: Option<String>,
    pub visible: bool,
    pub votes: i64,
    pub slug: Option<String>,
    pub updated_at: SqlDateTime,
    pub answer_accepted: bool,
    pub tag_id: Option<Vec<i64>>,
    pub tags: Option<Vec<String>>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AR2 {
    pub question_id: i64,
    pub title: String,
    pub answer_id: i64,
    pub visible: bool,
    pub votes: i64,
    pub slug: String,
    pub updated_at: SqlDateTime,
    pub answer_accepted: bool,
    pub uat: i64,
    pub tid: String,
    pub tags: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AnswersResponse1 {
    pub questions: Vec<AR2>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct AnswersResponse {
    pub questions: Vec<AR>,
}

#[derive(Deserialize, Validate)]
pub struct AnswerReq {
    pub id: String,
    #[validate(length(min = 20, max = 100000))]
    pub value: String,
    pub reply_to: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostResponse {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct EditRequest {
    #[validate(length(min = 6, max = 256))]
    pub title: Option<String>,
    #[validate(length(min = 20, max = 100000))]
    pub description: String,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserQuestionsReq {
    pub uat: String,
    pub direction: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserAnswersReq {
    pub uat: String,
    pub direction: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserBookmarksReq {
    pub uat: String,
    pub bookmarks_per_page: i64,
    pub direction: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookmarkIDs {
    pub qid: i64,
    pub aid: i64,
}
