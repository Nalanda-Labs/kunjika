use super::dao::IQuestion;
use super::question::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::{AppState, AppStateRaw};
use crate::uploads::save::save_upload;
use crate::utils::request::{
    db_error, optional_user_id, parse_cursor, parse_id, parse_keyset_cursor,
};
use crate::utils::slug::create_slug;

use lettre::{AsyncTransport, Message};
use ntex::web::{get, post, HttpRequest, HttpResponse, Responder};
use ntex_multipart::Multipart;
use serde_json::json;
// `EditRequest` derives `Validate`; the derive macro is already in scope via
// `#[macro_use] extern crate validator` in main.rs, but the trait itself has to
// be imported for `er.validate()` to resolve.
use validator::Validate;

/// The first tag that repeats in `tags`, compared case-insensitively.
///
/// `tags.name` is `citext`, so "Rust" and "rust" name the SAME tag - a
/// case-sensitive check would let `["rust", "Rust"]` through and it would then
/// be rejected downstream as "All tags were not found".
fn duplicate_tag(tags: &[String]) -> Option<String> {
    let mut seen = std::collections::HashSet::new();
    for t in tags {
        if !seen.insert(t.trim().to_lowercase()) {
            return Some(t.clone());
        }
    }
    None
}

/// Hard ceiling on the number of questions a single `/search-questions/`
/// response may contain. Search results are relevance-ranked
/// (`ts_rank_cd` desc), so clamping the requested page size to this keeps the
/// most relevant matches (the "top 50") and bounds the payload regardless of
/// what the client asks for.
const SEARCH_RESULTS_LIMIT: i64 = 50;

/// True for moderators and superusers - the only roles allowed to list and
/// review post reports.
fn is_reviewer(auth: &AuthorizationService) -> bool {
    auth.is_moderator()
}

/// File a report against a post (question or answer).
///
/// Rules: the post must exist, you cannot report your own post, you cannot
/// have a second OPEN report on the same post, and you are limited to
/// `REPORTS_PER_DAY_LIMIT` (5) reports per rolling 24 hours.
#[post("/report-post/")]
async fn report_post(
    form: ntex::web::types::Json<ReportReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();

    let reason = form.reason.trim().to_lowercase();
    if !REPORT_REASONS.contains(&reason.as_str()) {
        return HttpResponse::BadRequest().json(&json!({
            "status": "fail",
            "message": format!("Invalid reason. Must be one of: {}", REPORT_REASONS.join(", ")),
        }));
    }
    let details = form.details.unwrap_or_default();
    let details = details.trim();
    if details.len() > 2000 {
        return HttpResponse::BadRequest().json(&json!({
            "status": "fail",
            "message": "Details must be at most 2000 characters.",
        }));
    }

    match state
        .get_ref()
        .report_post(form.post_id, &reason, details, auth.user.id)
        .await
    {
        Ok(ReportOutcome::Created) => HttpResponse::Ok().json(&json!({"success": true})),
        Ok(ReportOutcome::PostNotFound) => HttpResponse::NotFound().json(&json!({
            "status": "fail", "message": "Post not found.",
        })),
        Ok(ReportOutcome::OwnPost) => HttpResponse::BadRequest().json(&json!({
            "status": "fail", "message": "You cannot report your own post.",
        })),
        Ok(ReportOutcome::AlreadyReported) => HttpResponse::Ok().json(&json!({
            "success": false, "data": "You have already reported this post.",
        })),
        Ok(ReportOutcome::RateLimited) => HttpResponse::TooManyRequests().json(&json!({
            "status": "fail",
            "message": format!(
                "Report limit reached: at most {} reports per day.",
                REPORTS_PER_DAY_LIMIT
            ),
        })),
        Ok(_) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": "Unexpected report outcome."})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

const REPORT_STATUSES: [&str; 4] = ["open", "reviewed", "dismissed", "actioned"];


#[post("/create-question")]
async fn insert_question(
    form: ntex::web::types::Json<QuestionRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let r = form.into_inner();
    if r.tag_list.len() == 0 {
        return HttpResponse::UnprocessableEntity()
            .json(&json!({"status": "fail", "message": "All tags were not found"}));
    }

    if r.tag_list.len() > 5 {
        return HttpResponse::UnprocessableEntity()
            .json(&json!({"status": "fail", "message": "Too many tags."}));
    }

    // Reject repeated tags up front. Without this, a duplicate reached
    // `insert_question`, whose `tag_ids.len() != tag_list.len()` guard fired
    // and reported the misleading "All tags were not found" even though every
    // tag existed.
    if let Some(dup) = duplicate_tag(&r.tag_list) {
        return HttpResponse::UnprocessableEntity()
            .json(&json!({"status": "fail", "message": format!("Duplicate tag: {dup}")}));
    }

    // `QuestionRequest` declares title and body lengths but nothing called
    // `validate()`, so a 2-character title was accepted. Checked after the
    // tag rules so their 422 responses are unchanged.
    if let Err(e) = r.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }

    let slug = create_slug(&r.title).await;

    let q = DbQuestion {
        title: r.title,
        description: r.description,
        posted_by_id: auth.user.id,
        slug: slug.clone(),
        updated_by_id: auth.user.id,
        tag_list: r.tag_list,
    };

    match state.get_ref().insert_question(&q).await {
        Ok(0) => {
            return HttpResponse::BadRequest()
                .json(&json!({"status": "fail", "message": "All tags were not found"}))
        }
        Ok(t) => {
            debug!("insert question {:?} ", t);
            let res = AskResponse {
                id: t.to_string(),
                slug,
            };
            HttpResponse::Ok().json(&json!({"data": res}))
        }
        Err(e) => {
            error!("insert question error: {:?}", e);
            db_error(&e, json!("fail"))
        }
    }
}

#[get("/questions/{id}/{slug}")]
async fn get_question(
    params: ntex::web::types::Path<(String, String)>,
    req: HttpRequest,
    state: AppState,
) -> impl Responder {
    let remote_address = match req.peer_addr() {
        Some(ra) => ra.to_string(),
        None => "".to_owned(),
    };

    let parts: Vec<&str> = remote_address.split(":").collect();
    let mut ipaddr = "";

    if parts[0] != "" {
        ipaddr = parts[0];
    }

    let uid = optional_user_id(&req, state.get_ref()).await;

    let qid = match parse_id(&params.0) {
        Ok(id) => id,
        Err(r) => return r,
    };
    debug!("Question id is {}", qid);
    match state.get_ref().get_question(qid, uid, ipaddr).await {
        Ok(db_question) => HttpResponse::Ok().json(&json!({"data": db_question})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/questions/")]
async fn get_questions(
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();
    let uat = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    if form.questions_per_page > state.config.questions_per_page as i64 {
        return HttpResponse::BadRequest().json(
            &json!({"status": false, "message": "Wrong values for questions to be fetched!"}),
        );
    }

    match state
        .get_ref()
        .get_questions(&uat, form.questions_per_page, &form.direction)
        .await
    {
        Ok((questions, pinned)) => HttpResponse::Ok()
            .json(&json!({"data": questions, "pinned": pinned, "count": questions.count})),
        Err(e) => db_error(&e, json!(false)),
    }
}

#[post("/questions/tagged/{tag}")]
async fn get_questions_by_tag(
    params: ntex::web::types::Path<String>,
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let tag = params.into_inner();
    let form = form.into_inner();
    let uat = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    if form.questions_per_page > state.config.questions_per_page as i64 {
        return HttpResponse::BadRequest().json(
            &json!({"status": false, "message": "Wrong values for questions to be fetched!"}),
        );
    }

    match state
        .get_ref()
        .get_questions_by_tag(&uat, form.questions_per_page, &tag, &form.direction)
        .await
    {
        Ok(db_questions) => {
            HttpResponse::Ok().json(&json!({"data": db_questions, "count": db_questions.count}))
        }
        Err(e) => db_error(&e, json!(false)),
    }
}

/// One page of a question's answers, oldest first, paginated with
/// `?limit=&offset=`. `count` is the total number of answers, so the client
/// can work out how many pages there are, and `offset` is the offset that
/// was actually used.
///
/// `?answer={id}` (instead of `offset`) returns the page holding that answer,
/// so a link to one answer still lands on it when it is not on page one.
///
/// Any request body is ignored: older clients sent a `cat` timestamp cursor
/// here, which this endpoint no longer uses.
#[post("/question/get-answers/{id}/")]
async fn get_answers(
    params: ntex::web::types::Path<i64>,
    q: ntex::web::types::Query<AnswersQuery>,
    req: HttpRequest,
    state: AppState,
) -> impl Responder {
    let uid = optional_user_id(&req, state.get_ref()).await;

    let qid = params.into_inner();
    let limit = q.limit.unwrap_or(ANSWERS_DEFAULT_LIMIT).clamp(1, ANSWERS_MAX_LIMIT);
    let offset = match q.answer {
        // Round the answer's position down to the start of its page.
        Some(aid) => match state.get_ref().answer_position(qid, aid).await {
            Ok(position) => position - position % limit,
            Err(e) => return db_error(&e, json!("fail")),
        },
        None => q.offset.unwrap_or(0).max(0),
    };
    debug!("{:?}, {:?}, {:?}", &qid, limit, offset);

    match state.get_ref().get_answers(qid, limit, offset, uid).await {
        Ok((db_answers, count)) => HttpResponse::Ok()
            .json(&json!({"data": db_answers, "count": count, "offset": offset})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// Email everyone who took part in question `qid` (except `author`) that a
/// new answer was posted. Runs in the background after the answer is
/// committed: a mail problem is logged, never reported as a failed answer
/// (which made users retry and post duplicates).
async fn notify_participants(state: AppStateRaw, qid: i64, author: i64) {
    let FindParticipants(participants, slug) =
        match (&state).find_participants(qid, author).await {
            Ok(p) => p,
            Err(e) => {
                error!("answer notification: participants for {}: {:?}", qid, e);
                return;
            }
        };
    if participants.is_empty() {
        return;
    }

    let from = state.config.from_name.clone() + "<" + &state.config.from_email + ">";
    let from_addr: lettre::message::Mailbox = match from.parse() {
        Ok(a) => a,
        Err(e) => {
            error!("answer notification: invalid from address {:?}: {:?}", from, e);
            return;
        }
    };

    let body = format!(
        "Hi,

A new answer has been posted for the question you interacted with.
You can read it at https://{}/questions/{}/{}
Thanks,
Team Kunjika",
        state.config.host, qid, slug
    );

    let mut builder = Message::builder()
        .from(from_addr)
        .subject("A new answer has been posted for a question you interacted with");
    // A single malformed participant address must not stop the others.
    let mut recipients = 0usize;
    for e in participants {
        match e.parse::<lettre::message::Mailbox>() {
            Ok(addr) => {
                builder = builder.bcc(addr);
                recipients += 1;
            }
            Err(err) => info!("skipping unparsable participant {:?}: {:?}", e, err),
        }
    }
    if recipients == 0 {
        return;
    }

    match builder.body(body) {
        Ok(email) => {
            if let Err(e) = state.mailer.send(email).await {
                error!("answer notification: send failed: {:?}", e);
            }
        }
        Err(e) => error!("answer notification: build failed: {:?}", e),
    }
}

#[post("/answer")]
async fn answer(
    params: ntex::web::types::Json<AnswerReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let answer = params.into_inner();

    // `AnswerReq` declares a 20..=100000 length but nothing called
    // `validate()`, so a one-character answer was accepted.
    if let Err(e) = answer.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }

    match state.get_ref().insert_answer(&answer, &auth.user.id).await {
        Ok(answer_res) => {
            ntex::rt::spawn(notify_participants(
                state.get_ref().clone(),
                answer.id,
                auth.user.id,
            ));
            HttpResponse::Ok().json(&json!({"data": answer_res.to_string()}))
        }
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[get("edit/question/{id}")]
async fn get_edit_post(
    params: ntex::web::types::Path<i64>,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();

    debug!("post id to be edited is {:?}", pid);

    match state.get_ref().get_post(pid).await {
        Ok(post) => HttpResponse::Ok().json(&json!({"data": post})),
        // A missing post is a 404, not a 500: the edit page checks for 404 and
        // redirects to the not-found route.
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound()
            .json(&json!({"status": "fail", "message": "No such post."})),
        Err(e) => {
            error!("get_post failed for {}: {:?}", pid, e);
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": "Something went wrong, try again!"}))
        }
    }
}

#[post("/edit-post/{id}")]
async fn update_post(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<EditRequest>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();
    let er = form.into_inner();

    // Only the author may edit their own post (question or answer). This
    // handler used to extract the session as `_auth` and never use it, so any
    // logged-in user could rewrite anyone's title, body and tags.
    match state.get_ref().post_owner(pid).await {
        Ok(owner) if owner == auth.user.id => {}
        Ok(_owner) => {
            return HttpResponse::Forbidden()
                .json(&json!({"status": "fail", "message": "You can only edit your own post."}))
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound()
                .json(&json!({"status": "fail", "message": "No such post."}))
        }
        Err(e) => {
            error!("post_owner lookup failed for {}: {:?}", pid, e);
            return HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": "Something went wrong, try again!"}));
        }
    }

    // `EditRequest` declares length limits but nothing ever called
    // `validate()`, so a 1-character title and a 2-character body were accepted.
    if let Err(e) = er.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }

    // Same rule as /create-question: a repeated tag is rejected rather than
    // silently collapsed by `insert_question`/`update_post`.
    if let Some(tags) = er.tag_list.as_ref() {
        if let Some(dup) = duplicate_tag(tags) {
            return HttpResponse::UnprocessableEntity()
                .json(&json!({"status": "fail", "message": format!("Duplicate tag: {dup}")}));
        }
    }

    let title = match er.title {
        Some(t) => t,
        None => "".to_owned(),
    };
    let slug = create_slug(&title.to_string()).await;

    match state
        .get_ref()
        .update_post(pid, &er.description, &er.tag_list, &title, &slug)
        .await
    {
        Ok((post, slug)) => HttpResponse::Ok().json(&json!({"data": { "id": post, "slug": slug}})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/image-upload")]
async fn image_upload(
    payload: Multipart,
    _auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    // Uploading requires a session; the extractor rejects anonymous requests.
    match save_upload(payload, state.get_ref()).await {
        Ok(url) => HttpResponse::Ok().json(&json!({"success": true, "url": url})),
        Err(r) => r,
    }
}

#[post("accept-answer/{qid}/{aid}/")]
async fn accept_answer(
    params: ntex::web::types::Path<(i64, i64)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let (qid, aid) = params.into_inner();

    match state.get_ref().accept_answer(qid, aid, &auth.user.id).await {
        Ok(AcceptOutcome::Accepted) => {
            HttpResponse::Ok().json(&json!({"success": true, "accepted": true}))
        }
        Ok(AcceptOutcome::Unaccepted) => {
            HttpResponse::Ok().json(&json!({"success": true, "accepted": false}))
        }
        Ok(AcceptOutcome::NotOwner) => HttpResponse::Forbidden().json(&json!({
            "status": "fail", "message": "Only the question's author can accept an answer.",
        })),
        Ok(AcceptOutcome::NotFound) => HttpResponse::NotFound().json(&json!({
            "status": "fail", "message": "No such answer on this question.",
        })),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/{id}/get-questions-by-user/")]
async fn get_questions_by_user(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<UserQuestionsReq>,
    state: AppState,
) -> impl Responder {
    let uid = params.into_inner();
    let form = form.into_inner();

    let time = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    match state
        .get_ref()
        .get_questions_by_user(uid, &time, &form.direction)
        .await
    {
        Ok((user_questions, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_questions.questions, "count": count}))
        }
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/{id}/get-answers-by-user/")]
async fn get_answers_by_user(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<UserAnswersReq>,
    state: AppState,
) -> impl Responder {
    let uid = params.into_inner();
    let form = form.into_inner();

    let time = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    match state
        .get_ref()
        .get_answers_by_user(uid, &time, &form.direction)
        .await
    {
        Ok((user_answers, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_answers.questions, "count": count}))
        }
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/{id}/get-bookmarks-by-user/")]
async fn get_bookmarks_by_user(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<UserBookmarksReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let uid = params.into_inner();
    let form = form.into_inner();
    let uid1 = auth.user.id;

    if uid != uid1 {
        return HttpResponse::BadRequest()
            .json(&json!({"status": false, "message": "Only self can view bookmarks!"}));
    }

    let time = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    match state
        .get_ref()
        .get_bookmarks_by_user(uid, &time, form.bookmarks_per_page, &form.direction)
        .await
    {
        Ok((user_bookmarks, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_bookmarks.questions, "count": count}))
        }
        Err(e) => db_error(&e, json!(false)),
    }
}

#[post("/bookmark/{qid}/{aid}")]
async fn bookmark(
    params: ntex::web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let (qid, aid) = match (parse_id(&params.0), parse_id(&params.1)) {
        (Ok(q), Ok(a)) => (q, a),
        (Err(r), _) | (_, Err(r)) => return r,
    };
    let uid = auth.user.id;

    match state.get_ref().bookmark(qid, aid, uid).await {
        Ok(_t) => HttpResponse::Ok().json(&json!({})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/bookmark/{qid}")]
async fn pin(
    params: ntex::web::types::Path<String>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    // Pinning puts a question at the top of everyone's front page, so it is
    // a moderation action, like reviewing reports and editing tag info.
    if !auth.is_moderator() {
        return HttpResponse::Forbidden().json(&json!({
            "status": "fail", "message": "Only moderators can pin questions.",
        }));
    }

    let qid = match parse_id(&params) {
        Ok(id) => id,
        Err(r) => return r,
    };

    match state.get_ref().pin(qid).await {
        Ok(PinOutcome::Pinned) => {
            HttpResponse::Ok().json(&json!({"pinned": true, "message": "The question has been pinned."}))
        }
        Ok(PinOutcome::Unpinned) => HttpResponse::Ok()
            .json(&json!({"pinned": false, "message": "The question has been unpinned."})),
        Ok(PinOutcome::NotFound) => HttpResponse::NotFound()
            .json(&json!({"status": "fail", "message": "No such question."})),
        Ok(PinOutcome::LimitReached) => HttpResponse::Conflict().json(&json!({
            "status": "fail",
            "message": format!("At most {MAX_PINNED} questions can be pinned. Unpin one first."),
        })),
        Err(e) => db_error(&e, json!("fail")),
    }
}

#[post("/unanswered-questions/")]
async fn get_unanswered_questions(
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();
    let uat = match parse_cursor(&form.uat) {
        Ok(t) => t,
        Err(r) => return r,
    };

    if form.questions_per_page > state.config.questions_per_page as i64 {
        return HttpResponse::BadRequest().json(
            &json!({"status": false, "message": "Wrong values for questions to be fetched!"}),
        );
    }

    match state
        .get_ref()
        .get_unanswered_questions(&uat, form.questions_per_page, &form.direction)
        .await
    {
        Ok(db_questions) => {
            HttpResponse::Ok().json(&json!({"data": db_questions, "count": db_questions.count}))
        }
        Err(e) => db_error(&e, json!(false)),
    }
}

#[get("/get-questions-count/")]
async fn get_questions_count(state: AppState) -> impl Responder {
    match state.get_ref().get_questions_count().await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => db_error(&e, json!(false)),
    }
}

#[get("/get-unanswered-questions-count/")]
async fn get_unanswered_questions_count(state: AppState) -> impl Responder {
    match state.get_ref().get_unanswered_questions_count().await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => db_error(&e, json!(false)),
    }
}

/// Full-text search over questions, backed by the `posts_search_idx` GIN index
/// (see `20261009120000_posts_search_idx.sql`). `search` is tokenised with
/// `plainto_tsquery('english', …)` and results are ranked with `ts_rank_cd`.
#[post("/search-questions/")]
async fn search_questions(
    form: ntex::web::types::Json<SearchReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();

    // Cap the response at the top `SEARCH_RESULTS_LIMIT` (50) most relevant
    // questions, whatever the client requested. Clamping (rather than
    // rejecting) keeps the client's requested page size working while bounding
    // the payload; results stay ordered by relevance.
    let limit = form.questions_per_page.clamp(1, SEARCH_RESULTS_LIMIT);

    match state
        .get_ref()
        .search_questions(&form.search, limit)
        .await
    {
        Ok(db_questions) => {
            HttpResponse::Ok().json(&json!({"data": db_questions, "count": db_questions.count}))
        }
        Err(e) => db_error(&e, json!(false)),
    }
}

/// Moderation queue: list reports by status, newest first. Moderators and
/// superusers only.
///
/// Cursor paginated: send the `next_cursor` of one page as `cursor` to get
/// the page after it; `next_cursor` is null on the last page. Unlike an
/// offset, the cursor stays correct while reports are filed and reviewed
/// between requests.
#[post("/reports/")]
async fn get_reports(
    form: ntex::web::types::Json<ReportsReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !is_reviewer(&auth) {
        return HttpResponse::Forbidden().json(&json!({
            "status": "fail", "message": "Only moderators can view reports.",
        }));
    }

    let status = form.status.clone().unwrap_or_else(|| "open".to_string());
    let status = status.trim().to_lowercase();
    if !REPORT_STATUSES.contains(&status.as_str()) {
        return HttpResponse::BadRequest().json(&json!({
            "status": "fail",
            "message": format!("Invalid status. Must be one of: {}", REPORT_STATUSES.join(", ")),
        }));
    }
    let per_page = form.per_page.unwrap_or(20).clamp(1, 100);
    let cursor = match parse_keyset_cursor(form.cursor.as_deref()) {
        Ok(c) => c,
        Err(r) => return r,
    };

    match state.get_ref().get_reports(&status, per_page, cursor).await {
        Ok(reports) => HttpResponse::Ok().json(&json!({
            "count": reports.count,
            "next_cursor": reports.next_cursor,
            "data": reports,
        })),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// Number of OPEN reports, for the sidebar badge. Moderators and superusers
/// only - the count would otherwise leak moderation activity.
#[post("/reports/open-count/")]
async fn get_open_reports_count(auth: AuthorizationService, state: AppState) -> impl Responder {
    if !is_reviewer(&auth) {
        return HttpResponse::Forbidden().json(&json!({
            "status": "fail", "message": "Only moderators can view reports.",
        }));
    }

    match state.get_ref().get_open_reports_count().await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

/// Review a report: `reviewed`, `dismissed`, or `actioned`.
/// `actioned` deletes the reported post. Moderators and superusers only.
#[post("/reports/{id}/review/")]
async fn review_report(
    params: ntex::web::types::Path<String>,
    form: ntex::web::types::Json<ReviewReportReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    if !is_reviewer(&auth) {
        return HttpResponse::Forbidden().json(&json!({
            "status": "fail", "message": "Only moderators can review reports.",
        }));
    }

    let report_id = match params.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(&json!({
                "status": "fail", "message": "Invalid report id.",
            }))
        }
    };
    let status = form.status.trim().to_lowercase();
    if !["reviewed", "dismissed", "actioned"].contains(&status.as_str()) {
        return HttpResponse::BadRequest().json(&json!({
            "status": "fail",
            "message": "Invalid status. Must be one of: reviewed, dismissed, actioned.",
        }));
    }

    match state
        .get_ref()
        .review_report(report_id, &status, auth.user.id)
        .await
    {
        Ok(ReportOutcome::Reviewed) => HttpResponse::Ok().json(&json!({"success": true})),
        Ok(ReportOutcome::ReportNotFound) => HttpResponse::NotFound().json(&json!({
            "status": "fail", "message": "Report not found.",
        })),
        Ok(ReportOutcome::AlreadyReviewed) => HttpResponse::Ok().json(&json!({
            "success": false, "data": "This report has already been reviewed.",
        })),
        Ok(_) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": "Unexpected review outcome."})),
        Err(e) => db_error(&e, json!("fail")),
    }
}


#[post("/edit/answer/{id}")]
async fn edit_answer(
    params: ntex::web::types::Path<i64>,
    form: ntex::web::types::Json<EditAnswer>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let pid = params.into_inner();
    let er = form.into_inner();

    // Only the author may edit their own post (question or answer). This
    // handler used to extract the session as `_auth` and never use it, so any
    // logged-in user could rewrite anyone's title, body and tags.
    match state.get_ref().post_owner(pid).await {
        Ok(owner) if owner == auth.user.id => {}
        Ok(_owner) => {
            return HttpResponse::Forbidden()
                .json(&json!({"status": "fail", "message": "You can only edit your own post."}))
        }
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound()
                .json(&json!({"status": "fail", "message": "No such post."}))
        }
        Err(e) => {
            error!("post_owner lookup failed for {}: {:?}", pid, e);
            return HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": "Something went wrong, try again!"}));
        }
    }

    // `EditRequest` declares length limits but nothing ever called
    // `validate()`, so a 1-character title and a 2-character body were accepted.
    if let Err(e) = er.validate() {
        return HttpResponse::BadRequest()
            .json(&json!({"status": "fail", "message": e.to_string()}));
    }

    match state
        .get_ref()
        .edit_answer(pid, &er.description)
        .await
    {
        Ok(()) => HttpResponse::Ok().json(&json!({"data": {}})),
        Err(e) => db_error(&e, json!("fail")),
    }
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(insert_question);
    cfg.service(get_question);
    cfg.service(get_questions);
    cfg.service(get_answers);
    cfg.service(answer);
    cfg.service(get_questions_by_tag);
    cfg.service(get_edit_post);
    cfg.service(update_post);
    cfg.service(image_upload);
    cfg.service(accept_answer);
    cfg.service(get_questions_by_user);
    cfg.service(get_answers_by_user);
    cfg.service(bookmark);
    cfg.service(pin);
    cfg.service(get_bookmarks_by_user);
    cfg.service(get_unanswered_questions);
    cfg.service(get_questions_count);
    cfg.service(get_unanswered_questions_count);
    cfg.service(search_questions);
    cfg.service(report_post);
    cfg.service(get_reports);
    cfg.service(get_open_reports_count);
    cfg.service(review_report);
    cfg.service(edit_answer);
}
