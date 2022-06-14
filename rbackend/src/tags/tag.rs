use chrono::Utc;

#[cfg(any(feature = "postgres"))]
type SqlID = i64;

type SqlDateTime = chrono::DateTime<Utc>;

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct Tag {
    pub id: SqlID,
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    pub info: Option<String>,
    pub create_dt: Option<SqlDateTime>,
    pub update_dt: Option<SqlDateTime>,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct  TagRequest {
    // #[validate(length(min = 1, max = 32))]
    pub tag: String,
}