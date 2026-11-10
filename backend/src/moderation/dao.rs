use crate::config::Config;
use crate::state::AppStateRaw;
use crate::users::user::User;
use crate::utils::request::encode_keyset_cursor;

type Tx<'a> = sqlx::Transaction<'a, sqlx::Postgres>;
type SqlDateTime = time::OffsetDateTime;

fn is_reviewer(user: &User) -> bool {
    user.is_moderator.unwrap_or(false) || user.is_superuser.unwrap_or(false)
}

/// Permanently remove post `post_id` inside `tx`. Returns false when there is
/// no such post.
///
/// A question takes its whole thread with it: its answers would otherwise be
/// orphan rows that the list queries expect to have a parent. Everything that
/// points at a removed post goes too (tags, bookmarks, votes, notifications,
/// revisions), the counters that included it are corrected, and any report
/// still open against it is marked `actioned` by `actor_id`.
///
/// The karma the removed posts earned their authors is taken back (see
/// `karma::dao::reverse_for_posts`), and the deletion is written to the
/// moderation log with a description of what was removed.
///
/// Shared by the moderation queue ("actioned" reports) and by
/// [`delete_post`], so both paths leave the database in the same state.
pub async fn purge_post(
    tx: &mut Tx<'_>,
    post_id: i64,
    actor_id: i64,
    config: &Config,
) -> sqlx::Result<bool> {
    let target = sqlx::query!(
        r#"select p.id, p.op_id, p.answer_accepted, p.title, p.posted_by_id,
            u.username as "author?", q.title as "question_title?"
        from posts p
        left join users u on u.id = p.posted_by_id
        left join posts q on q.id = p.op_id
        where p.id = $1"#,
        post_id
    )
    .fetch_optional(&mut **tx)
    .await?;
    let Some(t) = target else {
        return Ok(false);
    };
    let question_id = t.op_id.unwrap_or(0);
    let author = t.author.clone().unwrap_or_default();
    let details = if question_id == 0 {
        format!("question \"{}\" by {author}", t.title.clone().unwrap_or_default())
    } else {
        format!(
            "answer by {author} on \"{}\"",
            t.question_title.clone().unwrap_or_default()
        )
    };

    let mut doomed: Vec<i64> = vec![t.id];
    if question_id == 0 {
        // A question: its answers go with it, and its tags lose a post.
        // Tag counters are adjusted BEFORE post_tags is emptied below.
        let answers = sqlx::query!(r#"select id from posts where op_id = $1"#, t.id)
            .fetch_all(&mut **tx)
            .await?;
        doomed.extend(answers.iter().map(|a| a.id));
        sqlx::query!(
            r#"update tags set post_count = greatest(post_count - 1, 0)
            where id in (select tag_id from post_tags where post_id = $1)"#,
            t.id
        )
        .execute(&mut **tx)
        .await?;
    } else {
        // An answer: its question has one answer fewer, and is no longer
        // "answered" if this was the accepted one.
        sqlx::query!(
            r#"update posts set
                answer_count = greatest(answer_count - 1, 0),
                answer_accepted = answer_accepted and not $2
            where id = $1"#,
            question_id,
            t.answer_accepted
        )
        .execute(&mut **tx)
        .await?;
    }

    // While the votes and accepted flags are still there to count.
    crate::karma::dao::reverse_for_posts(tx, &doomed, actor_id, config).await?;

    sqlx::query!(r#"delete from post_tags where post_id = any($1)"#, &doomed)
        .execute(&mut **tx)
        .await?;
    sqlx::query!(r#"delete from thread_mutes where question_id = any($1)"#, &doomed)
        .execute(&mut **tx)
        .await?;
    sqlx::query!(
        r#"delete from bookmarks where qid = any($1) or aid = any($1)"#,
        &doomed
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(r#"delete from votes where topic_id = any($1)"#, &doomed)
        .execute(&mut **tx)
        .await?;
    sqlx::query!(
        r#"delete from notifications where post_id = any($1) or question_id = any($1)"#,
        &doomed
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(r#"delete from post_revisions where post_id = any($1)"#, &doomed)
        .execute(&mut **tx)
        .await?;
    // Questions closed as a duplicate of a removed one stay closed, but no
    // longer link to a question that is not there.
    sqlx::query!(
        r#"update posts set duplicate_of_id = null where duplicate_of_id = any($1)"#,
        &doomed
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(r#"delete from posts where id = any($1)"#, &doomed)
        .execute(&mut **tx)
        .await?;

    // Nothing is left to review on a post that no longer exists.
    sqlx::query!(
        r#"update post_reports set status = 'actioned',
            reviewed_by_id = $1, reviewed_at = now()
        where post_id = any($2) and status = 'open'"#,
        actor_id,
        &doomed
    )
    .execute(&mut **tx)
    .await?;

    log(&mut **tx, actor_id, "delete_post", Some(post_id), Some(t.posted_by_id), &details).await?;

    Ok(true)
}

/// Add an entry to the moderation log. `details` describes the target in
/// words, because the row it points at may be deleted later (or already is).
pub async fn log<'e, E>(
    executor: E,
    actor_id: i64,
    action: &str,
    post_id: Option<i64>,
    target_user_id: Option<i64>,
    details: &str,
) -> sqlx::Result<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    // The column is varchar(512); cut on a character boundary.
    let details: String = details.chars().take(512).collect();
    sqlx::query!(
        r#"insert into moderation_log (actor_id, action, post_id, target_user_id, details)
        values ($1, $2, $3, $4, $5)"#,
        actor_id,
        action,
        post_id,
        target_user_id,
        details
    )
    .execute(executor)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
pub struct LogEntry {
    pub id: i64,
    pub action: String,
    pub actor_id: i64,
    pub actor: Option<String>,
    pub target_user_id: Option<i64>,
    pub target_user: Option<String>,
    pub post_id: Option<i64>,
    /// Where the post can still be seen; `None` once it has been deleted.
    pub question_id: Option<i64>,
    pub question_slug: Option<String>,
    pub details: String,
    #[serde(skip)]
    pub created_at: SqlDateTime,
    /// Unix seconds of the action.
    pub cat: i64,
}

/// One page of the moderation log, newest first, and the cursor for the
/// page after it.
pub async fn log_entries(
    state: &AppStateRaw,
    limit: i64,
    cursor: Option<(SqlDateTime, i64)>,
) -> sqlx::Result<(Vec<LogEntry>, Option<String>)> {
    let (after_at, after_id) = match cursor {
        Some((at, id)) => (Some(at), Some(id)),
        None => (None, None),
    };
    let rows = sqlx::query!(
        r#"select l.id, l.action, l.actor_id, l.target_user_id, l.post_id, l.details, l.created_at,
            a.username as "actor?", t.username as "target_user?",
            q.id as "question_id?", q.slug as "question_slug?"
        from moderation_log l
        left join users a on a.id = l.actor_id
        left join users t on t.id = l.target_user_id
        left join posts p on p.id = l.post_id
        left join posts q on q.id = coalesce(nullif(p.op_id, 0), p.id)
        where ($2::timestamptz is null
               or (l.created_at, l.id) < ($2::timestamptz, $3::bigint))
        order by l.created_at desc, l.id desc
        limit $1"#,
        limit + 1,
        after_at,
        after_id
    )
    .fetch_all(&state.sql)
    .await?;
    let mut items: Vec<LogEntry> = rows
        .into_iter()
        .map(|r| LogEntry {
            id: r.id,
            action: r.action,
            actor_id: r.actor_id,
            actor: r.actor,
            target_user_id: r.target_user_id,
            target_user: r.target_user,
            post_id: r.post_id,
            question_id: r.question_id,
            question_slug: r.question_slug,
            details: r.details,
            cat: r.created_at.unix_timestamp(),
            created_at: r.created_at,
        })
        .collect();
    let next = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|e| encode_keyset_cursor(e.created_at, e.id))
    } else {
        None
    };
    Ok((items, next))
}

pub enum DeleteOutcome {
    /// Deleted. `question_id` is 0 when the post was itself a question.
    Deleted { question_id: i64 },
    NotFound,
    Forbidden,
    /// Authors cannot delete a question other people have answered.
    HasAnswers,
}

/// Delete a post on behalf of `user`: authors may delete their own answers
/// and their own unanswered questions; moderators may delete anything.
pub async fn delete_post(
    state: &AppStateRaw,
    pid: i64,
    user: &User,
) -> sqlx::Result<DeleteOutcome> {
    let mut tx = state.sql.begin().await?;
    let post = sqlx::query!(
        r#"select op_id, posted_by_id, answer_count from posts where id = $1"#,
        pid
    )
    .fetch_optional(&mut *tx)
    .await?;
    let Some(post) = post else {
        return Ok(DeleteOutcome::NotFound);
    };
    let question_id = post.op_id.unwrap_or(0);
    let reviewer = is_reviewer(user);
    if post.posted_by_id != user.id && !reviewer {
        return Ok(DeleteOutcome::Forbidden);
    }
    if question_id == 0 && post.answer_count > 0 && !reviewer {
        return Ok(DeleteOutcome::HasAnswers);
    }
    purge_post(&mut tx, pid, user.id, &state.config).await?;
    tx.commit().await?;
    Ok(DeleteOutcome::Deleted { question_id })
}

/// Record post `pid` as it is now, before an edit replaces it with
/// `new_title` / `new_description`. Nothing is recorded when neither changed
/// (pass an empty `new_title` for an answer, which has none).
pub async fn snapshot_post(
    tx: &mut Tx<'_>,
    pid: i64,
    editor_id: i64,
    new_title: &str,
    new_description: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"insert into post_revisions (post_id, title, description, tags, edited_by_id)
        select p.id, p.title, p.description,
            (select string_agg(t.name::text, ',' order by t.name::text)
             from tags t join post_tags pt on pt.tag_id = t.id
             where pt.post_id = p.id),
            $2
        from posts p
        where p.id = $1
          and (p.description <> $3 or ($4 <> '' and coalesce(p.title, '') <> $4))"#,
        pid,
        editor_id,
        new_description,
        new_title
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
pub struct Revision {
    pub id: i64,
    pub title: Option<String>,
    pub description: String,
    pub tags: Option<String>,
    pub edited_by_id: Option<i64>,
    pub editor: Option<String>,
    pub created_at: SqlDateTime,
    /// Unix seconds of `created_at`: when this version was replaced.
    pub cat: i64,
}

#[derive(Serialize, Debug)]
pub struct History {
    /// The question this post belongs to (the post itself for a question).
    pub question_id: i64,
    pub question_slug: String,
    pub is_question: bool,
    /// The post as it is now.
    pub title: Option<String>,
    pub description: String,
    pub author: Option<String>,
    pub posted_by_id: i64,
    /// Earlier versions, newest first.
    pub revisions: Vec<Revision>,
}

pub async fn post_history(state: &AppStateRaw, pid: i64) -> sqlx::Result<Option<History>> {
    let post = sqlx::query!(
        r#"select p.id, p.op_id, p.title, p.description, p.posted_by_id, p.slug,
            u.username as "author?", q.slug as "question_slug?"
        from posts p
        left join users u on u.id = p.posted_by_id
        left join posts q on q.id = p.op_id
        where p.id = $1"#,
        pid
    )
    .fetch_optional(&state.sql)
    .await?;
    let Some(post) = post else {
        return Ok(None);
    };
    let rows = sqlx::query!(
        r#"select r.id, r.title, r.description, r.tags, r.edited_by_id, r.created_at,
            u.username as "editor?"
        from post_revisions r
        left join users u on u.id = r.edited_by_id
        where r.post_id = $1
        order by r.created_at desc, r.id desc
        limit 100"#,
        pid
    )
    .fetch_all(&state.sql)
    .await?;

    let op_id = post.op_id.unwrap_or(0);
    let is_question = op_id == 0;
    Ok(Some(History {
        question_id: if is_question { post.id } else { op_id },
        question_slug: if is_question { post.slug } else { post.question_slug }.unwrap_or_default(),
        is_question,
        title: post.title,
        description: post.description,
        author: post.author,
        posted_by_id: post.posted_by_id,
        revisions: rows
            .into_iter()
            .map(|r| Revision {
                id: r.id,
                title: r.title,
                description: r.description,
                tags: r.tags,
                edited_by_id: r.edited_by_id,
                editor: r.editor,
                cat: r.created_at.unix_timestamp(),
                created_at: r.created_at,
            })
            .collect(),
    }))
}

pub enum CloseOutcome {
    Done,
    /// No such question (answers cannot be closed).
    NotFound,
    /// `duplicate_of` is not an existing question.
    DuplicateNotFound,
    /// A question cannot be a duplicate of itself.
    SelfDuplicate,
}

/// Close question `qid`, optionally as a duplicate of `duplicate_of`.
/// Closing an already closed question just updates the duplicate link.
pub async fn close_question(
    state: &AppStateRaw,
    qid: i64,
    duplicate_of: Option<i64>,
    by: i64,
) -> sqlx::Result<CloseOutcome> {
    if duplicate_of == Some(qid) {
        return Ok(CloseOutcome::SelfDuplicate);
    }
    if let Some(original) = duplicate_of {
        let found = sqlx::query!(
            r#"select id from posts where id = $1 and coalesce(op_id, 0) = 0"#,
            original
        )
        .fetch_optional(&state.sql)
        .await?;
        if found.is_none() {
            return Ok(CloseOutcome::DuplicateNotFound);
        }
    }
    let done = sqlx::query!(
        r#"update posts set closed_at = coalesce(closed_at, now()), closed_by_id = $2,
            duplicate_of_id = $3
        where id = $1 and coalesce(op_id, 0) = 0"#,
        qid,
        by,
        duplicate_of
    )
    .execute(&state.sql)
    .await?;
    if done.rows_affected() == 0 {
        return Ok(CloseOutcome::NotFound);
    }
    let details = match duplicate_of {
        Some(original) => format!("{} (duplicate of #{original})", question_label(state, qid).await?),
        None => question_label(state, qid).await?,
    };
    log(&state.sql, by, "close_question", Some(qid), None, &details).await?;
    Ok(CloseOutcome::Done)
}

/// `question "<title>"`, for moderation log entries.
async fn question_label(state: &AppStateRaw, qid: i64) -> sqlx::Result<String> {
    let row = sqlx::query!(r#"select title from posts where id = $1"#, qid)
        .fetch_optional(&state.sql)
        .await?;
    Ok(format!(
        "question \"{}\"",
        row.and_then(|r| r.title).unwrap_or_default()
    ))
}

pub async fn reopen_question(state: &AppStateRaw, qid: i64, by: i64) -> sqlx::Result<CloseOutcome> {
    // Only a closed question can be reopened; reopening an open one changes
    // nothing and is not worth a log entry.
    let done = sqlx::query!(
        r#"update posts set closed_at = null, closed_by_id = null, duplicate_of_id = null
        where id = $1 and coalesce(op_id, 0) = 0 and closed_at is not null"#,
        qid
    )
    .execute(&state.sql)
    .await?;
    if done.rows_affected() > 0 {
        let details = question_label(state, qid).await?;
        log(&state.sql, by, "reopen_question", Some(qid), None, &details).await?;
        return Ok(CloseOutcome::Done);
    }
    Ok(match question_closed(state, qid).await? {
        Some(_) => CloseOutcome::Done,
        None => CloseOutcome::NotFound,
    })
}

/// Whether question `qid` is closed; `None` when it is not a question.
pub async fn question_closed(state: &AppStateRaw, qid: i64) -> sqlx::Result<Option<bool>> {
    let row = sqlx::query!(
        r#"select (closed_at is not null) as "closed!" from posts
        where id = $1 and coalesce(op_id, 0) = 0"#,
        qid
    )
    .fetch_optional(&state.sql)
    .await?;
    Ok(row.map(|r| r.closed))
}

pub enum SuspendOutcome {
    Done,
    NotFound,
    /// You cannot suspend yourself.
    SelfTarget,
    /// Moderators cannot suspend other moderators or superusers.
    Forbidden,
}

/// Suspend or reinstate `target`. A suspended account cannot sign in and its
/// existing sessions stop working (see `middlewares/auth.rs`); its posts stay.
pub async fn set_suspended(
    state: &AppStateRaw,
    target: i64,
    suspended: bool,
    actor: &User,
) -> sqlx::Result<SuspendOutcome> {
    if target == actor.id {
        return Ok(SuspendOutcome::SelfTarget);
    }
    let user = sqlx::query!(
        r#"select is_superuser, is_moderator, username, status from users where id = $1 and deleted = false"#,
        target
    )
    .fetch_optional(&state.sql)
    .await?;
    let Some(user) = user else {
        return Ok(SuspendOutcome::NotFound);
    };
    let target_is_staff =
        user.is_superuser.unwrap_or(false) || user.is_moderator.unwrap_or(false);
    let actor_is_superuser = actor.is_superuser.unwrap_or(false);
    if user.is_superuser.unwrap_or(false) || (target_is_staff && !actor_is_superuser) {
        return Ok(SuspendOutcome::Forbidden);
    }
    sqlx::query!(
        r#"update users set status = $2, modified_date = now() where id = $1"#,
        target,
        if suspended { "suspended" } else { "normal" }
    )
    .execute(&state.sql)
    .await?;
    // Logged only when the state actually changed.
    if (user.status == "suspended") != suspended {
        let action = if suspended { "suspend_user" } else { "reinstate_user" };
        log(&state.sql, actor.id, action, None, Some(target), &format!("user {}", user.username)).await?;
    }
    Ok(SuspendOutcome::Done)
}
