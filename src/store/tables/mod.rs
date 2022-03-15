use postgres::Client;
use simple_error::SimpleResult;

pub struct Migration {
    pub test: fn(&mut Client) -> SimpleResult<bool>,
    pub apply: fn(&mut Client) -> SimpleResult<()>,
}

pub(crate) mod pgn;
