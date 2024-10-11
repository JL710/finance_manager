use super::{Bill, Id, Timespan, Transaction};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Filter<I: Clone + std::fmt::Debug> {
    pub negated: bool,
    pub id: I,
    pub include: bool,
    pub timespan: Option<Timespan>,
}

/// exclude > include
///
/// Default timespan is used as default for every selected category and account.
/// Included timespan includes for that account/category transactions that are in range (by default nothing is included).
/// Exclude timespan excludes only transaction in that range but does not include everything else and has higher priority than includes.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct TransactionFilter {
    default_timespan: Timespan,
    accounts: Vec<Filter<Id>>,
    categories: Vec<Filter<Id>>,
    bills: Vec<Filter<Bill>>,
    budgets: Vec<Filter<Id>>,
}

impl TransactionFilter {
    pub fn set_default_timespan(&mut self, timespan: Timespan) {
        self.default_timespan = timespan;
    }

    pub fn get_default_timespan(&self) -> &Timespan {
        &self.default_timespan
    }

    pub fn get_account_filters(&self) -> &Vec<Filter<Id>> {
        &self.accounts
    }

    pub fn add_account(&mut self, filter: Filter<Id>) {
        self.accounts.push(filter);
    }

    pub fn push_account(self, filter: Filter<Id>) -> Self {
        let mut new = self;
        new.add_account(filter);
        new
    }

    pub fn delete_account(&mut self, filter: Filter<Id>) {
        self.accounts.retain(|x| *x != filter);
    }

    pub fn edit_account(&mut self, old: Filter<Id>, new: Filter<Id>) {
        for f in self.accounts.iter_mut() {
            if *f == old {
                *f = new;
                return;
            }
        }
    }

    pub fn get_category_filters(&self) -> &Vec<Filter<Id>> {
        &self.categories
    }

    pub fn add_category(&mut self, filter: Filter<Id>) {
        self.categories.push(filter);
    }

    pub fn push_category(self, filter: Filter<Id>) -> Self {
        let mut new = self;
        new.add_category(filter);
        new
    }

    pub fn delete_category(&mut self, filter: Filter<Id>) {
        self.categories.retain(|x| *x != filter);
    }

    pub fn edit_category(&mut self, old: Filter<Id>, new: Filter<Id>) {
        for f in self.categories.iter_mut() {
            if *f == old {
                *f = new;
                return;
            }
        }
    }

    pub fn get_bill_filters(&self) -> &Vec<Filter<Bill>> {
        &self.bills
    }

    pub fn add_bill(&mut self, filter: Filter<Bill>) {
        self.bills.push(filter);
    }

    pub fn push_bill(self, filter: Filter<Bill>) -> Self {
        let mut new = self;
        new.add_bill(filter);
        new
    }

    pub fn delete_bill(&mut self, filter: Filter<Bill>) {
        self.bills.retain(|x| *x != filter);
    }

    pub fn edit_bill(&mut self, old: Filter<Bill>, new: Filter<Bill>) {
        for f in self.bills.iter_mut() {
            if *f == old {
                *f = new;
                return;
            }
        }
    }

    pub fn get_budget_filters(&self) -> &Vec<Filter<Id>> {
        &self.budgets
    }

    pub fn add_budget(&mut self, filter: Filter<Id>) {
        self.budgets.push(filter);
    }

    pub fn push_budget(self, filter: Filter<Id>) -> Self {
        let mut new = self;
        new.add_budget(filter);
        new
    }

    pub fn delete_budget(&mut self, filter: Filter<Id>) {
        self.budgets.retain(|x| *x != filter);
    }

    pub fn edit_budget(&mut self, old: Filter<Id>, new: Filter<Id>) {
        for f in self.budgets.iter_mut() {
            if *f == old {
                *f = new;
                return;
            }
        }
    }

    pub fn total_timespan(&self) -> Timespan {
        let mut timespan = self.default_timespan;
        for timespan_iteration in self
            .accounts
            .iter()
            .map(|x| x.timespan)
            .chain(self.categories.iter().map(|x| x.timespan))
            .chain(self.bills.iter().map(|x| x.timespan))
            .chain(self.budgets.iter().map(|x| x.timespan))
        {
            if timespan_iteration.is_none() {
                continue;
            }
            let (start, end) = timespan_iteration.unwrap();

            // check start
            if let Some(timespan_start) = timespan.0 {
                println!("check1.0");
                if start.is_none() {
                    timespan.0 = None;
                } else if start.unwrap() < timespan_start {
                    println!("check1");
                    timespan.0 = start;
                }
            } else if let Some(start) = start {
                timespan.0 = Some(start);
            }

            // check end
            if let Some(timespan_end) = timespan.1 {
                if end.is_none() {
                    timespan.1 = None;
                } else if end.unwrap() > timespan_end {
                    println!("check2");
                    timespan.1 = end;
                }
            } else if let Some(end) = end {
                timespan.1 = Some(end);
            }
        }

        timespan
    }

    pub fn filter_transactions(&self, mut transactions: Vec<Transaction>) -> Vec<Transaction> {
        transactions.retain(|transaction| {
            // create iterators from filters with include/exclude and timespan
            let account_filter_iterator = self
                .accounts
                .iter()
                .filter(|account| {
                    transaction.connection_with_account(account.id) != account.negated
                })
                .map(|x| (x.include, x.timespan));
            let category_filter_iterator = self
                .categories
                .iter()
                .filter(|category| {
                    (transaction
                        .categories()
                        .iter()
                        .filter(|x| *x.0 == category.id)
                        .count()
                        >= 1)
                        != category.negated
                })
                .map(|x| (x.include, x.timespan));
            let bill_filter_iterator = self
                .bills
                .iter()
                .filter(|bill_filter| {
                    bill_filter.id.transactions().contains_key(transaction.id())
                        != bill_filter.negated
                })
                .map(|x| (x.include, x.timespan));
            let budget_filter_iterator = self
                .budgets
                .iter()
                .filter(|budget_filter| {
                    (transaction.budget().map(|x| x.0) == Some(budget_filter.id))
                        != budget_filter.negated
                })
                .map(|x| (x.include, x.timespan));

            // if the transaction should stay or get removed
            let mut stay = false;

            // iterate over all filters
            for (include, timespan) in account_filter_iterator
                .chain(category_filter_iterator)
                .chain(bill_filter_iterator)
                .chain(budget_filter_iterator)
                .map(|(x, y)| (x, y.unwrap_or(self.default_timespan)))
            {
                // check if it is in the timespan
                let in_timespan = if let Some(start) = timespan.0 {
                    start <= *transaction.date()
                } else {
                    true
                } || if let Some(end) = timespan.1 {
                    end >= *transaction.date()
                } else {
                    true
                };

                // kick based on that
                if in_timespan {
                    if !include {
                        // if it is in the timespan and should be excluded directly end the filter check for this transaction
                        return false;
                    }
                    // if it is in the timespan and should be included set stay to true
                    stay = true;
                }
            }

            stay
        });

        transactions
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Bill, Currency, Sign};
    use std::collections::HashMap;
    use time::macros::*;

    fn generate_simple_transaction(id: Id, source: Id, destination: Id) -> Transaction {
        Transaction::new(
            id,
            Currency::default(),
            format!("Transaction {}", id),
            None,
            source,
            destination,
            None,
            time::OffsetDateTime::now_utc(),
            HashMap::new(),
            HashMap::new(),
        )
    }

    fn generate_transaction_with_budget(id: Id, budget: Option<Id>) -> Transaction {
        Transaction::new(
            id,
            Currency::default(),
            format!("Transaction {}", id),
            None,
            1,
            2,
            budget.map(|x| (x, Sign::Positive)),
            time::OffsetDateTime::now_utc(),
            HashMap::new(),
            HashMap::new(),
        )
    }

    fn generate_advanced_transaction(
        id: Id,
        source: Id,
        destination: Id,
        budget: Option<Id>,
        categories: HashMap<Id, Sign>,
    ) -> Transaction {
        Transaction::new(
            id,
            Currency::default(),
            format!("Transaction {}", id),
            None,
            source,
            destination,
            budget.map(|x| (x, Sign::Positive)),
            time::OffsetDateTime::now_utc(),
            HashMap::new(),
            categories,
        )
    }

    fn generate_test_transactions_1() -> Vec<Transaction> {
        vec![
            generate_simple_transaction(1, 1, 5),
            generate_simple_transaction(2, 3, 4),
            generate_simple_transaction(3, 3, 4),
            generate_simple_transaction(4, 1, 2),
        ]
    }

    fn generate_test_bill_1() -> Bill {
        Bill::new(
            1,
            "Bill".to_string(),
            None,
            Currency::default(),
            HashMap::from([(2, Sign::Positive), (3, Sign::Positive)]),
            None,
        )
    }

    fn generate_test_bill_2() -> Bill {
        Bill::new(
            1,
            "Bill".to_string(),
            None,
            Currency::default(),
            HashMap::from([(3, Sign::Positive), (4, Sign::Positive)]),
            None,
        )
    }

    #[test]
    fn filter_bill_include() {
        let transactions = generate_test_transactions_1();
        let bill = generate_test_bill_1();
        let mut filter = TransactionFilter::default();
        filter.add_bill(Filter {
            negated: false,
            id: bill,
            include: true,
            timespan: None,
        });
        let result = filter.filter_transactions(transactions);
        assert_eq!(result.len(), 2);
        result.iter().find(|x| *x.id() == 2).unwrap();
        result.iter().find(|x| *x.id() == 3).unwrap();
    }

    #[test]
    fn filter_bill_include_and_exclude() {
        let mut filter = TransactionFilter::default();
        filter.add_bill(Filter {
            negated: false,
            id: generate_test_bill_1(),
            include: true,
            timespan: None,
        });
        filter.add_bill(Filter {
            negated: false,
            id: generate_test_bill_2(),
            include: false,
            timespan: None,
        });
        let result = filter.filter_transactions(generate_test_transactions_1());
        assert_eq!(result.len(), 1);
        assert_eq!(*result[0].id(), 2);
    }

    #[test]
    fn filter_account_include() {
        let result = TransactionFilter::default()
            .push_account(Filter {
                negated: false,
                id: 1,
                include: true,
                timespan: None,
            })
            .filter_transactions(generate_test_transactions_1());
        assert_eq!(result.len(), 2);
        result.iter().find(|x| *x.id() == 1).unwrap();
        result.iter().find(|x| *x.id() == 4).unwrap();
    }

    #[test]
    fn filter_account_include_and_exclude() {
        let result = TransactionFilter::default()
            .push_account(Filter {
                negated: false,
                id: 1,
                include: true,
                timespan: None,
            })
            .push_account(Filter {
                negated: false,
                id: 5,
                include: false,
                timespan: None,
            })
            .filter_transactions(generate_test_transactions_1());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), &4);
    }

    #[test]
    fn filter_budget_include() {
        let transactions = vec![
            generate_transaction_with_budget(1, Some(1)),
            generate_transaction_with_budget(2, Some(2)),
            generate_transaction_with_budget(3, Some(2)),
            generate_transaction_with_budget(4, Some(4)),
        ];

        let result = TransactionFilter::default()
            .push_budget(Filter {
                negated: false,
                id: 2,
                include: true,
                timespan: None,
            })
            .filter_transactions(transactions);
        assert_eq!(result.len(), 2);
        result.iter().find(|x| *x.id() == 2).unwrap();
        result.iter().find(|x| *x.id() == 3).unwrap();
    }

    #[test]
    fn filter_budget_include_and_exclude() {
        let transactions = vec![
            generate_advanced_transaction(1, 1, 2, Some(1), HashMap::default()),
            generate_advanced_transaction(2, 1, 2, Some(2), HashMap::default()),
            generate_advanced_transaction(3, 1, 2, Some(2), HashMap::default()),
            generate_advanced_transaction(4, 1, 2, Some(3), HashMap::default()),
        ];

        let result = TransactionFilter::default()
            .push_account(Filter {
                negated: false,
                id: 1,
                include: true,
                timespan: None,
            })
            .push_budget(Filter {
                negated: false,
                id: 2,
                include: false,
                timespan: None,
            })
            .filter_transactions(transactions);
        assert_eq!(result.len(), 2);
        assert!(result.iter().find(|x| x.id() == &1).is_some());
        assert!(result.iter().find(|x| x.id() == &4).is_some());
    }

    #[test]
    fn filter_category_include() {
        let transactions = vec![
            generate_advanced_transaction(
                1,
                1,
                2,
                None,
                HashMap::from([(1, Sign::Positive), (2, Sign::Negative)]),
            ),
            generate_advanced_transaction(2, 1, 2, None, HashMap::from([(1, Sign::Positive)])),
            generate_advanced_transaction(3, 1, 2, None, HashMap::from([(2, Sign::Positive)])),
            generate_advanced_transaction(4, 1, 2, None, HashMap::default()),
        ];

        let result = TransactionFilter::default()
            .push_category(Filter {
                negated: false,
                id: 1,
                include: true,
                timespan: None,
            })
            .filter_transactions(transactions);
        assert_eq!(result.len(), 2);
        result.iter().find(|x| *x.id() == 1).unwrap();
        result.iter().find(|x| *x.id() == 2).unwrap();
    }

    #[test]
    fn filter_category_include_and_exclude() {
        let transactions = vec![
            generate_advanced_transaction(
                1,
                1,
                2,
                None,
                HashMap::from([(1, Sign::Positive), (2, Sign::Negative)]),
            ),
            generate_advanced_transaction(2, 1, 2, None, HashMap::from([(1, Sign::Positive)])),
            generate_advanced_transaction(3, 1, 2, None, HashMap::from([(2, Sign::Positive)])),
            generate_advanced_transaction(4, 1, 2, None, HashMap::default()),
        ];

        let result = TransactionFilter::default()
            .push_category(Filter {
                negated: false,
                id: 1,
                include: true,
                timespan: None,
            })
            .push_category(Filter {
                negated: false,
                id: 2,
                include: false,
                timespan: None,
            })
            .filter_transactions(transactions);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id(), &2);
    }

    #[test]
    fn filter_total_timespan_empty() {
        let mut filter = TransactionFilter::default();
        filter.add_bill(Filter {
            negated: false,
            id: generate_test_bill_1(),
            include: true,
            timespan: None,
        });
        filter.add_account(Filter {
            negated: false,
            id: 2,
            include: true,
            timespan: None,
        });
        filter.add_category(Filter {
            negated: false,
            id: 1,
            include: true,
            timespan: None,
        });
        filter.add_budget(Filter {
            negated: false,
            id: 1,
            include: true,
            timespan: None,
        });
        assert_eq!(filter.total_timespan(), (None, None));
    }

    #[test]
    fn filter_total_timespan_only_one() {
        let timespan = (
            Some(time::OffsetDateTime::new_utc(
                date!(2024 - 01 - 01),
                time!(10:30),
            )),
            Some(time::OffsetDateTime::new_utc(
                date!(2024 - 02 - 01),
                time!(10:30),
            )),
        );

        assert_eq!(
            TransactionFilter::default()
                .push_bill(Filter {
                    negated: false,
                    id: generate_test_bill_1(),
                    include: true,
                    timespan: Some(timespan.clone()),
                })
                .total_timespan(),
            timespan.clone()
        );
        assert_eq!(
            TransactionFilter::default()
                .push_account(Filter {
                    negated: false,
                    id: 1,
                    include: true,
                    timespan: Some(timespan.clone()),
                })
                .total_timespan(),
            timespan.clone()
        );
        assert_eq!(
            TransactionFilter::default()
                .push_category(Filter {
                    negated: false,
                    id: 2,
                    include: true,
                    timespan: Some(timespan.clone()),
                })
                .total_timespan(),
            timespan.clone()
        );
        assert_eq!(
            TransactionFilter::default()
                .push_budget(Filter {
                    negated: false,
                    id: 2,
                    include: true,
                    timespan: Some(timespan.clone()),
                })
                .total_timespan(),
            timespan.clone()
        );
    }
}
