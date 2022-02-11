use std::path::Path;

use mudfish::{PgnReader, ReadOutcome};
use mudfish::store::{SqliteStore, SavePgn};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    pgnfile: String,

    target: String,

    #[clap(long, default_value = "pgn")]
    table: String,

    #[clap(long, default_value_t = 0)]
    skip_first: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let p = Path::new(args.pgnfile.as_str());
    let name = String::from(p.file_stem().unwrap().to_str().unwrap());

    let mut reader = PgnReader::new(args.pgnfile.as_str())?;
    let mut count: usize = 0;

    let store = SqliteStore::open(args.target.as_str(), args.table)?;
    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;
                if count > args.skip_first {
                    if let Err(err) = store.upsert_pgn(format!("{}.{}", name, count), &pgn) {
                        return Err(Box::new(err));
                    }
                }
            }
            ReadOutcome::Ended => {
                println!("{}", count);
                return Ok(());
            }
            ReadOutcome::EndedUnexpectedly => {
                println!("End unexpectedly.");
                println!("{}", count);
                return Ok(());
            }
            ReadOutcome::IoError(err) => return Err(Box::new(err)),
            ReadOutcome::ParseError(err) => return Err(Box::new(err)),
        }
    }
}
