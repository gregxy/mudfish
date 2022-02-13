use postgres::{Client, NoTls};

use crate::pgn::RawPgn;
use crate::store::SavePgn;
use simple_error::simple_error;
use simple_error::SimpleResult;

pub struct PostgresStore {
    client: Client,
    table: String,
    empty: String,
    insert_statement: String,
}

impl PostgresStore {
    pub fn open<S: Into<String>>(target: &str, table: S) -> SimpleResult<Self> {
        let client = Client::connect(target, NoTls).map_err(|e| simple_error!(e.to_string()))?;

        let mut store = Self {
            client,
            table: table.into(),
            empty: String::new(),
            insert_statement: String::new(),
        };

        store.create_table()?;

        store.construct_statements();

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
    			$1,
    			$2,
    			$3,
    			$4,
    			$5,
    			$6,
    			$7,
    			$8,
    			$9,
    			$10,
    			$11,
    			$12,
    			$13,
    			$14,
    			$15,
    			$16,
                $17,
                $18,
                $19)
            ON CONFLICT (id) DO UPDATE SET
            	event = $2,
            	site = $3,
            	round = $4,
    			date = $5,
    			time = $6,
    			white = $7,
    			white_title = $8,
    			white_elo = $9,
    			white_fide = $10,
    			black = $11,
                black_title = $12,
                black_elo = $13,
    			black_fide = $14,
    			eco = $15,
    			opening = $16,
    			variation = $17,
                result = $18,
    			moves = $19",
            self.table.as_str()
        );
    }

    fn create_table(&mut self) -> SimpleResult<()> {
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
            .client
            .execute(statement.as_str(), &[])
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

impl SavePgn for PostgresStore {
    fn upsert_pgn(&mut self, name: &str, pgn: &RawPgn) -> SimpleResult<()> {
        return self
            .client
            .execute(
                self.insert_statement.as_str(),
                &[
                    &name.to_string(),
                    pgn.tags.get("Event").unwrap_or(&self.empty),
                    pgn.tags.get("Site").unwrap_or(&self.empty),
                    pgn.tags.get("Round").unwrap_or(&self.empty),
                    pgn.tags.get("Date").unwrap_or(&self.empty),
                    pgn.tags.get("Time").unwrap_or(&self.empty),
                    pgn.tags.get("White").unwrap_or(&self.empty),
                    pgn.tags.get("WhiteTitle").unwrap_or(&self.empty),
                    &parse_to_number(pgn.tags.get("WhiteElo")),
                    &parse_to_number(pgn.tags.get("WhiteFideId")),
                    pgn.tags.get("Black").unwrap_or(&self.empty),
                    pgn.tags.get("BlackTitle").unwrap_or(&self.empty),
                    &parse_to_number(pgn.tags.get("BlackElo")),
                    &parse_to_number(pgn.tags.get("BlackFideId")),
                    pgn.tags.get("ECO").unwrap_or(&self.empty),
                    pgn.tags.get("Opening").unwrap_or(&self.empty),
                    pgn.tags.get("Variation").unwrap_or(&self.empty),
                    pgn.tags.get("Result").unwrap_or(&self.empty),
                    &pgn.moves,
                ],
            )
            .map(|_| ())
            .map_err(|err| simple_error!(err.to_string()));
    }
}
