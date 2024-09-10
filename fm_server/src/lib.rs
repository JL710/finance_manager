#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server_cli;

#[cfg(feature = "server")]
pub mod server;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
struct Tokenized<T: Clone> {
    token: String,
    content: T,
}
