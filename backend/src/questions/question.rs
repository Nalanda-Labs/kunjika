use time;
use std::collections::HashSet;

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
    pub image_url: Option<String>,
    pub vote_by_current_user: i64,
    pub cat: i64,
    pub uat: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionsReq {
    pub uat: String,
    pub questions_per_page: i64,
    pub direction: Option<String>,
}

/// Full-text search over questions.
///
/// `search` is fed to `plainto_tsquery('english', …)` so a plain phrase such as
/// "rust borrow checker" is tokenised into an AND of its words. Only rows that
/// are questions (`title IS NOT NULL AND title <> ''`) are matched, which is
/// exactly the predicate backing the `posts_search_idx` GIN index created in
/// `20261009120000_posts_search_idx.sql`.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchReq {
    pub search: String,
    pub questions_per_page: i64,
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
    pub count: i64
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
    // Both come from `left join users u1 on t.reply_to_id = u1.id`, so they
    // are NULL for every answer that is not a reply. Plain `String` made the
    // whole 500 with "decoding column 12: unexpected null" as soon as the
    // result set contained a single ordinary (non-reply) answer.
    pub rusername: Option<String>,
    pub rimage_url: Option<String>,
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

/// Accept a JSON integer or a numeric string.
///
/// The SvelteKit client is inconsistent about these ids: the question id comes
/// from the route (a string), while `reply_to` is copied straight from
/// `response.data.posted_by_id` (a JSON number), and is `undefined` for a
/// top-level answer - which serde renders as a missing field. Accepting both
/// shapes keeps `AnswerReq` deserialisable instead of 400-ing.
fn de_i64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Num {
        Int(i64),
        Str(String),
    }
    match <Num as serde::Deserialize>::deserialize(d)? {
        Num::Int(i) => Ok(i),
        Num::Str(s) => s.trim().parse::<i64>().map_err(serde::de::Error::custom),
    }
}

/// Same as [`de_i64`] but also tolerates `null`, a missing field, and `""`.
///
/// An absent `reply_to` means "top-level answer" and maps to a NULL
/// `posts.reply_to_id`. It must NOT be parsed unconditionally: the previous
/// `answer.reply_to.parse::<i64>().unwrap()` panicked with `ParseIntError
/// { kind: Empty }` and aborted the connection without a status line.
fn de_opt_i64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Num {
        Int(i64),
        Str(String),
    }
    match <Option<Num> as serde::Deserialize>::deserialize(d)? {
        None => Ok(None),
        Some(Num::Int(i)) => Ok(Some(i)),
        Some(Num::Str(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t.parse::<i64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
    }
}

#[derive(Deserialize, Validate, Debug)]
pub struct AnswerReq {
    /// Id of the question being answered.
    #[serde(deserialize_with = "de_i64")]
    pub id: i64,
    #[validate(length(min = 20, max = 100000))]
    pub value: String,
    /// The user being replied to; `None` for a top-level answer.
    #[serde(default, deserialize_with = "de_opt_i64")]
    pub reply_to: Option<i64>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct FindParticipants(pub HashSet<String>, pub String);
