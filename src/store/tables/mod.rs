use postgres::Client;

pub struct Migration {
    pub test: fn(&mut Client) -> Result<bool, postgres::error::Error>,
    pub apply: fn(&mut Client) -> Result<(), postgres::error::Error>,
}

pub(crate) mod pgn;
