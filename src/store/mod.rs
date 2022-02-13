use crate::pgn::RawPgn;
use simple_error::SimpleResult;

pub trait SavePgn {
    fn upsert_pgn(&mut self, name: &str, pgn: &RawPgn) -> SimpleResult<()>;
}

mod sqlite;
pub use sqlite::SqliteStore;

mod postgres;
pub use self::postgres::PostgresStore;
