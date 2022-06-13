#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct User {
    pub id: SqlID,
    pub name: String,
    // pub phone: String,
    pub email: String,
    // not return password
    #[serde(skip_serializing)]
    pub pass: String,
    pub status: String,
    pub create_dt: SqlDateTime,
    pub update_dt: SqlDateTime,
}