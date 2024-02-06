use super::user::*;
use crate::state::AppStateRaw;
use md5::compute;

#[async_trait]
pub trait IUser: std::ops::Deref<Target = AppStateRaw> {
    async fn user_add(&self, form: &Register) -> sqlx::Result<u64>;
    async fn get_users(&self, form: &UsersReq) -> sqlx::Result<UserResponse>;
    async fn get_profile(&self, uid: &i64) -> sqlx::Result<ProfileResponse>;
    async fn update_username(&self, uid: i64, username: &String) -> sqlx::Result<bool>;
    async fn update_title(&self, uid: i64, title: &String) -> sqlx::Result<bool>;
    async fn update_name(&self, uid: i64, name: &String) -> sqlx::Result<bool>;
    async fn update_designation(&self, uid: i64, designation: &String) -> sqlx::Result<bool>;
    async fn update_location(&self, uid: i64, location: &String) -> sqlx::Result<bool>;
    async fn get_links(&self, uid: i64) -> sqlx::Result<LinksResponse>;
    async fn update_links(&self, uid: i64, form: &LinksResponse) -> sqlx::Result<bool>;
    async fn verify_email(&self, who: &str) -> sqlx::Result<bool>;
    async fn user_query(&self, who: &str) -> sqlx::Result<User> {
        let (column, placeholder) = column_placeholder(who);

        let sql = format!(
            "SELECT id, username, email, password_hash, status, email_verified, image_url, created_date, modified_date
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

    async fn get_users(&self, form: &UsersReq) -> sqlx::Result<UserResponse> {
        let qr = sqlx::query!(
            r#"
        select id, username, name, location, image_url from users where
        username > $1 order by karma desc, username limit $2
        "#,
            &form.last_user,
            self.config.users_per_page as i64
        )
        .fetch_all(&self.sql)
        .await?;

        let mut urs: UserResponse = UserResponse { users: Vec::new() };
        for q in qr {
            let name = match q.name {
                Some(n) => n,
                None => "".to_owned(),
            };
            let location = match q.location {
                Some(n) => n,
                None => "".to_owned(),
            };
            let image_url = q.image_url;
            let ur: UR = UR {
                id: q.id.to_string(),
                username: q.username,
                name,
                location,
                image_url,
            };
            urs.users.push(ur);
        }

        Ok(urs)
    }

    async fn get_profile(&self, uid: &i64) -> sqlx::Result<ProfileResponse> {
        let qr = sqlx::query!(
            r#"
            select username, name, title, designation, location, email, image_url, git, website, twitter, karma from users
            where id = $1
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await?;

        let name = match qr.name {
            Some(s) => s,
            None => "".to_owned(),
        };
        let title = match qr.title {
            Some(s) => s,
            None => "".to_owned(),
        };
        let designation = match qr.designation {
            Some(s) => s,
            None => "".to_owned(),
        };
        let location = match qr.location {
            Some(s) => s,
            None => "".to_owned(),
        };
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
        let karma = match qr.karma {
            Some(s) => s.to_string(),
            None => 1.to_string(),
        };
        let p = ProfileResponse {
            username: qr.username,
            name,
            title,
            designation,
            location,
            image_url,
            git,
            website,
            twitter,
            karma,
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
            form.website, form.git, form.twitter, uid
        )
        .execute(&self.sql)
        .await?;

        Ok(true)
    }
}

fn column_placeholder(id_or_name_or_email: &str) -> (&'static str, &'static str) {
    let mut column = "name";

    if id_or_name_or_email.contains("@") {
        column = "email";
    } else if first_char_is_number(id_or_name_or_email) {
        column = "username";
    }

    // postgres: $1, $2 ..
    // mysql/sqlite: ?, ? ..
    let placeholder = if cfg!(feature = "postgres") {
        "$1"
    } else {
        "?"
    };

    (column, placeholder)
}
