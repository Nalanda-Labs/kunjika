// PBKDF2 < bcrypt < scrypt < argon2
use argon2::{self, Config};
use ring::digest;
use time;

fn passhash(pass: &str) -> String {
    let config = Config::default();
    const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
    let salt = [0u8; CREDENTIAL_LEN];
    let hash = argon2::hash_encoded(pass.as_bytes(), &salt, &config).unwrap();
    // info!("{}{}: {}", name, pass, hash);
    hash
}
fn passhash_verify(pass: &str, hash: &str) -> bool {
    argon2::verify_encoded(&hash, pass.as_bytes()).unwrap()
}

#[cfg(any(feature = "postgres"))]
type SqlID = i64;

type SqlDateTime = time::OffsetDateTime;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: SqlID,
    pub username: String,
    // pub phone: String,
    pub email: String,
    // not return password
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub password_hash: String,
    pub status: String,
    pub image_url: String,
    pub email_verified: Option<bool>,
    pub created_date: SqlDateTime,
    pub modified_date: SqlDateTime,
    pub designation: Option<String>,
    pub location: Option<String>,
    pub git: Option<String>,
    pub website: Option<String>,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct UserCookie {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl Login {
    pub fn verify(&self, hash: &str) -> bool {
        passhash_verify(&self.password, hash)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Claims {
    // username
    pub sub: String,
    pub exp: usize,
    pub email: String,
    pub username: String,
    pub id: i64,
    pub xsrf_token: String,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct Register {
    #[validate(length(min = 3, max = 33), custom(function = "validate_username"))]
    pub username: String,
    #[validate(length(min = 6, max = 256), email)]
    pub email: String,
    #[validate(length(min = 16, max = 64))]
    pub password: String,
    #[validate(length(min = 16, max = 64))]
    pub confirm_password: String,
}

use validator::ValidationError;
fn validate_username(username: &str) -> Result<(), ValidationError> {
    // todo: use regex for robust
    if first_char_is_number(username) {
        return Err(ValidationError::new(
            "terrible_username: first char is number",
        ));
    }

    if username.contains("@") {
        // the value of the username will automatically be added later
        return Err(ValidationError::new("terrible_username: contains @"));
    }

    Ok(())
}

impl Register {
    pub fn passhash(&self) -> String {
        passhash(&self.password)
    }
    pub fn match_password(&self) -> bool {
        self.password == self.confirm_password
    }
}

pub fn first_char_is_number(s: &str) -> bool {
    s.get(0..1).and_then(|c| c.parse::<u8>().ok()).is_some()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailabilityResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserName {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsersReq {
    pub last_user: String,
    pub direction: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UR {
    pub id: SqlID,
    pub username: String,
    pub displayname: String,
    pub name: String,
    pub location: String,
    pub image_url: String,
    pub karma: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub users: Vec<UR>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileResponse {
    pub id: SqlID,
    pub username: String,
    pub displayname: String,
    pub name: String,
    pub title: String,
    pub designation: String,
    pub location: String,
    pub image_url: String,
    pub git: String,
    pub website: String,
    pub twitter: String,
    pub karma: i64,
    pub created_date: SqlDateTime,
    pub cat: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileReq {
    pub username: String,
    pub designation: String,
    pub location: String,
    pub git: String,
    pub website: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinksResponse {
    pub website: String,
    pub git: String,
    pub twitter: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub user: User,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SummaryResponse {
    pub answers_count: Option<i64>,
    pub questions_count: Option<i64>,
    pub karma: i64,
}
