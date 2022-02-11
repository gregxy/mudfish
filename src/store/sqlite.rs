use std::path::Path;

use rusqlite::params;
use rusqlite::Connection;
use simple_error::simple_error;
use simple_error::SimpleResult;

use crate::pgn::RawPgn;
use crate::store::SavePgn;

pub struct SqliteStore {
    table: String,
    connection: Connection,
    empty: String,
    insert_statement: String,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>, S: Into<String>>(path: P, table: S) -> SimpleResult<Self> {
        let connection = Connection::open(path).map_err(|e| simple_error!(e.to_string()))?;

        SqliteStore::internal_open(table, connection)
    }

    pub fn open_in_memory<S: Into<String>>(table: S) -> SimpleResult<Self> {
        let connection = Connection::open_in_memory().map_err(|e| simple_error!(e.to_string()))?;

        SqliteStore::internal_open(table, connection)
    }

    fn internal_open<S: Into<String>>(table: S, connection: Connection) -> SimpleResult<Self> {
        let mut store = Self {
            table: table.into(),
            connection,
            empty: String::new(),
            insert_statement: String::new(),
        };

        store.construct_statements();

        if let Err(err) = store.create_table() {
            return Err(err);
        }

        Ok(store)
    }

    fn construct_statements(&mut self) {
        self.insert_statement = format!(
            "INSERT INTO {} (
    			id,
    			event,
    			site,
    			round,
    			date,
    			time,
    			white,
    			white_title,
    			white_elo,
    			white_fide,
    			black,
                black_title,
                black_elo,
    			black_fide,
    			eco,
    			opening,
    			variation,
                result,
    			moves
    		)
    		VALUES(
    			?1,
    			?2,
    			?3,
    			?4,
    			?5,
    			?6,
    			?7,
    			?8,
    			?9,
    			?10,
    			?11,
    			?12,
    			?13,
    			?14,
    			?15,
    			?16,
                ?17,
                ?18,
                ?19)",
            self.table.as_str()
        );
    }

    fn create_table(&self) -> SimpleResult<()> {
        let statement = format!(
            "CREATE TABLE IF NOT EXISTS {} (
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
            self.table.as_str()
        );

        return self
            .connection
            .execute(statement.as_str(), [])
            .map(|_| ())
            .map_err(|err| simple_error!(err.to_string()));
    }
}

fn parse_to_number(o: Option<&String>) -> i32 {
    if let Some(n) = o {
        return n.parse().unwrap_or(0);
    }

    0
}

impl SavePgn for SqliteStore {
    fn upsert_pgn<S: Into<String>>(&self, name: S, pgn: &RawPgn) -> SimpleResult<()> {
        return self
            .connection
            .execute(
                self.insert_statement.as_str(),
                params![
                    name.into(),
                    pgn.tags.get("Event").unwrap_or(&self.empty),
                    pgn.tags.get("Site").unwrap_or(&self.empty),
                    pgn.tags.get("Round").unwrap_or(&self.empty),
                    pgn.tags.get("Date").unwrap_or(&self.empty),
                    pgn.tags.get("Time").unwrap_or(&self.empty),
                    pgn.tags.get("White").unwrap_or(&self.empty),
                    pgn.tags.get("WhiteTitle").unwrap_or(&self.empty),
                    parse_to_number(pgn.tags.get("WhiteElo")),
                    parse_to_number(pgn.tags.get("WhiteFideId")),
                    pgn.tags.get("Black").unwrap_or(&self.empty),
                    pgn.tags.get("BlackTitle").unwrap_or(&self.empty),
                    parse_to_number(pgn.tags.get("BlackElo")),
                    parse_to_number(pgn.tags.get("BlackFideId")),
                    pgn.tags.get("ECO").unwrap_or(&self.empty),
                    pgn.tags.get("Opening").unwrap_or(&self.empty),
                    pgn.tags.get("Variation").unwrap_or(&self.empty),
                    pgn.tags.get("Result").unwrap_or(&self.empty),
                    pgn.moves,
                ],
            )
            .map(|_| ())
            .map_err(|err| simple_error!(err.to_string()));
    }
}
