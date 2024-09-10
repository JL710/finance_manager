<div align="center">
    <img src="FM_Logo.svg" width="300px" />
</div>

# Finance Manager

Finance Manager is a program, made with the Rust programming language, for managing your private finances.

Finance Manager is a highly composable system with components for client-server usage, a GUI that runs natively and on the web, and a component for importing data from CSV files.

## Crates
### Core
The core logic is based in the `fm_core` library crate.

It contains the `FinanceManager` trait, which is used to store the data. This crate contains some basic implementations, such as SQLite and RAM-stored data.
The `FMController` provides a secure interface for implementors of `FinanceManager` that handles checks and additional functionalities.

### Server
A Server and Client that communicate via a REST API are available via the `fm_server` crate. 
It provides a backend based on Axum and a client that implements the `FinanceManager` trait from `fm_core`.

#### Run the backend
For the safety and security of your data, make sure to use HTTPS/TLS!

The API will try to prevent brute force attacks on the token, with timeouts after too many wrong tokens.

```
cd fm_server
cargo run <api-token>
```
> `cargo run -- --help` for help

### Importer
To import data into the finance manager, you can use the `fm_importer` crate. It provides a mechanic to import data from formats such as CSV.

### GUI
The `fm_gui` crate provides graphical access to the financial data.

It runs natively and as Wasm on the web. Although current issues with iced on Wasm make Wasm as platform unusable until the bug is fixed. 

#### Run the GUI
Run the GUI natively with SQLite support:
```
cd fm_gui
cargo run --no-default-features
```
Run the GUI natively without SQLite support:
```
cd fm_gui
cargo run
```

## Installing
Use the commands from the `Run the GUI` section and replace `cargo run` with `cargo install`.
