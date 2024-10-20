use super::vote::*;
use crate::state::AppStateRaw;

use crate::users::user::User;

#[async_trait]
pub trait IVote: std::ops::Deref<Target = AppStateRaw> {
    async fn handle_vote(&self, data: &VoteRequest, user: &User) -> sqlx::Result<VoteResult>;
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IVote for &AppStateRaw {
    async fn handle_vote(&self, data: &VoteRequest, user: &User) -> sqlx::Result<VoteResult> {
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
            return Ok(VoteResult {
                e: VoteEnum::ReceivingUserNotFound,
                vote_by_user: -2,
            });
        }

        if receiving_user == user.id {
            debug!("You cannot vote on your own post!");
            return Ok(VoteResult {
                e: VoteEnum::VoteYourOwnPost,
                vote_by_user: -2,
            });
        }

        let query1 = sqlx::query!(r#"select karma from users where id=$1"#, receiving_user)
            .fetch_one(&mut *tx)
            .await?;

        let karma = query1.karma;

        let vote = sqlx::query!(
            "select * from votes where topic_id=$1 and from_user_id=$2",
            data.id,
            user.id
        )
        .fetch_all(&mut *tx)
        .await?;

        let mut vote_by_user = 0;

        if vote.len() == 0 {
            sqlx::query!(
                r#"insert into votes(topic_id, from_user_id, to_user_id, vote) values($1, $2, $3, $4)"#,
                data.id,
                user.id,
                receiving_user,
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
                return Ok(VoteResult {
                    e: VoteEnum::VoteOnce,
                    vote_by_user: -2,
                });
            } else {
                sqlx::query!(
                    r#"update votes set vote = vote + $1 where topic_id=$2 and from_user_id=$3"#,
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

            let vote = sqlx::query!(
                "select * from votes where topic_id=$1 and from_user_id=$2",
                data.id,
                user.id
            )
            .fetch_one(&mut *tx)
            .await?;

            vote_by_user = vote.vote;
        }

        tx.commit().await?;
        Ok(VoteResult {
            e: VoteEnum::VoteSuccessful,
            vote_by_user,
        })
    }
}
