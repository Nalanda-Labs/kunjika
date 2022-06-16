// PBKDF2 < bcrypt < scrypt < argon2
use argon2::{self, Config};
use chrono::Utc;
use ring::digest;

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

type SqlDateTime = chrono::DateTime<Utc>;

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: SqlID,
    pub username: String,
    // pub phone: String,
    pub email: String,
    // not return password
    #[serde(skip_serializing)]
    pub pass: String,
    pub status: String,
    pub image_url: String,
    pub create_dt: SqlDateTime,
    pub update_dt: SqlDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub rememberme: bool,
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
    pub image_url: String
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct Register {
    #[validate(length(min = 3, max = 33), custom = "validate_username")]
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UR {
    pub id: String,
    pub username: String,
    pub name: String,
    pub location: String,
    pub image_url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub users: Vec<UR>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileResponse {
    pub username: String,
    pub name: String,
    pub title: String,
    pub designation: String,
    pub location: String,
    pub image_url: String,
    pub git: String,
    pub website: String,
    pub twitter: String,
    pub karma: String,
}
