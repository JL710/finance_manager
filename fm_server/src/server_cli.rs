use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// The database file to use
    #[clap(short, long, default_value = "fm.db")]
    db: String,
    /// The url to bind to
    #[clap(short, long, default_value = "127.0.0.1:3000")]
    url: String,
}

pub fn run() {
    let args = Args::parse();
    super::server::run(args.url, args.db);
}
