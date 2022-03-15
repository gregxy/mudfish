use super::Migration;

use simple_error::simple_error;

pub fn get_migrations() -> Vec<Migration> {
    return vec![Migration {
        test: |client| {
            let statement = "
    				SELECT FROM pg_tables
    				WHERE schemaname = 'public' AND tablename  = 'pgn'";

            client
                .query_opt(statement, &[])
                .map(|opt| opt.is_some())
                .map_err(|e| simple_error!(e.to_string()))
        },
        apply: |client| {
            let statement = "CREATE TABLE pgn (
		                id          VARCHAR(255)    NOT NULL PRIMARY KEY,
	                    event       TEXT            DEFAULT '',
	                    site        TEXT            DEFAULT '',
	                    round       TEXT            DEFAULT '',
	                    date        VARCHAR(31)     DEFAULT '',
	                    time        VARCHAR(31)     DEFAULT '',
	                    time_control    VARCHAR(63)	DEFAULT '',
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
	                    tags        TEXT            NOT NULL,
	                    moves       TEXT            NOT NULL)";
            client
                .execute(statement, &[])
                .map(|_| ())
                .map_err(|e| simple_error!(e.to_string()))
        },
    }];
}
