use super::question::*;
use crate::state::AppStateRaw;

#[async_trait]
pub trait IQuestion: std::ops::Deref<Target = AppStateRaw> {
    async fn insert_question(&self, name: &DbQuestion) -> sqlx::Result<u64>;
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

        let id = sqlx::query!(
            r#"
        INSERT INTO posts (title, description, slug, op_id, posted_by_id, updated_by_id)
        VALUES ($1 ,$2 ,$3, $4, $5, $6)
                "#,
            q.title,
            q.description,
            q.slug,
            q.op_id,
            q.posted_by_id,
            q.updated_by_id
        )
        .execute(&mut tx)
        .await
        .map(|d| d.rows_affected());

        tx.commit().await?;

        id
    }
}
