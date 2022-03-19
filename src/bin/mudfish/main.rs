use std::path::Path;

use clap::{Args, Parser, Subcommand};
use simple_error::simple_error;

use mudfish::pgn::{ReadOutcome, Reader};
use mudfish::store::PostgresStore;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stores PGNs to databse.
    StorePgn(StorePgnArgs),

    /// Reads PGNs.
    ReadPgn(ReadPgnArgs),
}

#[derive(Args, Debug)]
struct ReadPgnArgs {
    #[clap(long, default_value_t = 0)]
    start: usize,

    #[clap(long, default_value_t = 0)]
    end: usize,

    #[clap(short, long)]
    print: bool,

    #[clap(short, long)]
    count: bool,

    pgnfile: String,
}

#[derive(Args, Debug)]
struct StorePgnArgs {
    #[clap(long, default_value = "postgres://localhost/mudfish")]
    postgres_uri: String,

    #[clap(long, default_value_t = 0)]
    start: usize,

    #[clap(long, default_value_t = 0)]
    end: usize,

    #[clap(short, long)]
    count: bool,

    pgnfile: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    command: Commands,
}

fn store_pgn(args: &StorePgnArgs) -> Result<(), Box<dyn std::error::Error>> {
    let p = Path::new(args.pgnfile.as_str());
    let mut reader = Reader::new(p)?;

    let mut store = PostgresStore::open(args.postgres_uri.as_str())?;

    let mut count: usize = 0;
    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;
                if count < args.start {
                    continue;
                }

                if let Err(err) = store.upsert_pgn(&pgn) {
                    return Err(Box::new(err));
                }

                if args.end > 0 && count >= args.end {
                    return Ok(());
                }
            }
            ReadOutcome::Ended => {
                if args.count {
                    println!("{}", count);
                }
                return Ok(());
            }
            ReadOutcome::BadPgn(message) => {
                println!("{}", message);
                continue;
            }
            ReadOutcome::Error(message) => return Err(Box::new(simple_error!(message))),
        }
    }
}

fn read_pgn(args: &ReadPgnArgs) -> Result<(), Box<dyn std::error::Error>> {
    let p = Path::new(args.pgnfile.as_str());
    let mut reader = Reader::new(p)?;

    let mut count: usize = 0;
    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;
                if count < args.start {
                    continue;
                }

                if args.print {
                    println!("{}\n\n{}\n{}\n", pgn.id, pgn.tags_text, pgn.moves_text);
                }

                if args.end > 0 && count >= args.end {
                    return Ok(());
                }
            }
            ReadOutcome::Ended => return Ok(()),
            ReadOutcome::BadPgn(message) => {
                println!("{}", message);
                continue;
            }
            ReadOutcome::Error(message) => return Err(Box::new(simple_error!(message))),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::parse();

    match &app.command {
        Commands::StorePgn(args) => store_pgn(args),
        Commands::ReadPgn(args) => read_pgn(args),
    }
}
