use std::path::Path;

use clap::{Args, Parser, Subcommand};

use mudfish::pgn::{PgnReader, ReadOutcome};
use mudfish::store::PostgresStore;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stores PGNs to databse.
    Store(StoreCommandArgs),

    /// Prints PGNs.
    Cat(CatCommandArgs),

    /// Validates PGNs
    Validate,

    /// Count number of PGNs in the PGN file.
    Count,
}

#[derive(Args, Debug)]
struct CatCommandArgs {
    #[clap(long, default_value_t = 0)]
    start: usize,

    #[clap(long, default_value_t = 0)]
    end: usize,
}

#[derive(Args, Debug)]
struct StoreCommandArgs {
    #[clap(long, default_value = "postgres://localhost/mudfish")]
    postgres_uri: String,

    #[clap(long, default_value_t = 0)]
    start: usize,

    #[clap(long, default_value_t = 0)]
    end: usize,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct App {
    #[clap(subcommand)]
    command: Commands,

    pgnfile: String,
}

fn store_pgn(
    args: &StoreCommandArgs,
    mut reader: PgnReader,
) -> Result<(), Box<dyn std::error::Error>> {
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
            ReadOutcome::Ended => return Ok(()),
            ReadOutcome::Error(err) => return Err(Box::new(err)),
        }
    }
}

fn cat_pgn(args: &CatCommandArgs, mut reader: PgnReader) -> Result<(), Box<dyn std::error::Error>> {
    let mut count: usize = 0;
    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;
                if count < args.start {
                    continue;
                }

                println!("{}\n\n{}\n{}\n", pgn.id, pgn.tags_text, pgn.moves_text);

                if args.end > 0 && count >= args.end {
                    return Ok(());
                }
            }
            ReadOutcome::Ended => return Ok(()),
            ReadOutcome::Error(err) => return Err(Box::new(err)),
        }
    }
}

fn validate_pgn(
    print_count: bool,
    mut reader: PgnReader,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut count: usize = 0;
    loop {
        match reader.read_next() {
            ReadOutcome::Game(_pgn) => {
                count += 1;
            }
            ReadOutcome::Ended => {
                if print_count {
                    println!("{}", count);
                }
                return Ok(());
            }
            ReadOutcome::Error(err) => return Err(Box::new(err)),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::parse();

    let p = Path::new(app.pgnfile.as_str());
    let reader = PgnReader::new(p)?;

    match &app.command {
        Commands::Store(args) => store_pgn(args, reader),
        Commands::Cat(args) => cat_pgn(args, reader),
        Commands::Validate => validate_pgn(false, reader),
        Commands::Count => validate_pgn(true, reader),
    }
}
