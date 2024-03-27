use super::tag::*;
use crate::state::AppStateRaw;

#[async_trait]
pub trait ITag: std::ops::Deref<Target = AppStateRaw> {
    async fn tag_query(&self, name: &str) -> sqlx::Result<Vec<Tag>>;
    async fn get_all_tags_by_name(
        &self,
        name: &str,
        direction: &Option<String>,
    ) -> sqlx::Result<(Vec<Tag>, i64)>;
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

    async fn get_all_tags_by_name(
        &self,
        name: &str,
        direction: &Option<String>,
    ) -> sqlx::Result<(Vec<Tag>, i64)> {
        let tags;

        match direction {
            Some(d) => {
                tags = sqlx::query_as!(
            Tag, r#"
            select t.id, t.name, t.info, t.post_count, w.count as weekly_count, d.count as daily_count from tags t
            left join weekly_tags_by_popularity w on w.name=t.name 
            left join daily_tags_by_popularity d on d.name=t.name
            where t.name < $1 order by t.name desc limit $2
            "#,
            name, self.config.tags_per_page
        )
        .fetch_all(&self.sql)
        .await?;
            }
            None => {
                tags = sqlx::query_as!(
            Tag, r#"
            select t.id, t.name, t.info, t.post_count, w.count as weekly_count, d.count as daily_count from tags t
            left join weekly_tags_by_popularity w on w.name=t.name 
            left join daily_tags_by_popularity d on d.name=t.name
            where t.name > $1 order by t.name asc limit $2
            "#,
            name, self.config.tags_per_page
        )
        .fetch_all(&self.sql)
        .await?;
            }
        }

        let count = sqlx::query!(
            r#"
            select count from tags_count;
            "#
        )
        .fetch_one(&self.sql)
        .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        Ok((tags, c))
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
                None => "".to_owned(),
            },
            Err(e) => e.to_string(),
        };

        Ok(info)
    }

    async fn update_tag_info(&self, info: &str, id: i64) -> sqlx::Result<String> {
        let id = sqlx::query!(
            r#"
            update tags set info=$1 where id=$2 returning id
            "#,
            info,
            id
        )
        .fetch_one(&self.sql)
        .await;

        let tid = match id {
            Ok(t) => t.id.to_string(),
            Err(e) => e.to_string(),
        };

        Ok(tid)
    }
}
