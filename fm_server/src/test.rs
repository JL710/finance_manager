#[cfg(test)]
mod test {
    use crate::client::Client;
    use crate::server::run_with_listener;
    use fm_core::FinanceManager;
    use tokio::net::TcpListener;

    fn run_in_tokio_context(f: impl std::future::Future<Output = ()>) {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(async {
                f.await;
            }),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.handle().block_on(async { f.await });
            }
        };
    }

    async fn test_runner(test: impl AsyncFn(crate::client::Client)) {
        run_in_tokio_context(async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap(); // Bind to any free port
            let server_address = listener.local_addr().unwrap().to_string();

            tokio::spawn(async {
                run_with_listener(listener, None, "1234".to_string()).await;
            });

            println!("{}", server_address);

            test(Client::new((format!("http://{}", server_address), "1234".to_string())).unwrap())
                .await;
        });
    }

    fm_core::finance_manager_test::unit_tests!(test_runner);
}
