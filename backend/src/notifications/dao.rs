use crate::state::AppStateRaw;
use crate::utils::request::encode_keyset_cursor;

type SqlDateTime = time::OffsetDateTime;

// ---------------------------------------------------------------------------
// Writers. These run after the action they describe has been committed, and
// are best effort: a failure is logged and never fails the user's request.
// Nobody is notified about their own actions.
// ---------------------------------------------------------------------------

fn log_failure(what: &str, r: sqlx::Result<sqlx::postgres::PgQueryResult>) {
    if let Err(e) = r {
        error!("notification ({what}) not recorded: {:?}", e);
    }
}

/// A new answer `aid` on question `qid`: tell everyone who has posted in the
/// thread. The user being replied to gets a `reply` instead of an `answer`.
pub async fn on_answer(state: &AppStateRaw, qid: i64, aid: i64, actor: i64, reply_to: Option<i64>) {
    let r = sqlx::query!(
        r#"insert into notifications (user_id, kind, actor_id, question_id, post_id)
        select u.id, case when u.id = $4::bigint then 'reply' else 'answer' end, $3, $1, $2
        from users u
        where u.deleted = false and u.id <> $3
          and (u.id = $4::bigint
               or u.id in (select posted_by_id from posts where id = $1 or op_id = $1))"#,
        qid,
        aid,
        actor,
        reply_to
    )
    .execute(&state.sql)
    .await;
    log_failure("answer", r);
}

/// Answer `aid` was accepted: tell its author.
pub async fn on_accepted(state: &AppStateRaw, qid: i64, aid: i64, actor: i64) {
    let r = sqlx::query!(
        r#"insert into notifications (user_id, kind, actor_id, question_id, post_id)
        select posted_by_id, 'accepted', $3, $1, $2
        from posts where id = $2 and op_id = $1 and posted_by_id <> $3"#,
        qid,
        aid,
        actor
    )
    .execute(&state.sql)
    .await;
    log_failure("accepted", r);
}

/// Post `post_id` was upvoted: tell its author. Flipping a vote back and
/// forth does not stack up notifications: one unread upvote per voter and
/// post is enough.
pub async fn on_upvote(state: &AppStateRaw, post_id: i64, actor: i64) {
    let r = sqlx::query!(
        r#"insert into notifications (user_id, kind, actor_id, question_id, post_id)
        select p.posted_by_id, 'upvote', $2,
            case when coalesce(p.op_id, 0) = 0 then p.id else p.op_id end, p.id
        from posts p
        where p.id = $1 and p.posted_by_id <> $2
          and not exists (
              select 1 from notifications n
              where n.user_id = p.posted_by_id and n.kind = 'upvote'
                and n.actor_id = $2 and n.post_id = p.id and n.read_at is null)"#,
        post_id,
        actor
    )
    .execute(&state.sql)
    .await;
    log_failure("upvote", r);
}

/// A new question `qid`: tell everyone following one of its tags.
pub async fn on_question(state: &AppStateRaw, qid: i64, actor: i64) {
    let r = sqlx::query!(
        r#"insert into notifications (user_id, kind, actor_id, question_id, post_id)
        select distinct f.user_id, 'tag_question', $2::bigint, $1::bigint, $1::bigint
        from tag_follows f
        join post_tags pt on pt.tag_id = f.tag_id
        where pt.post_id = $1 and f.user_id <> $2"#,
        qid,
        actor
    )
    .execute(&state.sql)
    .await;
    log_failure("tag_question", r);
}

// ---------------------------------------------------------------------------
// Readers.
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug)]
pub struct Notification {
    pub id: i64,
    pub kind: String,
    pub actor_id: Option<i64>,
    pub actor: Option<String>,
    pub question_id: Option<i64>,
    pub question_title: Option<String>,
    pub question_slug: Option<String>,
    pub post_id: Option<i64>,
    pub created_at: SqlDateTime,
    /// Unix seconds of `created_at`.
    pub cat: i64,
    pub read: bool,
}

/// One page of `user`'s notifications, newest first, and the cursor for the
/// page after it (`None` on the last page). Same keyset scheme as the
/// moderation queue.
pub async fn list(
    state: &AppStateRaw,
    user: i64,
    limit: i64,
    cursor: Option<(SqlDateTime, i64)>,
) -> sqlx::Result<(Vec<Notification>, Option<String>)> {
    let (after_at, after_id) = match cursor {
        Some((at, id)) => (Some(at), Some(id)),
        None => (None, None),
    };
    let rows = sqlx::query!(
        r#"select n.id, n.kind, n.actor_id, n.question_id, n.post_id, n.created_at, n.read_at,
            a.username as "actor?", q.title as "question_title?", q.slug as "question_slug?"
        from notifications n
        left join users a on a.id = n.actor_id
        left join posts q on q.id = n.question_id
        where n.user_id = $1
          and ($3::timestamptz is null
               or (n.created_at, n.id) < ($3::timestamptz, $4::bigint))
        order by n.created_at desc, n.id desc
        limit $2"#,
        user,
        limit + 1,
        after_at,
        after_id
    )
    .fetch_all(&state.sql)
    .await?;

    let mut items: Vec<Notification> = rows
        .into_iter()
        .map(|r| Notification {
            id: r.id,
            kind: r.kind,
            actor_id: r.actor_id,
            actor: r.actor,
            question_id: r.question_id,
            question_title: r.question_title,
            question_slug: r.question_slug,
            post_id: r.post_id,
            cat: r.created_at.unix_timestamp(),
            created_at: r.created_at,
            read: r.read_at.is_some(),
        })
        .collect();
    let next = if items.len() as i64 > limit {
        items.truncate(limit as usize);
        items.last().map(|n| encode_keyset_cursor(n.created_at, n.id))
    } else {
        None
    };
    Ok((items, next))
}

pub async fn unread_count(state: &AppStateRaw, user: i64) -> sqlx::Result<i64> {
    let row = sqlx::query!(
        r#"select count(*) as count from notifications where user_id = $1 and read_at is null"#,
        user
    )
    .fetch_one(&state.sql)
    .await?;
    Ok(row.count.unwrap_or(0))
}

/// Mark one notification read, or all of them when `id` is `None`.
pub async fn mark_read(state: &AppStateRaw, user: i64, id: Option<i64>) -> sqlx::Result<()> {
    sqlx::query!(
        r#"update notifications set read_at = now()
        where user_id = $1 and read_at is null and ($2::bigint is null or id = $2::bigint)"#,
        user,
        id
    )
    .execute(&state.sql)
    .await?;
    Ok(())
}

/// Read the email opt-out, after first changing it when `set` is given.
pub async fn email_setting(state: &AppStateRaw, user: i64, set: Option<bool>) -> sqlx::Result<bool> {
    if let Some(enabled) = set {
        sqlx::query!(
            r#"update users set email_notifications = $2 where id = $1"#,
            user,
            enabled
        )
        .execute(&state.sql)
        .await?;
    }
    let row = sqlx::query!(r#"select email_notifications from users where id = $1"#, user)
        .fetch_one(&state.sql)
        .await?;
    Ok(row.email_notifications)
}

// ---------------------------------------------------------------------------
// Followed tags.
// ---------------------------------------------------------------------------

/// Follow or unfollow tag `name`. `None` when there is no such tag.
pub async fn set_following(
    state: &AppStateRaw,
    user: i64,
    name: &str,
    following: bool,
) -> sqlx::Result<Option<bool>> {
    let tag = sqlx::query!(r#"select id from tags where name = $1::text::citext"#, name)
        .fetch_optional(&state.sql)
        .await?;
    let Some(tag) = tag else {
        return Ok(None);
    };
    if following {
        sqlx::query!(
            r#"insert into tag_follows (user_id, tag_id) values ($1, $2)
            on conflict (user_id, tag_id) do nothing"#,
            user,
            tag.id
        )
        .execute(&state.sql)
        .await?;
    } else {
        sqlx::query!(
            r#"delete from tag_follows where user_id = $1 and tag_id = $2"#,
            user,
            tag.id
        )
        .execute(&state.sql)
        .await?;
    }
    Ok(Some(following))
}

/// Names of the tags `user` follows, alphabetically.
pub async fn followed_tags(state: &AppStateRaw, user: i64) -> sqlx::Result<Vec<String>> {
    let rows = sqlx::query!(
        r#"select t.name::text as "name!" from tag_follows f
        join tags t on t.id = f.tag_id
        where f.user_id = $1
        order by 1"#,
        user
    )
    .fetch_all(&state.sql)
    .await?;
    Ok(rows.into_iter().map(|r| r.name).collect())
}
