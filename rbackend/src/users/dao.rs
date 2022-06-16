use super::user::*;
use crate::state::AppStateRaw;
use md5::compute;

#[async_trait]
pub trait IUser: std::ops::Deref<Target = AppStateRaw> {
    async fn user_add(&self, form: &Register) -> sqlx::Result<u64>;
    async fn get_users(&self, form: &UsersReq) -> sqlx::Result<UserResponse>;
    async fn get_profile(&self, uid: &i64) -> sqlx::Result<ProfileResponse>;
    async fn user_query(&self, who: &str) -> sqlx::Result<User> {
        let (column, placeholder) = column_placeholder(who);

        let sql = format!(
            "SELECT id, username, email, pass, status, image_url, create_dt, update_dt
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
        INSERT INTO users (username, email, pass, image_url)
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
            let image_url = match q.image_url {
                Some(n) => n,
                None => "".to_owned(),
            };
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
            select username, name, title, designation, location, email, image_url, git, website, twitter, reputation from users
            where id = $1
            "#,
            uid
        )
        .fetch_one(&self.sql)
        .await;

        let p = ProfileResponse {
            username: qr.username,
            name: qr.name,
            title: qr.title,
            designation: qr.designation,
            location: qr.location,
            image_url: qr.image_url,
            git: qr.git,
            website: qr.website,
            twitter: qr.twitter,
            karma: qr.karma,
        };
        Ok(p)
    }
}

fn column_placeholder(id_or_name_or_email: &str) -> (&'static str, &'static str) {
    let mut column = "name";

    if id_or_name_or_email.contains("@") {
        column = "email";
    } else if first_char_is_number(id_or_name_or_email) {
        column = "id";
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
