use clap::Parser;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    api_token: String,
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

    let finance_controller =
        fm_core::FMController::<fm_server::client::Client>::new((args.url, args.api_token))
            .unwrap();
    match args.format.as_str() {
        "CSV_CAMT_V2" => {
            let data = fm_importer::csv_parser::csv_camt_v2_data(args.source);
            let parser = fm_importer::csv_parser::csv_camt_v2_parser(data).unwrap();
            fm_importer::terminal_importer::run_in_terminal(
                fm_importer::Importer::new(parser, finance_controller.clone())
                    .await
                    .unwrap(),
                finance_controller,
            )
            .await
            .unwrap();
        }
        _ => eprintln!("Unknown format: {}", args.format),
    }
}
