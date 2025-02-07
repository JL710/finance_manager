use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// The API token to use/accept
    token: String,
    /// The database file to use
    #[clap(short, long, default_value = "fm.db")]
    db: String,
    /// The url to bind to
    #[clap(short, long, default_value = "127.0.0.1:3000")]
    url: String,
}

#[tokio::main]
async fn tokio_run(url: String, db: Option<String>, token: String) {
    super::server::init_subscriber();
    super::server::run(url, db, token).await;
}

pub fn run() {
    let args = Args::parse();
    tokio_run(args.url, Some(args.db), args.token);
}
