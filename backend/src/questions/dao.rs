use super::question::*;
use crate::state::AppStateRaw;

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
        updated_at: &SqlDateTime,
        limit: i64,
        uid: i64,
    ) -> sqlx::Result<AnswersResponse>;
    async fn get_questions(&self, updated_at: &SqlDateTime) -> sqlx::Result<QuestionsResponse>;
    async fn get_questions_by_tag(
        &self,
        updated_at: &SqlDateTime,
        tag: &String,
    ) -> sqlx::Result<QuestionsResponse>;
    async fn get_post(&self, pid: i64) -> sqlx::Result<PostResponse>;
    async fn insert_answer(&self, answer: &AnswerReq, user_id: &i64) -> sqlx::Result<u64>;
    async fn update_post(
        &self,
        pid: i64,
        description: &String,
        tag_list: &Option<Vec<String>>,
        title: &String,
        slug: &String,
    ) -> sqlx::Result<(u64, String)>;
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
            sqlx::query!(r#"update posts set views=views + 1 where id=$1"#, qid)
                .execute(&mut *tx)
                .await?;
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
            t.votes, t.views, t.op_id, t.updated_by_id, users.username as username, users.image_url as image_url
            from posts t left join users on t.posted_by_id=users.id where t.id=$1
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
            username: question.username,
            image_url: image_url,
            tags: trs,
            vote_by_current_user,
        };

        Ok(qr)
    }

    async fn get_answers(
        &self,
        qid: i64,
        updated_at: &SqlDateTime,
        limit: i64,
        uid: i64,
    ) -> sqlx::Result<AnswersResponse> {
        let questions = sqlx::query!(
            r#"
            select count(1) over(), t.id, t.description, t.visible, t.created_at, t.posted_by_id, t.updated_at,
            t.votes, t.answer_accepted, t.reply_to_id, u.username, u.image_url,
            u1.username as rusername, u1.image_url as rimage_url,
            (CASE WHEN v.vote IS NULL THEN 0
            ELSE vote::bigint END) as vote
            from posts t left join users as u on t.posted_by_id=u.id
            left join users as u1 on t.reply_to_id=u1.id
            left join votes as v on v.from_user_id=$4 and v.topic_id=t.id
            where t.op_id=$1  and t.created_at > $2 order by t.created_at asc limit $3
            "#, qid, updated_at, limit, uid
        ).fetch_all(&self.sql)
        .await?;

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
            };
            ars.questions.push(qr);
        }
        Ok(ars)
    }

    async fn get_questions(&self, updated_at: &SqlDateTime) -> sqlx::Result<QuestionsResponse> {
        let questions = sqlx::query!(
            r#"
            select t.id, t.visible, t.title, t.created_at , t.posted_by_id, t.updated_at, t.votes, t.views, t.slug, users.image_url,
            users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags, t.answer_count 
            from posts t left
            join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
            tags on post_tags.tag_id = tags.id where t.op_id=0 and t.updated_at < $1 group by t.id, users.id order by
            t.updated_at desc limit $2
            "#, updated_at, self.config.questions_per_page as i64
        ).fetch_all(&self.sql)
        .await?;

        let mut qrs: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
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
            };
            qrs.questions.push(qr);
            info!("{}", q.created_at);
        }
        Ok(qrs)
    }

    async fn get_questions_by_tag(
        &self,
        updated_at: &SqlDateTime,
        tag: &String,
    ) -> sqlx::Result<QuestionsResponse> {
        let questions = sqlx::query!(
            r#"
            select t.id, t.visible, t.title, t.created_at , t.posted_by_id, t.updated_at, t.votes, t.views, t.slug, users.image_url,
            users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags, t.answer_count from posts t left
            join users on t.posted_by_id=users.id left join post_tags on post_tags.post_id=t.id left join
            tags on post_tags.tag_id = tags.id where t.id in(select post_id from post_tags
            left join tags on tags.id=post_tags.tag_id where name=$1) and t.updated_at < $2 group by t.id, users.id order by
            t.updated_at desc limit $3
            "#, tag, updated_at, self.config.questions_per_page as i64
        ).fetch_all(&self.sql)
        .await?;

        let mut qrs: QuestionsResponse = QuestionsResponse {
            questions: Vec::new(),
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
            };
            qrs.questions.push(qr);
        }
        Ok(qrs)
    }

    async fn get_post(&self, pid: i64) -> sqlx::Result<PostResponse> {
        let r = sqlx::query!(
            r#"
            select t.title, t.description from posts t where t.id=$1
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

        let pr = PostResponse {
            title,
            description: r.description,
            tags: tags,
        };

        Ok(pr)
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
            answer.id.parse::<i64>().unwrap(),
            answer.reply_to.parse::<i64>().unwrap()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query! {
            r#"
            update posts set answer_count = answer_count + 1 where id = $1
            "#,
            answer.id.parse::<i64>().unwrap()
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
    ) -> sqlx::Result<(u64, String)> {
        let t_tags = Vec::new();
        let tags = match tag_list {
            Some(tl) => tl,
            None => &t_tags,
        };

        let mut tx = self.sql.begin().await?;

        sqlx::query!(
            r#"
            update posts set description=$1 where id=$2
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

            for t in &ts {
                if !tags.contains(&t.name) {
                    let _ = &tx.rollback().await?;
                    return Ok((0, "".to_owned()));
                }
            }

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
                update posts set title=$1, slug=$2 where id=$3
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
            slug = pr1.slug.unwrap();
        }

        tx.commit().await?;

        println!("The post id is {}", id);
        Ok((id as u64, slug))
    }
}
