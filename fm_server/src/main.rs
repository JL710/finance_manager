fn main() {
    #[cfg(feature = "server")]
    fm_server::server_cli::run();
}
