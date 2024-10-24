use super::user::*;
use crate::state::AppStateRaw;
use md5::compute;

#[async_trait]
pub trait IUser: std::ops::Deref<Target = AppStateRaw> {
    async fn user_add(&self, form: &Register) -> sqlx::Result<u64>;
    async fn get_users(&self, form: &UsersReq) -> sqlx::Result<(UserResponse, i64)>;
    async fn get_profile(&self, uid: &i64) -> sqlx::Result<ProfileResponse>;
    async fn update_username(&self, uid: i64, username: &String) -> sqlx::Result<bool>;
    async fn update_title(&self, uid: i64, title: &String) -> sqlx::Result<bool>;
    async fn update_name(&self, uid: i64, name: &String) -> sqlx::Result<bool>;
    async fn update_designation(&self, uid: i64, designation: &String) -> sqlx::Result<bool>;
    async fn update_location(&self, uid: i64, location: &String) -> sqlx::Result<bool>;
    async fn get_links(&self, uid: i64) -> sqlx::Result<LinksResponse>;
    async fn update_links(&self, uid: i64, form: &LinksResponse) -> sqlx::Result<bool>;
    async fn verify_email(&self, who: &str) -> sqlx::Result<bool>;
    async fn update_profile_image(&self, uid: i64, url: &String) -> sqlx::Result<bool>;
    async fn update_profile(&self, uid: &i64, data: &ProfileReq) -> sqlx::Result<bool>;
    async fn get_summary(&self, uid: i64) -> sqlx::Result<SummaryResponse>;
    async fn user_query(&self, who: &str) -> sqlx::Result<User> {
        let (column, placeholder) = column_placeholder(who);

        let sql = format!(
            "SELECT id, username, email, password_hash, status, email_verified, image_url, created_date, modified_date,
            designation, location, git, website
            FROM users
            where {} = {};",
            column, placeholder
        );

        sqlx::query_as(&sql).bind(who).fetch_one(&self.sql).await
    }
    async fn user_delete(&self, who: &str) -> sqlx::Result<User> {
        let (column, placeholder) = column_placeholder(who);

        let sql = format!(
            "update users set status='deleted' where {}={} RETURNING *;",
            column, placeholder
        );

        sqlx::query_as(&sql).bind(who).fetch_one(&self.sql).await
    }
}

#[cfg(any(feature = "postgres"))]
#[async_trait]
impl IUser for &AppStateRaw {
    async fn user_add(&self, form: &Register) -> sqlx::Result<u64> {
        let passh = form.passhash();
        let email_hash = compute(&form.email.as_bytes());
        // TODO: move it to config
        let image_url =
            "https://www.gravatar.com/avatar/".to_string() + &format!("{:x}", email_hash);
        sqlx::query!(
            r#"
        INSERT INTO users (username, email, password_hash, image_url)
        VALUES ($1 ,$2 ,$3, $4)
                "#,
            form.username,
            form.email,
            passh,
            image_url
        )
        .execute(&self.sql)
        .await
        .map(|d| d.rows_affected())
    }

    async fn get_users(&self, form: &UsersReq) -> sqlx::Result<(UserResponse, i64)> {
        let qr;

        match &form.direction {
            Some(_d) => {
                qr = sqlx::query_as!(
                    UR, r#"
                    select id, username, displayname, name, location, image_url, karma from users where
                    username > $1 order by displayname desc, username limit $2
                    "#,
                    &form.last_user,
                    self.config.users_per_page as i64
                )
                .fetch_all(&self.sql)
                .await?;
            }
            None => {
                qr = sqlx::query_as!(
                    UR, r#"
                    select id, username, displayname, name, location, image_url, karma from users where
                    username > $1 order by displayname asc, username limit $2
                    "#,
                    &form.last_user,
                    self.config.users_per_page as i64
                )
                .fetch_all(&self.sql)
                .await?;
            }
        };

        let count = sqlx::query!(
            r#"
            select count(1) from users
            "#
        )
        .fetch_one(&self.sql)
        .await?;

        let c = match count.count {
            Some(c) => c,
            None => 0,
        };

        let urs = UserResponse { users: qr };
        Ok((urs, c))
    }

    async fn get_profile(&self, uid: &i64) -> sqlx::Result<ProfileResponse> {
        let qr = sqlx::query!(
            r#"
            select id, username, name, title, designation, location, email, image_url, git, website,
            twitter, karma, displayname, created_date from users
            where id = $1
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let name = qr.name;
        let title = match qr.title {
            Some(s) => s,
            None => "".to_owned(),
        };
        let designation = match qr.designation {
            Some(s) => s,
            None => "".to_owned(),
        };
        let location = qr.location;
        let image_url = qr.image_url;
        let git = match qr.git {
            Some(s) => s,
            None => "".to_owned(),
        };
        let website = match qr.website {
            Some(s) => s,
            None => "".to_owned(),
        };
        let twitter = match qr.twitter {
            Some(s) => s,
            None => "".to_owned(),
        };

        let karma = qr.karma;

        let id = qr.id;
        let created_date = qr.created_date;
        let displayname = qr.displayname;

        let p = ProfileResponse {
            id,
            username: qr.username,
            displayname,
            name,
            title,
            designation,
            location,
            image_url,
            git,
            website,
            twitter,
            karma,
            created_date,
            cat: created_date.unix_timestamp(),
        };
        Ok(p)
    }

    async fn update_username(&self, uid: i64, username: &String) -> sqlx::Result<bool> {
        let r = sqlx::query!(
            r#"
            select * from users where username=$1
            "#,
            username
        )
        .fetch_one(&self.sql)
        .await;

        let id = match r {
            Ok(u) => u.id,
            Err(_e) => 0,
        };

        if id != 0 {
            return Ok(false);
        }

        sqlx::query!(
            r#"
            update users set username=$1 where id=$2
            "#,
            username,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn verify_email(&self, email: &str) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set email_verified=true where email=$1
            "#,
            email
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_title(&self, uid: i64, title: &String) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set title=$1 where id=$2
            "#,
            title,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_name(&self, uid: i64, name: &String) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set name=$1 where id=$2
            "#,
            name,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_designation(&self, uid: i64, designation: &String) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set designation=$1 where id=$2
            "#,
            designation,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_location(&self, uid: i64, location: &String) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set location=$1 where id=$2
            "#,
            location,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn get_links(&self, uid: i64) -> sqlx::Result<LinksResponse> {
        let r = sqlx::query!(
            r#"
            select website, git, twitter from users where id=$1
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let website = match r.website {
            Some(s) => s,
            None => "".to_owned(),
        };
        let git = match r.git {
            Some(s) => s,
            None => "".to_owned(),
        };
        let twitter = match r.twitter {
            Some(s) => s,
            None => "".to_owned(),
        };

        let lr = LinksResponse {
            website,
            git,
            twitter,
        };

        Ok(lr)
    }

    async fn update_links(&self, uid: i64, form: &LinksResponse) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set website=$1, git=$2, twitter=$3 where id=$4
            "#,
            form.website,
            form.git,
            form.twitter,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_profile_image(&self, uid: i64, url: &String) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set image_url=$1 where id=$2
            "#,
            url,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn update_profile(&self, uid: &i64, data: &ProfileReq) -> sqlx::Result<bool> {
        sqlx::query!(
            r#"
            update users set designation=$1, username=$2, git=$3, website=$4, location=$5 where id=$6
            "#,
            data.designation,
            data.username,
            data.git,
            data.website,
            data.location,
            uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }

    async fn get_summary(&self, uid: i64) -> sqlx::Result<SummaryResponse> {
        let answers_count = sqlx::query!(
            r#"
            select count(1) as answers_count from posts where posted_by_id=$1 and op_id!=0
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;
        let questions_count = sqlx::query!(
            r#"
            select count(1) as questions_count from posts where posted_by_id=$1 and op_id=0
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let karma = sqlx::query!(
            r#"
            select karma from users where id=$1
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let sr = SummaryResponse {
            answers_count: answers_count.answers_count,
            questions_count: questions_count.questions_count,
            karma: karma.karma,
        };

        Ok(sr)
    }
}

fn column_placeholder(id_or_name_or_email: &str) -> (&'static str, &'static str) {
    let mut column = "username";

    if id_or_name_or_email.contains("@") {
        column = "email";
    } else if first_char_is_number(id_or_name_or_email) {
        column = "username";
    }

    let placeholder = "$1";

    (column, placeholder)
}
