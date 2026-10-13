use std::{fs, io::Write};

use super::dao::IQuestion;
use super::question::*;
use crate::middlewares::auth::AuthorizationService;
use crate::state::AppState;
use crate::users::user::UserCookie;
use crate::utils::slug::create_slug;

use futures::{StreamExt, TryStreamExt};
use lettre::{AsyncTransport, Message};
use ntex::{
    http::HttpMessage,
    web::{self, get, post, HttpRequest, HttpResponse, Responder},
};
use ntex_multipart::Multipart;
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;
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
    auth.user.is_moderator.unwrap_or(false) || auth.user.is_superuser.unwrap_or(false)
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
            HttpResponse::InternalServerError()
                .json(&json!({"status": "fail", "message": e.to_string()}))
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

    let logged_in = match req.cookie("logged_in") {
        Some(s) => s.to_string(),
        None => "".to_owned(),
    };

    debug!("{:?}", logged_in);
    let cookie_str: Vec<&str> = logged_in.split("=").collect();
    let u: UserCookie;
    let mut uid = 0;

    if cookie_str.len() != 2 {
        // the user is not logged in
    } else {
        u = serde_json::from_str(cookie_str[1]).unwrap();
        uid = u.user.id;
    }

    let qid: i64 = (params.0).parse().unwrap();
    debug!("Question id is {}", qid);
    match state.get_ref().get_question(qid, uid, ipaddr).await {
        Ok(db_question) => HttpResponse::Ok().json(&json!({"data": db_question})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/questions/")]
async fn get_questions(
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();
    let uat;

    debug!("{:?}", &form.uat);
    if form.uat == "" {
        uat = time::OffsetDateTime::now_utc();
        debug!("{:?}", uat);
    } else {
        uat = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
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
    let uat;

    debug!("{:?}", &form.uat);
    if form.uat == "" {
        uat = time::OffsetDateTime::now_utc();
        debug!("{:?}", uat);
    } else {
        uat = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

#[post("/question/get-answers/{id}/")]
async fn get_answers(
    cat: ntex::web::types::Json<AnswersReq>,
    params: ntex::web::types::Path<i64>,
    q: ntex::web::types::Query<AnswersQuery>,
    req: HttpRequest,
    state: AppState,
) -> impl Responder {
    let logged_in = match req.cookie("logged_in") {
        Some(s) => s.to_string(),
        None => "".to_owned(),
    };

    debug!("{:?}", logged_in);
    let cookie_str: Vec<&str> = logged_in.split("=").collect();
    let u: UserCookie;
    let mut uid = 0;

    if cookie_str.len() != 2 {
        // the user is not logged in
    } else {
        u = serde_json::from_str(cookie_str[1]).unwrap();
        uid = u.user.id;
    }

    let created_at = cat.into_inner();
    let time;
    debug!("{:?}", &created_at.cat);
    if created_at.cat == "" {
        time = time::OffsetDateTime::now_utc();
        debug!("{:?}", time);
    } else {
        time = time::OffsetDateTime::parse(&created_at.cat, &Rfc3339).unwrap();
    }

    let qid = params.into_inner();
    debug!("{:?}, {:?}, {:?}", &qid, &time, q.limit);

    match state.get_ref().get_answers(qid, &time, q.limit, uid).await {
        Ok(db_answers) => HttpResponse::Ok().json(&json!({"data": db_answers})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/answer")]
async fn answer(
    params: ntex::web::types::Json<AnswerReq>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let answer = params.into_inner();

    debug!(
        "answer request is {:?}, {:?}, {:?}",
        &answer.id, &answer.value, &answer.reply_to
    );

    match state.get_ref().insert_answer(&answer, &auth.user.id).await {
        Ok(answer_res) => {
            let p = state
                .get_ref()
                .find_participants(answer.id, auth.user.id)
                .await;

            match p {
                Ok(h1) => {
                    let FindParticipants(h, slug) = h1;

                    // Nothing to notify. Answering your OWN question is the
                    // common case where `h` is empty (the only participant is
                    // you). Building a lettre Message with neither To nor Bcc
                    // made `.body(...)` fail with `MissingTo`, and the
                    // `.unwrap()` panicked the worker and aborted the
                    // connection without a status line. Skip the mail instead
                    // of crashing - the answer itself is already committed.
                    if h.is_empty() {
                        debug!("no participants to notify for answer {}", answer_res);
                        return HttpResponse::Ok().json(&json!({"data": answer_res.to_string()}));
                    }

                    let from =
                        state.config.from_name.clone() + "<" + &state.config.from_email + ">";
                    let subject = "A new answer has been poted for a question you ineteracted with";

                    let body = format!(
                        "Hi,

A new answer has been posted for the question you interacted with.
You can read it at https://{}/questions/{}/{}
Thanks,
Team Kunjika",
                        state.config.host, answer.id, slug
                    );

                    let from_addr: lettre::message::Mailbox = match from.parse() {
                        Ok(a) => a,
                        Err(e) => {
                            error!("invalid from address {:?}: {:?}", from, e);
                            return HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                        );
                        }
                    };

                    let mut builder = Message::builder().from(from_addr).subject(subject);
                    // A single malformed participant address must not take the
                    // whole request down, so skip it instead of unwrapping.
                    let mut recipients = 0usize;
                    for e in h {
                        match e.parse::<lettre::message::Mailbox>() {
                            Ok(addr) => {
                                builder = builder.bcc(addr);
                                recipients += 1;
                            }
                            Err(err) => info!("skipping unparsable participant {:?}: {:?}", e, err),
                        }
                    }

                    if recipients == 0 {
                        debug!("no usable participant addresses - skipping notification");
                        return HttpResponse::Ok().json(&json!({"data": answer_res.to_string()}));
                    }

                    let email = match builder.body(body) {
                        Ok(e) => e,
                        Err(e) => {
                            error!("could not build notification email: {:?}", e);
                            return HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                        );
                        }
                    };

                    debug!("Sending email");
                    match &state.mailer.send(email).await {
                        Ok(r) => {
                            debug!("{:?}", r);
                            HttpResponse::Ok().json(&json!({"data": answer_res.to_string()}))
                        }
                        Err(e) => {
                            info!("{:?}", e);
                            HttpResponse::InternalServerError().json(
                            &json!({"status": "fail", "message": "Something went wrong, try again!"}),
                        )
                        }
                    }
                }
                Err(_e) => HttpResponse::InternalServerError().json(
                    &json!({"status": "fail", "message": "Could not send email to participants."}),
                ),
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/image-upload")]
async fn image_upload(mut payload: Multipart, state: AppState) -> impl Responder {
    // This will panic in case of error, which is a good thing because it means it is broken.
    // It must not panic in general for the app to be usable.
    // we do not check for image size. This should be set at web server level
    // by limiting client max body size.
    let current_date = time::OffsetDateTime::now_utc();
    let year = current_date.year();
    let month = current_date.month();
    let day = current_date.day();
    // relative path from root
    let path = format!("{}/{}/{}/{}", state.config.upload_folder, year, month, day);
    fs::create_dir_all(&path).unwrap();
    let filename = Uuid::new_v4().to_string();
    let filepath = format!("{}/{}", &path, filename);

    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut len = 0;
        let filepath1 = format!("{}/{}", &path, filename);
        let mut f = web::block(|| std::fs::File::create(filepath1))
            .await
            .unwrap();
        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let filepath2 = format!("{}/{}", &path, filename);
            let data = chunk.unwrap();
            len += data.len();
            // file size is more than 2 MB so delete the file and return response
            if len > state.config.image_max_size {
                web::block(|| std::fs::remove_file(filepath2))
                    .await
                    .unwrap();

                return HttpResponse::BadRequest().json(&json!({"success": false}));
            }
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .unwrap();
        }
    }

    let url = state.config.backend_url.clone() + &filepath;

    return HttpResponse::Ok().json(&json!({"success": true, "url": url}));
}

#[post("accept-answer/{qid}/{aid}/")]
async fn accept_answer(
    params: ntex::web::types::Path<(i64, i64)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let (qid, aid) = params.into_inner();

    match state.get_ref().accept_answer(qid, aid, &auth.user.id).await {
        Ok(_t) => HttpResponse::Ok().json(&json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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

    let time;
    debug!("{:?}", &form.uat);

    if form.uat == "" {
        time = time::OffsetDateTime::now_utc();
        debug!("{:?}", time);
    } else {
        time = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

    match state
        .get_ref()
        .get_questions_by_user(uid, &time, &form.direction)
        .await
    {
        Ok((user_questions, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_questions.questions, "count": count}))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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

    let time;
    debug!("{:?}", &form.uat);

    if form.uat == "" {
        time = time::OffsetDateTime::now_utc();
        debug!("{:?}", time);
    } else {
        time = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

    match state
        .get_ref()
        .get_answers_by_user(uid, &time, &form.direction)
        .await
    {
        Ok((user_answers, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_answers.questions, "count": count}))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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

    let time;
    debug!("{:?}", &form.uat);

    if form.uat == "" {
        time = time::OffsetDateTime::now_utc();
        debug!("{:?}", time);
    } else {
        time = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

    match state
        .get_ref()
        .get_bookmarks_by_user(uid, &time, form.bookmarks_per_page, &form.direction)
        .await
    {
        Ok((user_bookmarks, count)) => {
            HttpResponse::Ok().json(&json!({"data": user_bookmarks.questions, "count": count}))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

#[post("/bookmark/{qid}/{aid}")]
async fn bookmark(
    params: ntex::web::types::Path<(String, String)>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let qid = params.0.parse::<i64>().unwrap();
    let aid = params.1.parse::<i64>().unwrap();
    let uid = auth.user.id;

    match state.get_ref().bookmark(qid, aid, uid).await {
        Ok(_t) => HttpResponse::Ok().json(&json!({})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/bookmark/{qid}")]
async fn pin(
    params: ntex::web::types::Path<String>,
    auth: AuthorizationService,
    state: AppState,
) -> impl Responder {
    let qid = params.parse::<i64>().unwrap();
    let uid = auth.user.id;

    match state.get_ref().pin(qid, uid).await {
        Ok(_t) => {
            HttpResponse::Ok().json(&json!({"messaage": "The post has been pinned/unpinned."}))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
    }
}

#[post("/unanswered-questions/")]
async fn get_unanswered_questions(
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();
    let uat;

    debug!("{:?}", &form.uat);
    if form.uat == "" {
        uat = time::OffsetDateTime::now_utc();
        debug!("{:?}", uat);
    } else {
        uat = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

#[post("/questions-by-score/")]
async fn get_questions_by_score(
    form: ntex::web::types::Json<QuestionsReq>,
    state: AppState,
) -> impl Responder {
    let form = form.into_inner();
    let uat;

    debug!("{:?}", &form.uat);
    if form.uat == "" {
        uat = time::OffsetDateTime::now_utc();
        debug!("{:?}", uat);
    } else {
        uat = time::OffsetDateTime::parse(&form.uat, &Rfc3339).unwrap();
    }

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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

#[get("/get-questions-count/")]
async fn get_questions_count(state: AppState) -> impl Responder {
    match state.get_ref().get_questions_count().await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

#[get("/get-unanswered-questions-count/")]
async fn get_unanswered_questions_count(state: AppState) -> impl Responder {
    match state.get_ref().get_unanswered_questions_count().await {
        Ok(count) => HttpResponse::Ok().json(&json!({"count": count})),
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": false, "message": e.to_string()})),
    }
}

/// Moderation queue: list reports by status, newest first. Moderators and
/// superusers only.
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
    let page = form.page.unwrap_or(0).max(0);
    let per_page = form.per_page.unwrap_or(20).clamp(1, 100);
    let offset = page * per_page;

    match state.get_ref().get_reports(&status, per_page, offset).await {
        Ok((reports, count)) => {
            HttpResponse::Ok().json(&json!({"data": reports, "count": count}))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
        Err(e) => HttpResponse::InternalServerError()
            .json(&json!({"status": "fail", "message": e.to_string()})),
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
    cfg.service(get_questions_by_score);
    cfg.service(search_questions);
    cfg.service(report_post);
    cfg.service(get_reports);
    cfg.service(get_open_reports_count);
    cfg.service(review_report);
}
