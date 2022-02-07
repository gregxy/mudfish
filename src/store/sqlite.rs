use rusqlite::Connection;
use simple_error::simple_error;
use simple_error::SimpleResult;
use std::path::Path;

// use crate::pgn::RawPgn;
// use crate::store::SavePgn;

pub struct SqliteStore {
    table: String,
    connection: Connection,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>, S: Into<String>>(path: P, table: S) -> SimpleResult<Self> {
        let connection = Connection::open(path).map_err(|e| simple_error!(e.to_string()))?;

        let store = Self {
            table: table.into(),
            connection,
        };

        if let Err(err) = store.create_table() {
            return Err(err);
        }

        Ok(store)
    }

    pub fn open_in_memory<S: Into<String>>(table: S) -> SimpleResult<Self> {
        let connection = Connection::open_in_memory().map_err(|e| simple_error!(e.to_string()))?;

        let store = Self {
            table: table.into(),
            connection,
        };

        if let Err(err) = store.create_table() {
            return Err(err);
        }

        Ok(store)
    }

    fn create_table(&self) -> SimpleResult<()> {
        return self
            .connection
            .execute(
                "CREATE TABLE IF NOT EXISTS ? (
                    id          VARCHAR(255)    NOT NULL PRIMARY KEY,
                    event       TEXT            DEFAULT '',
                    site        TEXT            DEFAULT '',
                    round       TEXT            DEFAULT '',
                    date        VARCHAR(31)     DEFAULT '',
                    time        VARCHAR(31)     DEFAULT '',
                    white       VARCHAR(255)    NOT NULL,
                    white_title VARCHAR(7)      DEFAULT '',
                    white_elo   INT             DEFAULT 0,
                    white_fide  INT             DEFAULT 0,
                    black       VARCHAR(255)    NOT NULL,
                    black_title VARCHAR(7)      DEFAULT '',
                    black_elo   INT             DEFAULT 0,
                    black_fide  INT             DEFAULT 0,
                    eco         VARCHAR(7)      DEFAULT '',
                    opening     TEXT            DEFAULT '',
                    variation   TEXT            DEFAULT '',
                    result      VARCHAR(15)     DEFAULT '',
                    moves       TEXT            NOT NULL
                )",
                [self.table.as_str()],
            )
            .map(|_| ())
            .map_err(|err| simple_error!(err.to_string()));
    }
}
