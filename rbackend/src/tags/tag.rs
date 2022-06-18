use chrono::Utc;

#[cfg(any(feature = "postgres"))]
type SqlID = i64;

type SqlDateTime = chrono::DateTime<Utc>;

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct Tag {
    pub id: SqlID,
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    pub post_count: Option<i64>,
    pub info: Option<String>,
    pub create_dt: SqlDateTime,
    pub update_dt: SqlDateTime,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct TagRequest {
    // #[validate(length(min = 1, max = 32))]
    pub tag: String,
    pub post_count: SqlID
}

#[derive(FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct TagInfoRequest {
    // #[validate(length(min = 20, max = 1000000))]
    pub tag_info: String
}
