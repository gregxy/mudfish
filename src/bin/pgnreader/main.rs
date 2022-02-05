use mudfish::{PgnReader, ReadOutcome};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    pgnfile: String,

    #[clap(short, long)]
    count: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut reader = PgnReader::new(args.pgnfile)?;
    let mut count: u64 = 0;

    loop {
        match reader.read_next() {
            ReadOutcome::Game(pgn) => {
                count += 1;
                if !args.count {
                    println!("{}:", count);
                    println!("{:?}", pgn);
                }
            }
            ReadOutcome::Ended => {
                if args.count {
                    println!("{}", count);
                }
                return Ok(());
            }
            ReadOutcome::EndedUnexpectedly => {
                println!("End unexpectedly.");
                if args.count {
                    println!("{}", count);
                }
                return Ok(());
            }
            ReadOutcome::IoError(err) => return Err(Box::new(err)),
            ReadOutcome::ParseError(err) => return Err(Box::new(err)),
        }
    }
}