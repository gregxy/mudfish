use crate::pgn::RawPgn;
use simple_error::SimpleResult;

pub trait SavePgn {
    fn upsert_pgn<S: Into<String>>(&self, name: S, pgn: &RawPgn) -> SimpleResult<()>;
}

mod sqlite;
pub use sqlite::SqliteStore;
