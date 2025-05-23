use super::*;
use bigdecimal::FromPrimitive;
use time::macros::*;

pub async fn create_asset_account_test<T: FinanceManager>(mut fm: T) {
    let account = fm
        .create_asset_account("Test".to_string(), None, None, None, Currency::default())
        .await
        .unwrap();
    assert_eq!(account.name, "Test");
    assert_eq!(account.note, None);
    assert_eq!(account.iban, None);
    assert_eq!(account.bic, None);
    assert_eq!(account.offset, Currency::default());

    if let account::Account::AssetAccount(fetched_account) =
        fm.get_account(account.id).await.unwrap().unwrap()
    {
        assert_eq!(fetched_account, account);
    } else {
        panic!();
    }
}

pub async fn get_accounts_test<T: FinanceManager>(mut fm: T) {
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

pub async fn create_book_checking_account_test<T: FinanceManager>(mut fm: T) {
    let account = fm
        .create_book_checking_account("Test".to_string(), None, None, None)
        .await
        .unwrap();
    assert_eq!(account.name, "Test");
    assert_eq!(account.note, None);
    assert_eq!(account.iban, None);
    assert_eq!(account.bic, None);

    if let account::Account::BookCheckingAccount(fetched_account) =
        fm.get_account(account.id).await.unwrap().unwrap()
    {
        assert_eq!(fetched_account, account);
    } else {
        panic!()
    }
}

pub async fn delete_category_test<T: FinanceManager>(mut fm: T) {
    let acc1 = fm
        .create_asset_account("Test1".to_string(), None, None, None, Currency::default())
        .await
        .unwrap();

    let acc2 = fm
        .create_asset_account("Test2".to_string(), None, None, None, Currency::default())
        .await
        .unwrap();

    let category = fm.create_category("Test".to_string()).await.unwrap();

    let transaction = fm
        .create_transaction(
            Currency::default(),
            "Test".to_string(),
            None,
            acc1.id,
            acc2.id,
            None,
            DateTime::now_utc(),
            HashMap::new(),
            [(category.id, Sign::Positive)].iter().cloned().collect(),
        )
        .await
        .unwrap();

    fm.delete_category(category.id).await.unwrap();

    // check if category is deleted
    assert!(fm.get_category(category.id).await.unwrap().is_none());

    // check if category is removed from transactions
    assert!(
        fm.get_transaction(transaction.id)
            .await
            .unwrap()
            .unwrap()
            .categories
            .is_empty()
    );
}

pub async fn delete_budget_test<T: FinanceManager>(mut fm: T) {
    let acc1 = fm
        .create_asset_account("Test1".to_string(), None, None, None, Currency::default())
        .await
        .unwrap();

    let acc2 = fm
        .create_asset_account("Test2".to_string(), None, None, None, Currency::default())
        .await
        .unwrap();

    let budget1 = fm
        .create_budget(
            "test budget".to_string(),
            None,
            Currency::default(),
            budget::Recurring::DayInMonth(1),
        )
        .await
        .unwrap();

    let transaction1 = fm
        .create_transaction(
            Currency::default(),
            "Transaction1".to_string(),
            None,
            acc1.id,
            acc2.id,
            Some((budget1.id, Sign::Positive)),
            DateTime::now_utc(),
            HashMap::default(),
            HashMap::default(),
        )
        .await
        .unwrap();

    fm.delete_budget(budget1.id).await.unwrap();

    assert!(fm.get_budget(budget1.id).await.unwrap().is_none());

    assert!(
        fm.get_transaction(transaction1.id)
            .await
            .unwrap()
            .unwrap()
            .budget
            .is_none()
    );
}

pub mod timespan_test {
    use super::*;

    async fn generate_transactions<T: FinanceManager>(
        fm: &mut T,
    ) -> (
        Transaction,
        Transaction,
        Transaction,
        Transaction,
        Transaction,
        Transaction,
        Transaction,
        crate::account::Account,
        Budget,
    ) {
        let acc1 = fm
            .create_asset_account(
                "asset_acc".to_string(),
                None,
                None,
                None,
                Currency::default(),
            )
            .await
            .unwrap();
        let acc2 = fm
            .create_book_checking_account("book_checking_acc2".to_string(), None, None, None)
            .await
            .unwrap();
        let acc3 = fm
            .create_book_checking_account("book_checking_acc3".to_string(), None, None, None)
            .await
            .unwrap();

        let budget1 = fm
            .create_budget(
                "budget1".to_string(),
                None,
                Currency::default(),
                budget::Recurring::DayInMonth(1),
            )
            .await
            .unwrap();
        let budget2 = fm
            .create_budget(
                "budget1".to_string(),
                None,
                Currency::default(),
                budget::Recurring::DayInMonth(1),
            )
            .await
            .unwrap();

        let t0 = fm
            .create_transaction(
                Currency::default(),
                "t0".to_string(),
                None,
                acc1.id,
                acc3.id,
                Some((budget2.id, Sign::Positive)),
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(9:30)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t1 = fm
            .create_transaction(
                Currency::default(),
                "t1".to_string(),
                None,
                acc1.id,
                acc3.id,
                None,
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(9:30)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t2 = fm
            .create_transaction(
                Currency::default(),
                "t2".to_string(),
                None,
                acc1.id,
                acc2.id,
                Some((budget1.id, Sign::Positive)),
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(10:30)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t3 = fm
            .create_transaction(
                Currency::default(),
                "t3".to_string(),
                None,
                acc1.id,
                acc2.id,
                Some((budget1.id, Sign::Positive)),
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(11:30)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t4 = fm
            .create_transaction(
                Currency::default(),
                "t4".to_string(),
                None,
                acc1.id,
                acc2.id,
                Some((budget1.id, Sign::Positive)),
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(12:50)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t5 = fm
            .create_transaction(
                Currency::default(),
                "t5".to_string(),
                None,
                acc1.id,
                acc3.id,
                None,
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(13:50)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        let t6 = fm
            .create_transaction(
                Currency::default(),
                "t6".to_string(),
                None,
                acc1.id,
                acc3.id,
                Some((budget2.id, Sign::Positive)),
                time::OffsetDateTime::new_utc(date!(2024 - 01 - 01), time!(13:50)),
                HashMap::default(),
                HashMap::default(),
            )
            .await
            .unwrap();
        (t0, t1, t2, t3, t4, t5, t6, acc2.into(), budget1)
    }

    pub mod get_transactions_of_budget {
        use super::*;

        pub async fn start_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_budget(
                    objects.8.id,
                    (
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:30),
                        )),
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:50),
                        )),
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 1);
            assert!(result.iter().any(|x| x.id == objects.2.id));
        }

        pub async fn start_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_budget(
                    objects.8.id,
                    (
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:50),
                        )),
                        None,
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 2);
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
        }

        pub async fn none_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_budget(
                    objects.8.id,
                    (
                        None,
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(11:50),
                        )),
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 2);
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
        }

        pub async fn none_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_budget(objects.8.id, (None, None))
                .await
                .unwrap();
            assert_eq!(result.len(), 3);
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
        }
    }

    pub mod get_transactions_of_account {
        use super::*;

        pub async fn start_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_account(
                    *objects.7.id(),
                    (
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:30),
                        )),
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:50),
                        )),
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 1);
            assert!(result.iter().any(|x| x.id == objects.2.id));
        }

        pub async fn start_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_account(
                    *objects.7.id(),
                    (
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(10:50),
                        )),
                        None,
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 2);
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
        }

        pub async fn none_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_account(
                    *objects.7.id(),
                    (
                        None,
                        Some(time::OffsetDateTime::new_utc(
                            date!(2024 - 01 - 01),
                            time!(11:50),
                        )),
                    ),
                )
                .await
                .unwrap();
            assert_eq!(result.len(), 2);
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
        }

        pub async fn none_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_of_account(*objects.7.id(), (None, None))
                .await
                .unwrap();
            assert_eq!(result.len(), 3);
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
        }
    }

    pub mod get_transactions_in_timespan {
        use super::*;

        pub async fn start_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_in_timespan((
                    Some(time::OffsetDateTime::new_utc(
                        date!(2024 - 01 - 01),
                        time!(10:30),
                    )),
                    Some(time::OffsetDateTime::new_utc(
                        date!(2024 - 01 - 01),
                        time!(10:50),
                    )),
                ))
                .await
                .unwrap();
            assert_eq!(result.len(), 1);
            assert!(result.iter().any(|x| x.id == objects.2.id));
        }

        pub async fn start_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_in_timespan((
                    Some(time::OffsetDateTime::new_utc(
                        date!(2024 - 01 - 01),
                        time!(10:50),
                    )),
                    None,
                ))
                .await
                .unwrap();
            assert_eq!(result.len(), 4);
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
            assert!(result.iter().any(|x| x.id == objects.5.id));
            assert!(result.iter().any(|x| x.id == objects.6.id));
        }

        pub async fn none_end_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm
                .get_transactions_in_timespan((
                    None,
                    Some(time::OffsetDateTime::new_utc(
                        date!(2024 - 01 - 01),
                        time!(11:50),
                    )),
                ))
                .await
                .unwrap();
            assert_eq!(result.len(), 4);
            assert!(result.iter().any(|x| x.id == objects.0.id));
            assert!(result.iter().any(|x| x.id == objects.1.id));
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
        }

        pub async fn none_none_test<T: FinanceManager>(mut fm: T) {
            let objects = generate_transactions(&mut fm).await;
            let result = fm.get_transactions_in_timespan((None, None)).await.unwrap();
            assert_eq!(result.len(), 7);
            assert!(result.iter().any(|x| x.id == objects.0.id));
            assert!(result.iter().any(|x| x.id == objects.1.id));
            assert!(result.iter().any(|x| x.id == objects.2.id));
            assert!(result.iter().any(|x| x.id == objects.3.id));
            assert!(result.iter().any(|x| x.id == objects.4.id));
            assert!(result.iter().any(|x| x.id == objects.5.id));
            assert!(result.iter().any(|x| x.id == objects.6.id));
        }
    }
}

pub async fn update_transaction_test<T: FinanceManager>(mut fm: T) {
    let acc1 = fm
        .create_asset_account(
            "asset_acc".to_string(),
            None,
            None,
            None,
            Currency::default(),
        )
        .await
        .unwrap();
    let acc2 = fm
        .create_book_checking_account("book_checking_acc".to_string(), None, None, None)
        .await
        .unwrap();

    let mut transaction = fm
        .create_transaction(
            Currency::default(),
            "t1".to_string(),
            None,
            acc1.id,
            acc2.id,
            None,
            time::OffsetDateTime::now_utc(),
            HashMap::default(),
            HashMap::default(),
        )
        .await
        .unwrap();
    transaction.title = String::from("Changed Name");
    fm.update_transaction(transaction.clone()).await.unwrap();
    let fetched_transaction = fm.get_transaction(transaction.id).await.unwrap().unwrap();
    assert_eq!(fetched_transaction, transaction);
}

pub async fn create_bill_test<T: FinanceManager>(mut fm: T) {
    let acc1 = fm
        .create_asset_account(
            "asset_acc".to_string(),
            None,
            None,
            None,
            Currency::default(),
        )
        .await
        .unwrap();
    let acc2 = fm
        .create_book_checking_account("book_checking_acc".to_string(), None, None, None)
        .await
        .unwrap();
    let transaction = fm
        .create_transaction(
            Currency::default(),
            "t1".to_string(),
            None,
            acc1.id,
            acc2.id,
            None,
            time::OffsetDateTime::now_utc(),
            HashMap::default(),
            HashMap::default(),
        )
        .await
        .unwrap();
    let due_date = time::OffsetDateTime::now_utc().replace_time(time::Time::MIDNIGHT);
    let bill = fm
        .create_bill(
            "Name".to_string(),
            Some("Description".to_string()),
            Currency::Eur(bigdecimal::BigDecimal::from_f32(5.0).unwrap()),
            HashMap::from([(transaction.id, Sign::Positive)]),
            Some(due_date),
            true,
        )
        .await
        .unwrap();
    let fetched_bill = fm.get_bill(&bill.id).await.unwrap().unwrap();
    assert_eq!(bill, fetched_bill)
}

#[macro_export]
#[allow(unused_macros)]
macro_rules! unit_tests {
    ($runner:expr) => {
        use $crate::finance_manager_test::*;

        #[async_std::test]
        async fn create_asset_account() {
            ($runner)(create_asset_account_test).await;
        }

        #[async_std::test]
        async fn get_accounts() {
            ($runner)(get_accounts_test).await;
        }

        #[async_std::test]
        async fn create_book_checking_account() {
            ($runner)(create_book_checking_account_test).await;
        }

        #[async_std::test]
        async fn delete_category() {
            ($runner)(delete_category_test).await;
        }

        #[async_std::test]
        async fn delete_budget() {
            ($runner)(delete_budget_test).await;
        }

        mod get_transactions_in_timespan {
            use super::test_runner;
            use $crate::finance_manager_test::timespan_test::get_transactions_in_timespan::*;

            #[async_std::test]
            async fn start_end() {
                ($runner)(start_end_test).await;
            }

            #[async_std::test]
            async fn start_none() {
                ($runner)(start_none_test).await;
            }

            #[async_std::test]
            async fn none_end() {
                ($runner)(none_end_test).await;
            }

            #[async_std::test]
            async fn none_none() {
                ($runner)(none_none_test).await;
            }
        }

        mod get_transactions_of_account {
            use super::test_runner;
            use $crate::finance_manager_test::timespan_test::get_transactions_of_account::*;

            #[async_std::test]
            async fn start_end() {
                ($runner)(start_end_test).await;
            }

            #[async_std::test]
            async fn start_none() {
                ($runner)(start_none_test).await;
            }

            #[async_std::test]
            async fn none_end() {
                ($runner)(none_end_test).await;
            }

            #[async_std::test]
            async fn none_none() {
                ($runner)(none_none_test).await;
            }
        }

        mod get_transactions_of_budget {
            use super::test_runner;
            use $crate::finance_manager_test::timespan_test::get_transactions_of_budget::*;

            #[async_std::test]
            async fn start_end() {
                ($runner)(start_end_test).await;
            }

            #[async_std::test]
            async fn start_none() {
                ($runner)(start_none_test).await;
            }

            #[async_std::test]
            async fn none_end() {
                ($runner)(none_end_test).await;
            }

            #[async_std::test]
            async fn none_none() {
                ($runner)(none_none_test).await;
            }
        }

        #[async_std::test]
        async fn update_transaction() {
            ($runner)(update_transaction_test).await;
        }

        #[async_std::test]
        async fn create_bill() {
            ($runner)(create_bill_test).await;
        }
    };
}

use time::macros::date;
pub use unit_tests;
