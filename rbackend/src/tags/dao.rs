use super::tag::*;
use crate::state::AppStateRaw;

#[async_trait]
pub trait ITag: std::ops::Deref<Target = AppStateRaw> {
    async fn tag_query(&self, name: &str) -> sqlx::Result<Vec<Tag>>;
    async fn get_all_tags(&self, name: &str, post_count: i64) -> sqlx::Result<Vec<Tag>>;
    async fn update_tag_info(&self, info: &str, id: i64) -> sqlx::Result<String>;
    async fn get_tag_info(&self, id: i64) -> sqlx::Result<String>;
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl ITag for &AppStateRaw {
    async fn tag_query(&self, name: &str) -> sqlx::Result<Vec<Tag>> {       
        let sql = format!(
            "SELECT *
            FROM tags
            where {} like '{}%';",
            "name", name
        );

        sqlx::query_as(&sql).fetch_all(&self.sql).await
    }

    async fn get_all_tags(&self, name: &str, post_count: i64) -> sqlx::Result<Vec<Tag>> {
        sqlx::query_as!(
            Tag, r#"
            select * from tags where name > $1 and post_count< $2 order by post_count desc, name limit $3
            "#,
            name, post_count, self.config.tags_per_page
        )
        .fetch_all(&self.sql)
        .await
    }

    async fn get_tag_info(&self, id: i64) -> sqlx::Result<String> {
        let r = sqlx::query!(
            r#"
            select info from tags where id=$1
            "#,
            id
        )
        .fetch_one(&self.sql)
        .await;

        let info = match r {
            Ok(t) => match t.info {
                Some(i) => i,
                None => "".to_owned()
            },
            Err(e) => e.to_string()
        };

        Ok(info)
    }

    async fn update_tag_info(&self, info: &str, id: i64) -> sqlx::Result<String> {
        let id = sqlx::query!(
            r#"
            update tags set info=$1 where id=$2 returning id
            "#,
            info, id
        )
        .fetch_one(&self.sql)
        .await;

        let tid = match id {
            Ok(t) => t.id.to_string(),
            Err(e) => e.to_string()
        };

        Ok(tid)
    }
}