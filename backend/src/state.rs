pub use mobc_redis::{redis, RedisConnectionManager};
pub type Connection = mobc::Connection<RedisConnectionManager>;
pub type KvPool = mobc::Pool<RedisConnectionManager>;

#[cfg(any(feature = "postgres"))]
pub type SqlPool = sqlx::PgPool;
#[cfg(any(feature = "postgres"))]
pub type PoolOptions = sqlx::postgres::PgPoolOptions;

use crate::config::Config;

#[derive(Clone)]
pub struct State {
    pub config: Config,
    pub sql: SqlPool,
    pub kv: KvPool,
}

pub type AppStateRaw = std::sync::Arc<State>;
pub type AppState = ntex::web::types::State<AppStateRaw>;
