use super::vote::*;
use crate::state::AppStateRaw;

use crate::users::user::User;

#[async_trait]
pub trait IVote: std::ops::Deref<Target = AppStateRaw> {
    async fn handle_vote(&self, data: &VoteRequest, user: &User) -> sqlx::Result<bool>;
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IVote for &AppStateRaw {
    async fn handle_vote(&self, data: &VoteRequest, user: &User) -> sqlx::Result<bool> {
        let mut tx = self.sql.begin().await?;
        let query = sqlx::query!(
            r#"select posted_by_id from posts where id=$1 limit 1"#,
            &data.id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let receiving_user = match query {
            Some(r) => r.posted_by_id,
            None => 0,
        };

        if receiving_user == 0 {
            debug!("User receiving vote not found!");
            return Ok(false);
        }

        if receiving_user == user.id {
            debug!("You cannot vote on your own post!");
            return Ok(false);
        }

        let query1 = sqlx::query!(r#"select karma from users where id=$1"#, receiving_user)
            .fetch_one(&mut *tx)
            .await?;

        let karma = query1.karma;

        let vote = sqlx::query!(
            "select * from votes where topic_id=$1 and user_id=$2",
            data.id,
            user.id
        )
        .fetch_all(&mut *tx)
        .await?;

        if vote.len() == 0 {
            sqlx::query!(
                r#"insert into votes(topic_id, user_id, vote) values($1, $2, $3)"#,
                data.id,
                user.id,
                data.vote
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!(
                r#"update posts set votes = votes + $1 where id= $2"#,
                data.vote,
                data.id
            )
            .execute(&mut *tx)
            .await?;

            if data.vote == 1 {
                sqlx::query!(
                    "update users set karma = karma + $2 where id=$1",
                    receiving_user,
                    self.config.karma_gain_per_vote
                )
                .execute(&mut *tx)
                .await?;
            } else if data.vote == -1 {
                if karma + self.config.karma_loss_per_vote < 1 {
                    sqlx::query!(r#"update users set karma = 1 where id=$1"#, receiving_user)
                        .execute(&mut *tx)
                        .await?;
                } else {
                    sqlx::query!(
                        "update users set karma = karma + $2 where id=$1",
                        receiving_user,
                        self.config.karma_loss_per_vote
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
        } else {
            if data.vote == vote[0].vote {
                debug!("You can cast upvote/downvote only once.");
                return Ok(false);
            } else {
                sqlx::query!(
                    r#"update votes set vote = vote + $1 where topic_id=$2 and user_id=$3"#,
                    data.vote,
                    data.id,
                    user.id
                )
                .execute(&mut *tx)
                .await?;
                sqlx::query!(
                    r#"update posts set votes = votes + $1 where id= $2"#,
                    data.vote,
                    data.id
                )
                .execute(&mut *tx)
                .await?;

                if data.vote == 1 {
                    sqlx::query!(
                        "update users set karma = karma + $2 where id=$1",
                        receiving_user,
                        self.config.karma_gain_per_vote
                    )
                    .execute(&mut *tx)
                    .await?;
                } else if data.vote == -1 {
                    if karma + self.config.karma_loss_per_vote < 1 {
                        sqlx::query!(r#"update users set karma = 1 where id=$1"#, receiving_user)
                            .execute(&mut *tx)
                            .await?;
                    } else {
                        sqlx::query!(
                            "update users set karma = karma + $2 where id=$1",
                            receiving_user,
                            self.config.karma_loss_per_vote
                        )
                        .execute(&mut *tx)
                        .await?;
                    }
                }
            }
        }

        tx.commit().await?;
        Ok(true)
    }
}
