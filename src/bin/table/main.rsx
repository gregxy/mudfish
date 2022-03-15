use clap::{ArgEnum, Subcommand, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "postgres://localhost/mudfish",
        help = "PostgreSQL server URI.")]
    postgres_uri: String,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update to latest version if tables exist, or create new ones if not.
    upsert,

    /// Shows current state of tables.
    show,
}


