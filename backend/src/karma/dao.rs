use crate::config::Config;
use crate::state::AppStateRaw;
use crate::utils::request::encode_keyset_cursor;

type Tx<'a> = sqlx::Transaction<'a, sqlx::Postgres>;
type SqlDateTime = time::OffsetDateTime;

/// Change `user`'s karma by `delta` and record why. The ONLY place karma is
/// written, so the history cannot drift from the number on the profile.
///
/// Karma never goes below 1. The history records the full `delta` even when
/// the floor absorbs part of it.
pub async fn apply(
    tx: &mut Tx<'_>,
    user: i64,
    delta: i64,
    reason: &str,
    post_id: Option<i64>,
    actor: Option<i64>,
) -> sqlx::Result<()> {
    if delta == 0 {
        return Ok(());
    }
    sqlx::query!(
        r#"update users set karma = greatest(karma + $2, 1) where id = $1"#,
        user,
        delta
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!(
        r#"insert into karma_events (user_id, delta, reason, post_id, actor_id)
        values ($1, $2, $3, $4, $5)"#,
        user,
        delta,
        reason,
        post_id,
        actor
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Take back the karma the posts in `doomed` earned their authors, as they
/// are about to be deleted: the votes each one holds, and the bonus for an
/// accepted answer. Must run BEFORE the votes and posts are removed.
///
/// Worked out from the votes and flags as they stand rather than from the
/// history, so it is also right for posts older than the history table.
///
/// Only gains are taken back. A post that was voted down on balance cost its
/// author karma, and deleting it does not refund that: karma is floored at
/// 1, so the loss may never have been charged in full, and a refund would
/// let someone at the floor gain karma by deleting downvoted posts.
pub async fn reverse_for_posts(
    tx: &mut Tx<'_>,
    doomed: &[i64],
    actor: i64,
    config: &Config,
) -> sqlx::Result<()> {
    let earned = sqlx::query!(
        r#"select p.id, p.posted_by_id,
            (coalesce((select sum(case when v.vote > 0 then v.vote * $2::bigint
                                       else -v.vote * $3::bigint end)
                       from votes v where v.topic_id = p.id), 0)
             + case when p.answer_accepted and coalesce(p.op_id, 0) <> 0
                         and q.posted_by_id <> p.posted_by_id
                    then $4::bigint else 0 end)::bigint as "earned!"
        from posts p
        left join posts q on q.id = p.op_id
        where p.id = any($1)"#,
        doomed,
        config.karma_gain_per_vote,
        config.karma_loss_per_vote,
        config.karma_gain_per_answer
    )
    .fetch_all(&mut **tx)
    .await?;
    for e in earned.into_iter().filter(|e| e.earned > 0) {
        apply(tx, e.posted_by_id, -e.earned, "post_deleted", Some(e.id), Some(actor)).await?;
    }
    Ok(())
}

#[derive(Serialize, Debug)]
pub struct KarmaEvent {
    pub id: i64,
    pub delta: i64,
    pub reason: String,
    pub post_id: Option<i64>,
    /// The question to link to; `None` when the post has been deleted.
    pub question_id: Option<i64>,
    pub question_title: Option<String>,
    pub question_slug: Option<String>,
    /// Whether `post_id` is an answer (link to it within the question).
    pub is_answer: bool,
    #[serde(skip)]
    pub created_at: SqlDateTime,
    /// Unix seconds of the change.
    pub cat: i64,
}

/// One page of `user`'s karma history, newest first, and the cursor for the
/// page after it. Who voted is deliberately not part of the result.
pub async fn history(
    state: &AppStateRaw,
    user: i64,
    limit: i64,
    cursor: Option<(SqlDateTime, i64)>,
) -> sqlx::Result<(Vec<KarmaEvent>, Option<String>)> {
    let (after_at, after_id) = match cursor {
        Some((at, id)) => (Some(at), Some(id)),
        None => (None, None),
    };
    let rows = sqlx::query!(
        r#"select e.id, e.delta, e.reason, e.post_id, e.created_at,
            q.id as "question_id?", q.title as "question_title?", q.slug as "question_slug?",
            (coalesce(p.op_id, 0) <> 0) as "is_answer!"
        from karma_events e
        left join posts p on p.id = e.post_id
        left join posts q on q.id = coalesce(nullif(p.op_id, 0), p.id)
        where e.user_id = $1
          and ($3::timestamptz is null
               or (e.created_at, e.id) < ($3::timestamptz, $4::bigint))
        order by e.created_at desc, e.id desc
        limit $2"#,
        user,
        limit + 1,
        after_at,
        after_id
    )
    .fetch_all(&state.sql)
    .await?;

    let mut items: Vec<KarmaEvent> = rows
        .into_iter()
        .map(|r| KarmaEvent {
            id: r.id,
            delta: r.delta,
            reason: r.reason,
            post_id: r.post_id,
            question_id: r.question_id,
            question_title: r.question_title,
            question_slug: r.question_slug,
            is_answer: r.is_answer,
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

/// The user's current karma; `None` when there is no such (undeleted) user.
pub async fn current(state: &AppStateRaw, user: i64) -> sqlx::Result<Option<i64>> {
    let row = sqlx::query!(r#"select karma from users where id = $1 and deleted = false"#, user)
        .fetch_optional(&state.sql)
        .await?;
    Ok(row.map(|r| r.karma))
}
