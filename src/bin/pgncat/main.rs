use std::path::Path;

use mudfish::store::PostgresStore;

use mudfish::{PgnReader, ReadOutcome};

use clap::{ArgEnum, Parser};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Output {
    None,
    Stdout,
    Postgres,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    pgnfile: String,

    #[clap(long, arg_enum)]
    output: Output,

    #[clap(long, default_value = "postgres://localhost/mudfish")]
    postgres_uri: String,

    #[clap(long, default_value = "pgn")]
    postgres_table: String,

    #[clap(long, default_value_t = 0)]
    start: usize,

    #[clap(long, default_value_t = 0)]
    end: usize,

    #[clap(long)]
    count: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let p = Path::new(args.pgnfile.as_str());

    let mut reader = PgnReader::new(p)?;
    let mut count: usize = 0;

    let mut store_opt: Option<PostgresStore> = if args.output == Output::Postgres {
        PostgresStore::open(args.postgres_uri.as_str(), args.postgres_table).map(Some)?
    } else {
        None
    };

    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;

                if args.end > 0 && count > args.end {
                    if args.count {
                        println!("{}", count);
                    }
                    return Ok(());
                }

                if count < args.start {
                    continue;
                }

                match args.output {
                    Output::None => continue,
                    Output::Stdout => println!("{}\n\n{}\n{}\n", pgn.id, pgn.tags_text, pgn.moves_text),
                    Output::Postgres => {
                        if let Some(ref mut store) = store_opt {
                            if let Err(err) = store.upsert_pgn(&pgn) {
                                return Err(Box::new(err));
                            }
                        }
                    }
                }
            }
            ReadOutcome::Ended => {
                if args.count {
                    println!("{}", count);
                }
                return Ok(());
            }
            ReadOutcome::Error(err) => return Err(Box::new(err)),
        }
    }
}
