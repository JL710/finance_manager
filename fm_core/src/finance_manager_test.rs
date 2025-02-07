#[macro_export]
#[allow(unused_macros)]
macro_rules! unit_tests {
    ($gen_fm:expr) => {
        #[cfg(test)]
        mod test {
            use std::collections::HashMap;
            use $crate::FinanceManager;
            use $crate::*;

            #[async_std::test]
            async fn create_asset_account() {
                let mut fm = ($gen_fm)().await;
                let account = fm
                    .create_asset_account("Test".to_string(), None, None, None, Currency::default())
                    .await
                    .unwrap();
                assert_eq!(account.name(), "Test");
                assert_eq!(account.note(), None);
                assert_eq!(*account.iban(), None);
                assert_eq!(account.bic(), None);
                assert_eq!(*account.offset(), Currency::default());

                if let account::Account::AssetAccount(fetched_account) =
                    fm.get_account(account.id()).await.unwrap().unwrap()
                {
                    assert_eq!(fetched_account, account);
                } else {
                    assert!(false);
                }
            }

            #[async_std::test]
            async fn get_accounts() {
                let mut fm = ($gen_fm)().await;
                let accounts = fm.get_accounts().await.unwrap();
                assert_eq!(accounts.len(), 0);

                let acc = fm
                    .create_asset_account("Test".to_string(), None, None, None, Currency::default())
                    .await
                    .unwrap();
                let accounts = fm.get_accounts().await.unwrap();
                assert_eq!(accounts.len(), 1);
                assert_eq!(accounts[0], account::Account::AssetAccount(acc));
            }

            #[async_std::test]
            async fn create_book_checking_account() {
                let mut fm = ($gen_fm)().await;
                let account = fm
                    .create_book_checking_account("Test".to_string(), None, None, None)
                    .await
                    .unwrap();
                assert_eq!(account.name(), "Test");
                assert_eq!(account.note(), None);
                assert_eq!(*account.iban(), None);
                assert_eq!(account.bic(), None);

                if let account::Account::BookCheckingAccount(fetched_account) =
                    fm.get_account(account.id()).await.unwrap().unwrap()
                {
                    assert_eq!(fetched_account, account);
                } else {
                    assert!(false);
                }
            }

            #[async_std::test]
            async fn delete_category() {
                let mut fm = ($gen_fm)().await;

                let acc1 = fm
                    .create_asset_account(
                        "Test1".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let acc2 = fm
                    .create_asset_account(
                        "Test2".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let category = fm.create_category("Test".to_string()).await.unwrap();

                let transaction = fm
                    .create_transaction(
                        Currency::default(),
                        "Test".to_string(),
                        None,
                        acc1.id(),
                        acc2.id(),
                        None,
                        DateTime::now_utc(),
                        HashMap::new(),
                        [(category.id().clone(), Sign::Positive)]
                            .iter()
                            .cloned()
                            .collect(),
                    )
                    .await
                    .unwrap();

                fm.delete_category(*category.id()).await.unwrap();

                // check if category is deleted
                assert!(fm.get_category(*category.id()).await.unwrap().is_none());

                // check if category is removed from transactions
                assert!(fm
                    .get_transaction(*transaction.id())
                    .await
                    .unwrap()
                    .unwrap()
                    .categories()
                    .is_empty());
            }

            #[async_std::test]
            async fn delete_budget() {
                let mut fm = ($gen_fm)().await;

                let acc1 = fm
                    .create_asset_account(
                        "Test1".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let acc2 = fm
                    .create_asset_account(
                        "Test2".to_string(),
                        None,
                        None,
                        None,
                        Currency::default(),
                    )
                    .await
                    .unwrap();

                let budget1 = fm
                    .create_budget(
                        "test budget".to_string(),
                        None,
                        Currency::default(),
                        Recurring::DayInMonth(1),
                    )
                    .await
                    .unwrap();

                let transaction1 = fm
                    .create_transaction(
                        Currency::default(),
                        "Transaction1".to_string(),
                        None,
                        acc1.id(),
                        acc2.id(),
                        Some((*budget1.id(), Sign::Positive)),
                        DateTime::now_utc(),
                        HashMap::default(),
                        HashMap::default(),
                    )
                    .await
                    .unwrap();

                fm.delete_budget(*budget1.id()).await.unwrap();

                assert!(fm.get_budget(*budget1.id()).await.unwrap().is_none());

                assert!(fm
                    .get_transaction(*transaction1.id())
                    .await
                    .unwrap()
                    .unwrap()
                    .budget()
                    .is_none());
            }
        }
    };
}

pub use unit_tests;
