use super::question::{self, DbQuestion, Question, QuestionResponse};
use crate::state::AppStateRaw;
use crate::tags::tag::Tag;

#[async_trait]
pub trait IQuestion: std::ops::Deref<Target = AppStateRaw> {
    async fn insert_question(&self, name: &DbQuestion) -> sqlx::Result<u64>;
    async fn get_question(&self, qid: i64) -> sqlx::Result<QuestionResponse>;
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
            INSERT INTO posts (title, description, slug, op_id, posted_by_id, updated_by_id)
            VALUES ($1 ,$2 ,$3, $4, $5, $6)
            RETURNING id"#,
            q.title,
            q.description,
            q.slug,
            q.op_id,
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
            t.votes, t.views, t.op_id, t.updated_by_id, users.username from posts t left join users on
            t.posted_by_id=users.id where t.id=$1
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
            tags: trs,
        };

        Ok(qr)
    }
}
