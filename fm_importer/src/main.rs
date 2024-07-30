use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    /// Import Source File
    #[clap(short, long)]
    source: String,
    /// Import Format
    #[clap(short, long, default_value = "CSV_CAMT_V2")]
    format: String,
    /// The url to bind to
    #[clap(short, long, default_value = "http://127.0.0.1:3000")]
    url: String,
    /// Verbose mode
    #[clap(short, long, default_value = "false")]
    verbose: bool,
    /// Debug mode
    #[clap(short, long, default_value = "false")]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.verbose || args.debug {
        let stdout_log = tracing_subscriber::fmt::layer().compact();
        tracing_subscriber::registry()
            .with(stdout_log.with_filter(
                tracing_subscriber::filter::Targets::default().with_target(
                    "fm_importer",
                    if args.debug {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::INFO
                    },
                ),
            ))
            .init();
    }

    let finance_manager = Arc::new(Mutex::new(
        fm_core::FMController::<fm_server::client::Client>::new(args.url).unwrap(),
    ));
    match args.format.as_str() {
        "CSV_CAMT_V2" => {
            let data = std::fs::read(&args.source).unwrap();
            let as_utf8 = encoding_rs::ISO_8859_15.decode(&data).0.into_owned();
            let data = std::io::BufReader::new(as_utf8.as_bytes());
            let parser = fm_importer::csv_parser::csv_camt_v2_parser(data).unwrap();
            fm_importer::terminal_importer::run_in_terminal(
                fm_importer::Importer::new(parser, finance_manager.clone())
                    .await
                    .unwrap(),
                finance_manager,
            )
            .await;
        }
        _ => eprintln!("Unknown format: {}", args.format),
    }
}
