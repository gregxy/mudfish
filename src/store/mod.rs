use crate::pgn::RawPgn;
use simple_error::SimpleResult;

pub trait SavePgn {
    fn upsert_pgn(&self, name: &str, pgn: &RawPgn) -> SimpleResult<()>;
}

mod sqlite;
pub use sqlite::SqliteStore;