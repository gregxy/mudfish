use std::str::FromStr;

use postgres::{Client, NoTls};
use simple_error::simple_error;
use simple_error::SimpleResult;

use super::tables;
use crate::pgn::RawPgn;

pub struct PostgresStore {
    client: Client,
    empty: String,
}

impl PostgresStore {
    pub fn open(target: &str) -> SimpleResult<Self> {
        let mut config =
            postgres::config::Config::from_str(target).map_err(|e| simple_error!(e.to_string()))?;
        if config.get_user().is_none() {
            config.user(whoami::username().as_str());
        }

        let client = config
            .connect(NoTls)
            .map_err(|e| simple_error!(e.to_string()))?;

        let mut store = Self {
            client,
            empty: String::new(),
        };

        store.create_tables()?;

        Ok(store)
    }

    fn create_tables(&mut self) -> SimpleResult<()> {
        for migration in tables::pgn::get_migrations() {
            let done = (migration.test)(&mut self.client)?;
            if !done {
                (migration.apply)(&mut self.client)?;
            }
        }
        Ok(())
    }

    pub fn upsert_pgn(&mut self, pgn: &RawPgn) -> SimpleResult<()> {
        let statement = "INSERT INTO pgn (
                id,
                event,
                site,
                round,
                date,
                time,
                time_control,
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
                tags,
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
                $19,
                $20,
                $21)
            ON CONFLICT (id) DO UPDATE SET
                event = $2,
                site = $3,
                round = $4,
                date = $5,
                time = $6,
                time_control = $7,
                white = $8,
                white_title = $9,
                white_elo = $10,
                white_fide = $11,
                black = $12,
                black_title = $13,
                black_elo = $14,
                black_fide = $15,
                eco = $16,
                opening = $17,
                variation = $18,
                result = $19,
                tags = $20,
                moves = $21";

        return self
            .client
            .execute(
                statement,
                &[
                    &pgn.id,
                    pgn.tags.get("Event").unwrap_or(&self.empty),
                    pgn.tags.get("Site").unwrap_or(&self.empty),
                    pgn.tags.get("Round").unwrap_or(&self.empty),
                    pgn.tags.get("Date").unwrap_or(&self.empty),
                    pgn.tags.get("UTCTime").unwrap_or(&self.empty),
                    pgn.tags.get("TimeControl").unwrap_or(&self.empty),
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
                    &pgn.tags_text,
                    &pgn.moves_text,
                ],
            )
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
