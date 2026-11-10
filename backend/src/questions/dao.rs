use std::collections::HashSet;
use super::question::*;
use crate::state::AppStateRaw;
use crate::moderation::dao::{purge_post, snapshot_post};
use crate::utils::request::encode_keyset_cursor;

#[async_trait]
pub trait IQuestion: std::ops::Deref<Target = AppStateRaw> {
    async fn insert_question(&self, name: &DbQuestion) -> sqlx::Result<u64>;
    async fn get_question(
        &self,
        qid: i64,
        uid: i64,
        ipaddr: &str,
    ) -> sqlx::Result<QuestionResponse>;
    async fn get_answers(
        &self,
        qid: i64,
        limit: i64,
        offset: i64,
        uid: i64,
    ) -> sqlx::Result<(AnswersResponse, i64)>;
    async fn answer_position(&self, qid: i64, aid: i64) -> sqlx::Result<i64>;
    async fn get_questions(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<(QuestionsResponse, QuestionsResponse)>;
    async fn get_unanswered_questions(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<QuestionsResponse>;
    async fn get_questions_by_tag(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        tag: &String,
        direction: &Option<String>,
    ) -> sqlx::Result<QuestionsResponse>;
    async fn search_questions(
        &self,
        search: &String,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<QuestionsResponse>;
    async fn get_post(&self, pid: i64) -> sqlx::Result<PostResponse>;
    /// Owner of the post, used to authorise `/edit-post/{id}`.
    /// Returns `sqlx::Error::RowNotFound` when the post does not exist.
    async fn post_owner(&self, pid: i64) -> sqlx::Result<i64>;
    async fn insert_answer(&self, answer: &AnswerReq, user_id: &i64) -> sqlx::Result<u64>;
    async fn update_post(
        &self,
        pid: i64,
        description: &String,
        tag_list: &Option<Vec<String>>,
        title: &String,
        slug: &String,
        editor_id: i64,
    ) -> sqlx::Result<(u64, String)>;
    async fn accept_answer(&self, qid: i64, aid: i64, uid: &i64) -> sqlx::Result<AcceptOutcome>;
    async fn get_questions_by_user(
        &self,
        uid: i64,
        uat: &SqlDateTime,
        direction: &Option<String>,
    ) -> sqlx::Result<(QuestionsResponse, i64)>;
    async fn get_answers_by_user(
        &self,
        uid: i64,
        uat: &SqlDateTime,
        direction: &Option<String>,
    ) -> sqlx::Result<(AnswersResponse1, i64)>;
    async fn bookmark(&self, qid: i64, aid: i64, uid: i64) -> sqlx::Result<bool>;
    async fn pin(&self, qid: i64) -> sqlx::Result<PinOutcome>;
    async fn report_post(
        &self,
        post_id: i64,
        reason: &str,
        details: &str,
        uid: i64,
    ) -> sqlx::Result<ReportOutcome>;
    async fn get_reports(
        &self,
        status: &str,
        limit: i64,
        cursor: Option<(SqlDateTime, i64)>,
    ) -> sqlx::Result<ReportsResponse>;
    async fn get_open_reports_count(&self) -> sqlx::Result<i64>;
    async fn review_report(
        &self,
        report_id: i64,
        status: &str,
        reviewer_id: i64,
    ) -> sqlx::Result<ReportOutcome>;
    async fn get_bookmarks_by_user(
        &self,
        uid: i64,
        uat: &SqlDateTime,
        bookmarks_per_page: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<(AnswersResponse1, i64)>;
    async fn get_questions_count(&self) -> sqlx::Result<i64>;
    async fn get_unanswered_questions_count(&self) -> sqlx::Result<i64>;
    async fn find_participants(&self, id: i64, user_id: i64) -> sqlx::Result<FindParticipants>;
    async fn edit_answer(
        &self,
        pid: i64,
        description: &String,
        editor_id: i64,
    ) -> sqlx::Result<()>;
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IQuestion for &AppStateRaw {
    async fn insert_question(&self, q: &DbQuestion) -> sqlx::Result<u64> {
        let mut tx = self.sql.begin().await?;

        let tag_ids = sqlx::query!(
            r#"SELECT id
            FROM tags
            where name = ANY($1);"#,
            &q.tag_list[..]
        )
        .fetch_all(&mut *tx)
        .await?;

        if tag_ids.len() != q.tag_list.len() {
            let _ = &tx.rollback().await?;
            return Ok(0);
        }

        sqlx::query!(
            r#"
            UPDATE tags set post_count=post_count + 1 where name = ANY($1)
            "#,
            &q.tag_list[..]
        )
        .execute(&mut *tx)
        .await?;

        let p = sqlx::query!(
            r#"
            INSERT INTO posts (title, description, slug, posted_by_id, updated_by_id)
            VALUES ($1 ,$2 ,$3, $4, $5)
            RETURNING id"#,
            q.title,
            q.description,
            q.slug,
            q.posted_by_id,
            q.updated_by_id
        )
        .fetch_one(&mut *tx)
        .await?;

        for t in tag_ids {
            sqlx::query!(
                r#"
                INSERT INTO post_tags(post_id, tag_id)
                VALUES($1, $2)
                "#,
                p.id as i64,
                t.id
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(p.id as u64)
    }

    async fn get_question(
        &self,
        qid: i64,
        uid: i64,
        ipaddr: &str,
    ) -> sqlx::Result<QuestionResponse> {
        let mut tx = self.sql.begin().await?;
        let mut viewed_by_user = false;
        let mut vote_by_current_user = 0;

        if uid != 0 {
            let qr = sqlx::query!(
                r#"
                select count(1) from views where userid=$1 and qid=$2
                "#r,
                uid,
                qid
            )
            .fetch_one(&mut *tx)
            .await?;
            info!("step 1");
            viewed_by_user = match qr.count {
                Some(0) => false,
                Some(_i) => true,
                None => false,
            };
            let vr = sqlx::query!(
                r#"
                select vote from votes where from_user_id=$1 and topic_id=$2
                "#r,
                uid,
                qid
            )
            .fetch_optional(&mut *tx)
            .await?;
            info!("step 2");

            vote_by_current_user = match vr {
                Some(v) => v.vote,
                None => 0,
            };
        } else if ipaddr != "" {
            let qr = sqlx::query!(
                r#"
                select count(1) from views where ipaddress=$1 and qid=$2
                "#r,
                ipaddr,
                qid
            )
            .fetch_one(&mut *tx)
            .await?;
            viewed_by_user = match qr.count {
                Some(_i) => true,
                None => false,
            };
        }

        debug!("{}, {}", viewed_by_user, uid);
        if !viewed_by_user && uid != 0 {
            sqlx::query!(
                r#"
                insert into views (userid, qid) values($1, $2)
                "#r,
                uid,
                qid
            )
            .execute(&mut *tx)
            .await?;
            info!("step 3");
            sqlx::query!(r#"update posts set views=views + 1 where id=$1"#, qid)
                .execute(&mut *tx)
                .await?;
            info!("step 4");
        } else if !viewed_by_user && ipaddr != "" {
            sqlx::query!(
                r#"
                insert into views (ipaddress, qid) values($1, $2)
                "#r,
                ipaddr,
                qid
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!(r#"update posts set views=views + 1 where id=$1"#, qid)
                .execute(&mut *tx)
                .await?;
        }

        let question = sqlx::query!(
            r#"
            select t.title as title, t.description, t.visible, t.created_at, t.posted_by_id, t.updated_at,
            t.votes, t.views, t.op_id, t.updated_by_id, users.username as username, users.image_url as image_url,
            t.closed_at, t.duplicate_of_id, d.title as "duplicate_title?", d.slug as "duplicate_slug?",
            exists(select 1 from post_revisions r where r.post_id = t.id) as "edited!"
            from posts t left join users on t.posted_by_id=users.id
            left join posts d on d.id = t.duplicate_of_id where t.id=$1
            "#, qid
        ).fetch_one(&mut *tx)
        .await?;
        let tags = sqlx::query!(
            r#"
            select t.name from tags t left join post_tags on post_tags.tag_id=t.id where post_tags.post_id=$1
            "#,
            qid
        ).fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        let mut trs: Vec<String> = Vec::new();
        for t in tags {
            trs.push(t.name);
        }
        let image_url = question.image_url;
        let title = match question.title {
            Some(t) => t,
            None => "".to_owned(),
        };
        let op_id = match question.op_id {
            Some(o) => o,
            None => 0i64,
        };
        let updated_by_id = match question.updated_by_id {
            Some(o) => o,
            None => 0i64,
        };
        let qr = QuestionResponse {
            id: qid.to_string(),
            title,
            description: question.description,
            visible: question.visible,
            votes: question.votes,
            views: question.views,
            op_id: op_id.to_string(),
            posted_by_id: question.posted_by_id.to_string(),
            updated_by_id: updated_by_id.to_string(),
            created_at: question.created_at,
            updated_at: question.updated_at,
            username: question
                .username
                .expect("This is not null so must be there"), // seems a bug
            image_url,
            tags: trs,
            vote_by_current_user,
            cat: question.created_at.unix_timestamp(),
            uat: question.updated_at.unix_timestamp(),
            closed: question.closed_at.is_some(),
            duplicate_of: question.duplicate_of_id.map(|id| DuplicateOf {
                id: id.to_string(),
                title: question.duplicate_title.unwrap_or_default(),
                slug: question.duplicate_slug.unwrap_or_default(),
            }),
            edited: question.edited,
        };

        Ok(qr)
    }

    async fn get_answers(
        &self,
        qid: i64,
        limit: i64,
        offset: i64,
        uid: i64,
    ) -> sqlx::Result<(AnswersResponse, i64)> {
        // `id` breaks ties between answers created in the same instant;
        // without a total order, offset pages could overlap or skip a row.
        let questions = sqlx::query!(
            r#"
            select t.id, t.description, t.visible, t.created_at, t.posted_by_id, t.updated_at,
            t.votes, t.answer_accepted, t.reply_to_id, u.username, u.image_url,
            u1.username as "rusername: Option<String>",
            u1.image_url as "rimage_url: Option<String>",
            (CASE WHEN v.vote IS NULL THEN 0
            ELSE vote::bigint END) as vote,
            exists(select 1 from post_revisions r where r.post_id = t.id) as "edited!"
            from posts t left join users as u on t.posted_by_id=u.id
            left join users as u1 on t.reply_to_id=u1.id
            left join votes as v on v.from_user_id=$4 and v.topic_id=t.id
            where t.op_id=$1 order by t.created_at asc, t.id asc limit $2 offset $3
            "#, qid, limit, offset, uid
        ).fetch_all(&self.sql)
        .await?;

        // Counted separately (not `count(*) over()`) so the total is still
        // right when `offset` is past the last answer and no row comes back.
        let total = sqlx::query!(
            r#"select count(*) as count from posts where op_id = $1"#,
            qid
        )
        .fetch_one(&self.sql)
        .await?
        .count
        .unwrap_or(0);

        let mut ars: AnswersResponse = AnswersResponse {
            questions: Vec::new(),
        };

        for q in questions {
            let image_url = q.image_url;
            let qr = AR {
                question_id: q.id.to_string(),
                description: q.description,
                visible: q.visible,
                votes: q.votes,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                answer_accepted: q.answer_accepted,
                reply_to_id: q.reply_to_id,
                rusername: q.rusername,
                rimage_url: q.rimage_url,
                vote_by_current_user: Option::expect(q.vote, "Error, which will never happen."),
                cat: q.created_at.unix_timestamp(),
                edited: q.edited,
            };
            ars.questions.push(qr);
        }
        Ok((ars, total))
    }

    /// Zero-based position of answer `aid` among question `qid`'s answers, in
    /// the order `get_answers` lists them. 0 when `aid` is not an answer to
    /// `qid`, which sends an unknown answer id to the first page.
    async fn answer_position(&self, qid: i64, aid: i64) -> sqlx::Result<i64> {
        let row = sqlx::query!(
            r#"select count(*) as count from posts t join posts a on a.id = $2 and a.op_id = $1
            where t.op_id = $1 and (t.created_at, t.id) < (a.created_at, a.id)"#,
            qid,
            aid
        )
        .fetch_one(&self.sql)
        .await?;
        Ok(row.count.unwrap_or(0))
    }

    async fn get_questions(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<(QuestionsResponse, QuestionsResponse)> {
        let questions = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                    tags on post_tags.tag_id = tags.id where t.op_id=0 and t.pinned=false and t.updated_at > $1 group by t.id, users.id order by
                    t.updated_at asc limit $2
                    "#, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            },
            None => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                    tags on post_tags.tag_id = tags.id where t.op_id=0 and t.pinned=false and t.updated_at < $1 group by t.id, users.id order by
                    t.updated_at desc limit $2
                    "#, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            }
        };

        let pinned: Vec<QR1>;

        if let None = direction {
            // consider getting qids from a separate view
            pinned = sqlx::query_as!(
                QR1,
                r#"
                select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                t.views, t.slug, t.answer_accepted, users.image_url,
                users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                as tag_id, array_agg(tags.name) as tags, t.answer_count
                from posts t left
                join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                tags on post_tags.tag_id = tags.id where t.op_id=0 and pinned=true
                group by t.id, users.id
                "#
            )
            .fetch_all(&self.sql)
            .await?;
        } else {
            pinned = vec![];
        }

        let count = sqlx::query!(r#"select count from questions_count"#)
            .fetch_one(&self.sql)
            .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        let mut qrs: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
            count: c,
        };

        for q in questions {
            let image_url = q.image_url;
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id.to_string(),
                title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid.to_string(),
                tid,
                answers: q.answer_count,
                uat: q.updated_at.unix_timestamp(),
                cat: q.created_at.unix_timestamp(),
                answer_accepted: q.answer_accepted,
            };
            qrs.questions.push(qr);
        }

        let pcount = sqlx::query!(r#"select count from pinned_count"#)
            .fetch_one(&self.sql)
            .await?;

        let pc = match pcount.count {
            Some(c) => c,
            None => 0,
        };
        let mut ps: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
            count: pc,
        };

        for p in pinned {
            let image_url = p.image_url;
            let tags = match p.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match p.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match p.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match p.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let pr = QR {
                id: p.id.to_string(),
                title,
                visible: p.visible,
                votes: p.votes,
                views: p.views,
                slug,
                posted_by_id: p.posted_by_id.to_string(),
                created_at: p.created_at,
                updated_at: p.updated_at,
                username: p.username,
                image_url,
                tags,
                uid: p.uid.to_string(),
                tid,
                answers: p.answer_count,
                uat: p.updated_at.unix_timestamp(),
                cat: p.created_at.unix_timestamp(),
                answer_accepted: p.answer_accepted,
            };
            ps.questions.push(pr);
        }
        Ok((qrs, ps))
    }

    async fn get_questions_by_tag(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        tag: &String,
        direction: &Option<String>,
    ) -> sqlx::Result<QuestionsResponse> {
        let questions = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at , t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id,
                    array_agg(tags.name) as tags, t.answer_count from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                    tags on post_tags.tag_id = tags.id where t.id in(select post_id from post_tags
                    left join tags on tags.id=post_tags.tag_id where name=$1) and t.updated_at > $2 group by t.id, users.id order by
                    t.updated_at asc limit $3
                    "#, tag, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            },
            None => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at , t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id,
                    array_agg(tags.name) as tags, t.answer_count from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                    tags on post_tags.tag_id = tags.id where t.id in(select post_id from post_tags
                    left join tags on tags.id=post_tags.tag_id where name=$1) and t.updated_at < $2 group by t.id, users.id order by
                    t.updated_at desc limit $3
                    "#, tag, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            }
        };

        let count = sqlx::query!(r#"select post_count from tags where name=$1"#, tag)
            .fetch_one(&self.sql)
            .await?;

        let c = match count.post_count {
            Some(c) => c,
            None => 0,
        };

        let mut qrs = QuestionsResponse {
            questions: Vec::new(),
            count: c,
        };

        for q in questions {
            let image_url = q.image_url;
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id.to_string(),
                title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid.to_string(),
                tid,
                answers: q.answer_count,
                uat: q.updated_at.unix_timestamp(),
                cat: q.created_at.unix_timestamp(),
                answer_accepted: q.answer_accepted,
            };
            qrs.questions.push(qr);
        }
        Ok(qrs)
    }

    async fn search_questions(
        &self,
        search: &String,
        limit: i64,
        offset: i64,
    ) -> sqlx::Result<QuestionsResponse> {
        // The `@@ to_tsvector(...) @@ plainto_tsquery(...)` predicate and the
        // `ts_rank_cd(...)` ordering mirror the exact expression indexed by
        // `posts_search_idx` (see `20261009120000_posts_search_idx.sql`), so the
        // planner uses the GIN index (`Index Scan using posts_search_idx`).
        // The `title IS NOT NULL AND title <> ''` guard matches the index's
        // partial predicate, keeping the search scoped to questions only.
        let questions = sqlx::query_as!(
            QR1,
            r#"
            select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
            t.views, t.slug, t.answer_accepted, users.image_url,
            users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id,
            array_agg(tags.name) as tags, t.answer_count from posts t left
            join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
            tags on post_tags.tag_id = tags.id where t.title is not null and t.title <> '' and
            to_tsvector('english', coalesce(t.title, '') || ' ' || t.description) @@ plainto_tsquery('english', $1)
            group by t.id, users.id order by
            ts_rank_cd(to_tsvector('english', coalesce(t.title, '') || ' ' || t.description), plainto_tsquery('english', $1)) desc,
            t.updated_at desc, t.id desc limit $2 offset $3
            "#,
            search,
            limit,
            offset
        )
        .fetch_all(&self.sql)
        .await?;

        // Total number of matches, so the client can page through them. Same
        // predicate as above, so it is answered from `posts_search_idx` too.
        let count = sqlx::query!(
            r#"
            select count(*) as count from posts t where t.title is not null and t.title <> '' and
            to_tsvector('english', coalesce(t.title, '') || ' ' || t.description) @@ plainto_tsquery('english', $1)
            "#,
            search
        )
        .fetch_one(&self.sql)
        .await?
        .count
        .unwrap_or(0);

        let mut qrs = QuestionsResponse {
            questions: Vec::new(),
            count,
        };

        for q in questions {
            let image_url = q.image_url;
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id.to_string(),
                title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid.to_string(),
                tid,
                answers: q.answer_count,
                uat: q.updated_at.unix_timestamp(),
                cat: q.created_at.unix_timestamp(),
                answer_accepted: q.answer_accepted,
            };
            qrs.questions.push(qr);
        }
        Ok(qrs)
    }

    async fn get_post(&self, pid: i64) -> sqlx::Result<PostResponse> {
        let r = sqlx::query!(
            r#"
            select t.title, t.description, t.created_at, t.votes, t.posted_by_id from posts t where t.id=$1
            "#,
            pid
        )
        .fetch_one(&self.sql)
        .await?;

        let mut tags = Vec::new();
        let mut _tags1 = "".to_owned();

        let title = match r.title {
            Some(t) => t,
            None => "".to_owned(),
        };
        if title != "" {
            let ts = sqlx::query!(
                r#"
                select name from tags left join post_tags on post_tags.tag_id=tags.id where post_tags.post_id=$1
                "#,
                pid
            )
            .fetch_all(&self.sql)
            .await?;
            for t in ts {
                tags.push(t.name);
            }

            _tags1 = tags.iter().map(|e| e.to_string() + ",").collect();
        }

        // The edit form needs the author (ownership check), and the header
        // shows the author's name/avatar. `posted_by_id` may dangle when the
        // account was deleted, so fall back to empty rather than 500-ing.
        let author = sqlx::query!(
            r#"
            select username, image_url from users where id=$1
            "#,
            r.posted_by_id
        )
        .fetch_optional(&self.sql)
        .await?;

        let pr = PostResponse {
            title,
            description: r.description,
            tags,
            created_at: r.created_at,
            votes: r.votes,
            posted_by_id: r.posted_by_id,
            username: author.as_ref().map(|a| a.username.clone()).unwrap_or_default(),
            image_url: author.as_ref().map(|a| a.image_url.clone()),
        };

        Ok(pr)
    }

    async fn post_owner(&self, pid: i64) -> sqlx::Result<i64> {
        let r = sqlx::query!(
            r#"
            select posted_by_id from posts where id = $1
            "#,
            pid
        )
        .fetch_one(&self.sql)
        .await?;
        Ok(r.posted_by_id)
    }

    async fn insert_answer(&self, answer: &AnswerReq, user_id: &i64) -> sqlx::Result<u64> {
        let mut tx = self.sql.begin().await?;

        let p = sqlx::query!(
            r#"
            insert into posts(description, posted_by_id, op_id, reply_to_id)
            values ($1, $2, $3, $4) returning id
            "#,
            answer.value,
            user_id,
            answer.id,
            answer.reply_to
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query! {
            r#"
            update posts set answer_count = answer_count + 1 where id = $1
            "#,
            answer.id
        }
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(p.id as u64)
    }

    async fn update_post(
        &self,
        pid: i64,
        description: &String,
        tag_list: &Option<Vec<String>>,
        title: &String,
        slug: &String,
        editor_id: i64,
    ) -> sqlx::Result<(u64, String)> {
        let t_tags = Vec::new();
        let tags = match tag_list {
            Some(tl) => tl,
            None => &t_tags,
        };

        let mut tx = self.sql.begin().await?;

        // Keep the version being replaced (edit history).
        snapshot_post(&mut tx, pid, editor_id, title, description).await?;

        sqlx::query!(
            r#"
            update posts set description=$1, updated_at=now() where id=$2
            "#,
            description,
            pid
        )
        .execute(&mut *tx)
        .await?;

        // this will hold the result to be returned
        let pr;
        let pr1;
        let mut id;
        // if title is not empty then it is a question
        if title != "" {
            let tags1 = sqlx::query!(
                r#"
                select id, name from tags where name = ANY($1)
                "#,
                tags
            )
            .fetch_all(&mut *tx)
            .await?;
            let ts = sqlx::query!(
                r#"
                select id, name from tags where id in (select tag_id from post_tags where post_id=$1)
                "#,
                pid
            )
            .fetch_all(&mut *tx)
            .await?;

            // for t in &ts {
            //     if !tags.contains(&t.name) {
            //         let _ = &tx.rollback().await?;
            //         return Ok((0, "".to_owned()));
            //     }
            // }

            for t in ts {
                sqlx::query!(
                    r#"
                    update tags set post_count=post_count - 1 where name=$1
                    "#,
                    t.name
                )
                .execute(&mut *tx)
                .await?;
                sqlx::query!(
                    r#"
                    delete from post_tags where post_id=$1 and tag_id=$2
                    "#,
                    pid,
                    t.id
                )
                .execute(&mut *tx)
                .await?;
            }

            for t in tags1 {
                sqlx::query!(
                    r#"
                    update tags set post_count=post_count + 1 where name=$1
                    "#,
                    t.name
                )
                .execute(&mut *tx)
                .await?;
                sqlx::query!(
                    r#"
                    insert into post_tags(post_id, tag_id) values($1, $2)
                    "#,
                    pid,
                    t.id
                )
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query!(
                r#"
                update posts set title=$1, slug=$2, updated_at=now() where id=$3
                "#,
                title,
                slug,
                pid
            )
            .execute(&mut *tx)
            .await?;
        }

        pr = sqlx::query!(
            r#"
            select id, slug from posts where op_id=0 and id=$1
            "#,
            pid
        )
        .fetch_one(&mut *tx)
        .await;

        id = match pr.as_ref() {
            Ok(pr) => match pr.id {
                id => id as i64,
            },
            Err(_e) => 0,
        };
        let mut slug = match pr {
            Ok(pr) => match pr.slug {
                Some(s) => s,
                None => "".to_owned(),
            },
            Err(_e) => "".to_owned(),
        };

        debug!("id is {}", id);
        if id == 0 {
            pr1 = sqlx::query!(
                r#"
            select id, slug from posts where id in (select op_id from posts where op_id!=0 and id=$1)
            "#,
                pid
            )
            .fetch_one(&mut *tx)
            .await?;
            id = pr1.id;
            // An answer's parent question always has a slug, but a NULL would
            // previously panic here instead of degrading gracefully.
            slug = pr1.slug.unwrap_or_default();
        }

        tx.commit().await?;

        println!("The post id is {}", id);
        Ok((id as u64, slug))
    }

    /// Accept answer `aid` on question `qid`, or withdraw acceptance when it
    /// is already the accepted one. Karma moves with acceptance: the answer's
    /// author gains `karma_gain_per_answer`, and loses it again (floored at 1)
    /// when acceptance is withdrawn or moved to another answer. Self-answers
    /// earn nothing either way.
    async fn accept_answer(&self, qid: i64, aid: i64, uid: &i64) -> sqlx::Result<AcceptOutcome> {
        let mut tx = self.sql.begin().await?;

        let question = sqlx::query!(
            r#"select posted_by_id from posts where id=$1 and op_id=0"#,
            qid,
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(question) = question else {
            return Ok(AcceptOutcome::NotFound);
        };
        if question.posted_by_id != *uid {
            return Ok(AcceptOutcome::NotOwner);
        }

        // The answer must belong to this question. Without this check the
        // question's author could "accept" any post on the site and award
        // its author karma.
        let answer = sqlx::query!(
            r#"select posted_by_id from posts where id=$1 and op_id=$2"#,
            aid,
            qid
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(answer) = answer else {
            return Ok(AcceptOutcome::NotFound);
        };

        let current = sqlx::query!(
            r#"select id, posted_by_id from posts where op_id=$1 and answer_accepted=true"#,
            qid
        )
        .fetch_optional(&mut *tx)
        .await?;

        // Withdraw the current acceptance, if any, and the karma it gave.
        if let Some(cur) = &current {
            sqlx::query!(
                r#"update posts set answer_accepted=false where id=$1 or id=$2"#,
                cur.id,
                qid
            )
            .execute(&mut *tx)
            .await?;
            if cur.posted_by_id != *uid {
                sqlx::query!(
                    r#"update users set karma=greatest(karma - $2, 1) where id=$1"#,
                    cur.posted_by_id,
                    self.config.karma_gain_per_answer
                )
                .execute(&mut *tx)
                .await?;
            }
            // Clicking the accepted answer again un-accepts it.
            if cur.id == aid {
                tx.commit().await?;
                return Ok(AcceptOutcome::Unaccepted);
            }
        }

        sqlx::query!(
            r#"update posts set answer_accepted=true where id=$1 or id=$2"#,
            aid,
            qid
        )
        .execute(&mut *tx)
        .await?;
        if answer.posted_by_id != *uid {
            sqlx::query!(
                r#"update users set karma=karma + $2 where id=$1"#,
                answer.posted_by_id,
                self.config.karma_gain_per_answer
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(AcceptOutcome::Accepted)
    }

    // This function does a translation from QR1 to QR which are very similar.
    // To see why this is necessary convert query_as to query.
    async fn get_questions_by_user(
        &self,
        uid: i64,
        updated_at: &SqlDateTime,
        direction: &Option<String>,
    ) -> sqlx::Result<(QuestionsResponse, i64)> {
        let questions = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    QR1, r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t
                    left join users on t.posted_by_id=users.id
                    left join post_tags on post_tags.post_id=t.id
                    left join tags on post_tags.tag_id = tags.id
                    where t.op_id=0 and t.posted_by_id=$1 and
                    t.updated_at > $2 group by t.id, users.id order by
                    t.updated_at asc limit $3
                    "#, uid, updated_at, self.config.questions_per_page as i64
                ).fetch_all(&self.sql)
                .await?
            }
            None => {
                sqlx::query_as!(
                    QR1, r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
                    tags on post_tags.tag_id = tags.id where t.op_id=0 and t.posted_by_id=$1 and
                    t.updated_at < $2 group by t.id, users.id order by
                    t.updated_at desc limit $3
                    "#, uid, updated_at, self.config.questions_per_page as i64
                ).fetch_all(&self.sql)
                .await?
            }
        };

        let count = sqlx::query!(
            r#"select count from questions_count_by_user where posted_by_id = $1"#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        let mut qrs: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
            count: c,
        };

        for q in questions {
            let image_url = q.image_url;
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id.to_string(),
                title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid.to_string(),
                tid,
                answers: q.answer_count,
                uat: q.updated_at.unix_timestamp(),
                cat: q.created_at.unix_timestamp(),
                answer_accepted: q.answer_accepted,
            };
            qrs.questions.push(qr);
            info!("{}", q.created_at);
        }

        Ok((qrs, c))
    }

    async fn get_answers_by_user(
        &self,
        uid: i64,
        uat: &SqlDateTime,
        direction: &Option<String>,
    ) -> sqlx::Result<(AnswersResponse1, i64)> {
        let mut tx = self.sql.begin().await?;

        let count = sqlx::query!(
            r#"select count from answers_count where posted_by_id = $1"#,
            uid
        )
        .fetch_one(&mut *tx)
        .await?;

        let answers = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    AR1,
                    r#"
                select t.id as answer_id, t.visible, t.updated_at,
                t.votes, t.answer_accepted, p.id as question_id, p.title, p.slug,
                array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags
                from posts t left join users as u on t.posted_by_id=u.id
                left join users as u1 on t.reply_to_id=u1.id
                left join posts as p on p.id=t.op_id
                left join post_tags on post_tags.post_id=p.id
                left join tags on post_tags.tag_id = tags.id
                where t.op_id!=0 and t.posted_by_id=$1 and t.updated_at > $2
                group by p.id, t.id order by t.updated_at asc limit $3
                "#,
                    uid,
                    uat,
                    self.config.questions_per_page as i64
                )
                .fetch_all(&mut *tx)
                .await?
            }
            None => {
                sqlx::query_as!(
                    AR1,
                    r#"
                select t.id as answer_id, t.visible, t.updated_at,
                t.votes, t.answer_accepted, p.id as question_id, p.title, p.slug,
                array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags
                from posts t left join users as u on t.posted_by_id=u.id
                left join users as u1 on t.reply_to_id=u1.id
                left join posts as p on p.id=t.op_id
                left join post_tags on post_tags.post_id=p.id
                left join tags on post_tags.tag_id = tags.id
                where t.op_id!=0 and t.posted_by_id=$1 and t.updated_at < $2
                group by p.id, t.id order by t.updated_at desc limit $3
                "#,
                    uid,
                    uat,
                    self.config.questions_per_page as i64
                )
                .fetch_all(&mut *tx)
                .await?
            }
        };

        tx.commit().await?;

        let mut ars = AnswersResponse1 {
            questions: Vec::new(),
        };

        for q in answers {
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let qr = AR2 {
                question_id: q.question_id,
                visible: q.visible,
                answer_id: q.answer_id,
                title,
                tags,
                tid,
                slug,
                votes: q.votes,
                updated_at: q.updated_at,
                answer_accepted: q.answer_accepted,
                uat: q.updated_at.unix_timestamp(),
            };
            ars.questions.push(qr);
        }

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        Ok((ars, c))
    }

    async fn bookmark(&self, qid: i64, aid: i64, uid: i64) -> sqlx::Result<bool> {
        let r = sqlx::query!(
            r#"
            select qid, aid, uid from bookmarks where qid=$1 and aid=$2 and uid=$3
            "#,
            qid,
            aid,
            uid
        )
        .fetch_optional(&self.sql)
        .await?;

        if let Some(_row) = r {
            // a row is found so this is an unbookmark request
            sqlx::query!(
                r#"
                delete from bookmarks where qid=$1 and aid=$2 and uid=$3
                "#,
                qid,
                aid,
                uid
            )
            .execute(&self.sql)
            .await?;

            return Ok(true);
        } else {
            // no row has been found so this is a bookmark request
            sqlx::query!(
                r#"
                insert into bookmarks(qid, aid, uid, created_at) VALUES($1, $2, $3, now())
                "#,
                qid,
                aid,
                uid
            )
            .execute(&self.sql)
            .await?;

            return Ok(true);
        }
    }

    /// Toggle `qid`'s pinned state. Only questions can be pinned, and at most
    /// `MAX_PINNED` at once; unpinning is always allowed, including when the
    /// limit has been reached.
    async fn pin(&self, qid: i64) -> sqlx::Result<PinOutcome> {
        let mut tx = self.sql.begin().await?;

        let post = sqlx::query!(
            r#"select pinned from posts where id=$1 and op_id=0"#,
            qid
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some(post) = post else {
            return Ok(PinOutcome::NotFound);
        };

        if post.pinned.unwrap_or(false) {
            sqlx::query!(r#"update posts set pinned=false where id=$1"#, qid)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            return Ok(PinOutcome::Unpinned);
        }

        let pinned = sqlx::query!(r#"select count(1) as "count!" from posts where pinned=true"#)
            .fetch_one(&mut *tx)
            .await?;
        if pinned.count >= MAX_PINNED {
            return Ok(PinOutcome::LimitReached);
        }

        sqlx::query!(r#"update posts set pinned=true where id=$1"#, qid)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(PinOutcome::Pinned)
    }

    async fn get_bookmarks_by_user(
        &self,
        uid: i64,
        uat: &SqlDateTime,
        bookmarks_per_page: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<(AnswersResponse1, i64)> {
        let mut tx = self.sql.begin().await?;

        let count = sqlx::query!(r#"select count from bookmarks_count where uid = $1"#, uid)
            .fetch_one(&mut *tx)
            .await?;

        let answers = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    AR1,
                    r#"
                    select t.visible, t.updated_at,
                    t.votes, t.answer_accepted, t.title, t.slug,
                    array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags,
                    bookmarks.qid as question_id, bookmarks.aid as answer_id
                    from posts t
                    left join post_tags on post_tags.post_id=t.id
                    left join tags on post_tags.tag_id = tags.id
                    left join bookmarks on bookmarks.qid=t.id
                    where bookmarks.uid=$1 and bookmarks.created_at > $2
                    group by visible, t.updated_at, votes, answer_accepted, title, slug,
                    qid, aid, bookmarks.created_at
                    order by bookmarks.created_at asc limit $3
                    "#,
                    uid,
                    uat,
                    bookmarks_per_page
                )
                .fetch_all(&mut *tx)
                .await?
            }
            None => {
                sqlx::query_as!(
                    AR1,
                    r#"
                    select t.visible, t.updated_at,
                    t.votes, t.answer_accepted, t.title, t.slug,
                    array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags,
                    bookmarks.qid as question_id, bookmarks.aid as answer_id
                    from posts t
                    left join post_tags on post_tags.post_id=t.id
                    left join tags on post_tags.tag_id = tags.id
                    left join bookmarks on bookmarks.qid=t.id
                    where bookmarks.uid=$1 and bookmarks.created_at < $2
                    group by visible, t.updated_at, votes, answer_accepted, title, slug,
                    qid, aid, bookmarks.created_at
                    order by bookmarks.created_at desc limit $3
                    "#,
                    uid,
                    uat,
                    bookmarks_per_page
                )
                .fetch_all(&mut *tx)
                .await?
            }
        };

        tx.commit().await?;

        let mut ars = AnswersResponse1 {
            questions: Vec::new(),
        };

        for q in answers {
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let qr = AR2 {
                question_id: q.question_id,
                visible: q.visible,
                answer_id: q.answer_id,
                title,
                tags,
                tid,
                slug,
                votes: q.votes,
                updated_at: q.updated_at,
                answer_accepted: q.answer_accepted,
                uat: q.updated_at.unix_timestamp(),
            };
            ars.questions.push(qr);
        }

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        Ok((ars, c))
    }

    async fn get_unanswered_questions(
        &self,
        updated_at: &SqlDateTime,
        limit: i64,
        direction: &Option<String>,
    ) -> sqlx::Result<QuestionsResponse> {
        let questions = match direction {
            Some(_d) => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id
                    left join tags on post_tags.tag_id = tags.id where t.op_id=0 and t.updated_at > $1
                    and t.answer_count=0 group by t.id, users.id order by
                    t.updated_at asc limit $2
                    "#, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            },
            None => {
                sqlx::query_as!(
                    QR1,
                    r#"
                    select t.id, t.visible, t.title, t.created_at, t.posted_by_id, t.updated_at, t.votes,
                    t.views, t.slug, t.answer_accepted, users.image_url,
                    users.username as username, users.id as uid, array_agg(post_tags.tag_id)
                    as tag_id, array_agg(tags.name) as tags, t.answer_count
                    from posts t left
                    join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id
                    left join tags on post_tags.tag_id = tags.id where t.op_id=0 and t.updated_at < $1
                    and t.answer_count=0 group by t.id, users.id order by t.updated_at desc limit $2
                    "#, updated_at, limit
                )
                .fetch_all(&self.sql)
                .await?
            }
        };

        let count = sqlx::query!(r#"select count from unanswered_questions_count"#)
            .fetch_one(&self.sql)
            .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        let mut qrs: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
            count: c,
        };

        for q in questions {
            let image_url = q.image_url;
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let slug = match q.slug {
                Some(s) => s,
                None => "".to_owned(),
            };
            let title = match q.title {
                Some(t) => t,
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id.to_string(),
                title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug,
                posted_by_id: q.posted_by_id.to_string(),
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid.to_string(),
                tid,
                answers: q.answer_count,
                uat: q.updated_at.unix_timestamp(),
                cat: q.created_at.unix_timestamp(),
                answer_accepted: q.answer_accepted,
            };
            qrs.questions.push(qr);
        }
        Ok(qrs)
    }

    async fn get_questions_count(&self) -> sqlx::Result<i64> {
        let count = sqlx::query!(r#"select count from questions_count"#)
            .fetch_one(&self.sql)
            .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        Ok(c)
    }

    async fn get_unanswered_questions_count(&self) -> sqlx::Result<i64> {
        let count = sqlx::query!(r#"select count from unanswered_questions_count"#)
            .fetch_one(&self.sql)
            .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        Ok(c)
    }

    async fn report_post(
        &self,
        post_id: i64,
        reason: &str,
        details: &str,
        uid: i64,
    ) -> sqlx::Result<ReportOutcome> {
        let mut tx = self.sql.begin().await?;

        // The post must exist.
        let post = sqlx::query!(
            r#"select id, posted_by_id from posts where id = $1"#,
            post_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        let post = match post {
            Some(p) => p,
            None => return Ok(ReportOutcome::PostNotFound),
        };

        // No self-reporting.
        if post.posted_by_id == uid {
            return Ok(ReportOutcome::OwnPost);
        }

        // One open report per user per post.
        let existing = sqlx::query!(
            r#"select id from post_reports
            where post_id = $1 and reported_by_id = $2 and status = 'open'"#,
            post_id,
            uid
        )
        .fetch_optional(&mut *tx)
        .await?;
        if existing.is_some() {
            return Ok(ReportOutcome::AlreadyReported);
        }

        // 5 reports per rolling 24 hours.
        let day_count = sqlx::query!(
            r#"select count(*) as count from post_reports
            where reported_by_id = $1
              and created_at > now() - interval '24 hours'"#,
            uid
        )
        .fetch_one(&mut *tx)
        .await?;
        let used = day_count.count.unwrap_or(0);
        if used >= REPORTS_PER_DAY_LIMIT {
            return Ok(ReportOutcome::RateLimited);
        }

        sqlx::query!(
            r#"insert into post_reports (post_id, reported_by_id, reason, details)
            values ($1, $2, $3, $4)"#,
            post_id,
            uid,
            reason,
            details
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(ReportOutcome::Created)
    }

    async fn get_reports(
        &self,
        status: &str,
        limit: i64,
        cursor: Option<(SqlDateTime, i64)>,
    ) -> sqlx::Result<ReportsResponse> {
        // Keyset pagination over (created_at desc, id desc): the cursor is
        // the last row of the previous page and this page is everything
        // strictly after it. One extra row is fetched to learn whether a
        // further page exists.
        let (after_at, after_id) = match cursor {
            Some((at, id)) => (Some(at), Some(id)),
            None => (None, None),
        };
        let mut rows = sqlx::query_as!(
            ReportRow,
            r#"select r.id, r.post_id, r.reason, r.details, r.status, r.created_at,
                r.reviewed_by_id, r.reviewed_at, r.reported_by_id,
                ru.username as reporter,
                p.posted_by_id as author_id,
                au.username as author,
                p.title as post_title,
                left(p.description, 200) as post_excerpt,
                (p.op_id = 0) as is_question,
                (select count(*) from post_reports r2
                 where r2.post_id = r.post_id and r2.status = 'open') as open_count
            from post_reports r
            join users ru on ru.id = r.reported_by_id
            left join posts p on p.id = r.post_id
            left join users au on au.id = p.posted_by_id
            where r.status = $1
              and ($3::timestamptz is null
                   or (r.created_at, r.id) < ($3::timestamptz, $4::bigint))
            order by r.created_at desc, r.id desc
            limit $2"#,
            status,
            limit + 1,
            after_at,
            after_id
        )
        .fetch_all(&self.sql)
        .await?;

        let total = sqlx::query!(
            r#"select count(*) as count from post_reports where status = $1"#,
            status
        )
        .fetch_one(&self.sql)
        .await?;

        let next_cursor = if rows.len() as i64 > limit {
            rows.truncate(limit as usize);
            rows.last().map(|r| encode_keyset_cursor(r.created_at, r.id))
        } else {
            None
        };

        let count = total.count.unwrap_or(0);
        Ok(ReportsResponse { reports: rows, count, next_cursor })
    }

    async fn get_open_reports_count(&self) -> sqlx::Result<i64> {
        let row = sqlx::query!(
            r#"select count(*) as count from post_reports where status = 'open'"#
        )
        .fetch_one(&self.sql)
        .await?;
        Ok(row.count.unwrap_or(0))
    }

    async fn review_report(
        &self,
        report_id: i64,
        status: &str,
        reviewer_id: i64,
    ) -> sqlx::Result<ReportOutcome> {
        let mut tx = self.sql.begin().await?;

        let report = sqlx::query!(
            r#"select id, post_id, status from post_reports where id = $1"#,
            report_id
        )
        .fetch_optional(&mut *tx)
        .await?;
        let report = match report {
            Some(r) => r,
            None => return Ok(ReportOutcome::ReportNotFound),
        };
        if report.status != "open" {
            return Ok(ReportOutcome::AlreadyReviewed);
        }

        if status == "actioned" {
            // Delete the reported post (a question takes its answers with
            // it). `purge_post` also actions every other open report on it.
            if !purge_post(&mut tx, report.post_id, reviewer_id).await? {
                // Post already gone: just close this report as reviewed.
                sqlx::query!(
                    r#"update post_reports set status = 'reviewed',
                        reviewed_by_id = $1, reviewed_at = now()
                    where id = $2"#,
                    reviewer_id,
                    report_id
                )
                .execute(&mut *tx)
                .await?;
            }
        } else {
            sqlx::query!(
                r#"update post_reports set status = $1,
                    reviewed_by_id = $2, reviewed_at = now()
                where id = $3"#,
                status,
                reviewer_id,
                report_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(ReportOutcome::Reviewed)
    }

    async fn find_participants(&self, id: i64, user_id: i64) -> sqlx::Result<FindParticipants> {

        let qr = sqlx::query!(
            r#"
            select email from users where email_notifications = true and deleted = false
            and id in(select posted_by_id from posts where id = $1 or op_id=$1)
            "#, id)
            .fetch_all(&self.sql)
            .await?;

        let op_email = sqlx::query!(
            r#"
            select email from users where id=$1
            "#,
            user_id
        )
        .fetch_one(&self.sql)
        .await?;

        let s = sqlx::query!(
            r#"
            select slug from posts where id=$1
            "#,
            id
        )
        .fetch_one(&self.sql)
        .await?.slug;

        let slug = match s{
            Some(st) => st,
            None => "".to_string()
        };

        let mut h = HashSet::new();

        for q in qr {
            if q.email != op_email.email {
                h.insert(q.email);
            }
        }

        Ok(FindParticipants(h, slug))
    }

    async fn edit_answer(
        &self,
        pid: i64,
        description: &String,
        editor_id: i64,
    ) -> sqlx::Result<()> {
        let mut tx = self.sql.begin().await?;

        snapshot_post(&mut tx, pid, editor_id, "", description).await?;

        sqlx::query!(
            r#"
            update posts set description=$1, updated_at=now() where id=$2
            "#,
            description,
            pid
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
