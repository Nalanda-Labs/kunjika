use super::question::{*};
use crate::state::AppStateRaw;

use chrono::*;

#[async_trait]
pub trait IQuestion: std::ops::Deref<Target = AppStateRaw> {
    async fn insert_question(&self, name: &DbQuestion) -> sqlx::Result<u64>;
    async fn get_question(&self, qid: i64) -> sqlx::Result<QuestionResponse>;
    async fn get_answers(&self, qid: i64, updated_at: &chrono::DateTime<Utc>, limit: i64) -> sqlx::Result<AnswersResponse>;
    async fn get_questions(
        &self,
        updated_at: &chrono::DateTime<Utc>,
    ) -> sqlx::Result<QuestionsResponse>;
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IQuestion for &AppStateRaw {
    async fn insert_question(&self, q: &DbQuestion) -> sqlx::Result<u64> {
        let mut tx = self.sql.begin().await?;

        sqlx::query!(
            r#"
            UPDATE tags set post_count=post_count + 1 where name = ANY($1)
            "#,
            &q.tag_list[..]
        )
        .execute(&mut tx)
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
        .fetch_one(&mut tx)
        .await?;

        let tag_ids = sqlx::query!(
            r#"SELECT id
            FROM tags
            where name = ANY($1);"#,
            &q.tag_list[..]
        )
        .fetch_all(&mut tx)
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
            .execute(&mut tx)
            .await?;
        }
        tx.commit().await?;

        Ok(p.id as u64)
    }

    async fn get_question(&self, qid: i64) -> sqlx::Result<QuestionResponse> {
        let question = sqlx::query!(
            r#"
            select t.title as title, t.description, t.visible, t.created_at, t.posted_by_id, t.updated_at,
            t.votes, t.views, t.op_id, t.updated_by_id, users.username as username, users.image_url as image_url
            from posts t left join users on t.posted_by_id=users.id where t.id=$1
            "#, qid
        ).fetch_one(&self.sql)
        .await?;
        let tags = sqlx::query!(
            r#"
            select t.name from tags t left join post_tags on post_tags.tag_id=t.id where post_tags.post_id=$1
            "#,
            qid
        ).fetch_all(&self.sql)
        .await?;

        let mut trs: Vec<String> = Vec::new();
        for t in tags {
            trs.push(t.name);
        }
        let image_url = match question.image_url {
            Some(i) => i,
            None => "".to_owned(),
        };
        let qr = QuestionResponse {
            id: qid as i64,
            title: question.title,
            description: question.description,
            visible: question.visible,
            votes: question.votes,
            views: question.views,
            op_id: question.op_id,
            posted_by_id: question.posted_by_id,
            updated_by_id: question.updated_by_id,
            created_at: question.created_at,
            updated_at: question.updated_at,
            username: question.username,
            image_url: image_url,
            tags: trs,
        };

        Ok(qr)
    }

    async fn get_questions(
        &self,
        updated_at: &chrono::DateTime<Utc>,
    ) -> sqlx::Result<QuestionsResponse> {
        let questions = sqlx::query!(
            r#"
            select t.id, t.visible, t.title, t.created_at , t.posted_by_id, t.updated_at, t.votes, t.views, t.slug, users.image_url,
            users.username as username, users.id as uid, array_agg(post_tags.tag_id) as tag_id, array_agg(tags.name) as tags, t.answer_count from posts t left
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
            let image_url = match q.image_url {
                Some(i) => i,
                None => "".to_string(),
            };
            let tags = match q.tags {
                Some(t) => t.join(","),
                None => "".to_owned(),
            };
            let tid = match q.tag_id {
                Some(t) => t.iter().map(|&e| e.to_string() + ",").collect(),
                None => "".to_owned(),
            };
            let qr = QR {
                id: q.id,
                title: q.title,
                visible: q.visible,
                votes: q.votes,
                views: q.views,
                slug: q.slug,
                posted_by_id: q.posted_by_id,
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                tags,
                uid: q.uid,
                tid,
                answers: q.answer_count,
            };
            qrs.questions.push(qr);
        }
        Ok(qrs)
    }

    async fn get_answers(&self, qid: i64, updated_at: &chrono::DateTime<Utc>, limit: i64) -> sqlx::Result<AnswersResponse> {
        let questions = sqlx::query!(
            r#"
            select count(1) over(), t.id, t.description, t.visible, t.created_at, t.posted_by_id, t.updated_at,
            t.votes, t.answer_accepted, users.username, users.image_url from posts t left join users on t.posted_by_id=users.id
            where t.op_id=$1  and t.created_at > $2 order by t.created_at asc limit $3
            "#, qid, updated_at, limit
        ).fetch_all(&self.sql)
        .await?;

        let mut ars: AnswersResponse = AnswersResponse {
            questions: Vec::new(),
        };
        for q in questions {
            let image_url = match q.image_url {
                Some(i) => i,
                None => "".to_string(),
            };
            let qr = AR {
                question_id: q.id,
                description: q.description,
                visible: q.visible,
                votes: q.votes,
                posted_by_id: q.posted_by_id,
                created_at: q.created_at,
                updated_at: q.updated_at,
                username: q.username,
                image_url,
                answer_accepted: q.answer_accepted
            };
            ars.questions.push(qr);
        }
        Ok(ars)
    }
}
