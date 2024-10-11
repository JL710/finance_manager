use super::{Bill, Id, Timespan, Transaction};
pub type AccountFilter = (bool, Id, bool, Option<Timespan>); // true if negated, id, include (true) or exclude (false), optional timespan
pub type CategoryFilter = (bool, Id, bool, Option<Timespan>); // true if negated, id, include (true) or exclude (false), optional timespan
pub type BillFilter = (bool, super::Bill, bool, Option<Timespan>); // true if negated, id, include (true) or exclude (false), optional timespan

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
    accounts: Vec<AccountFilter>,
    categories: Vec<CategoryFilter>,
    bills: Vec<Filter<Bill>>,
}

impl TransactionFilter {
    pub fn set_default_timespan(&mut self, timespan: Timespan) {
        self.default_timespan = timespan;
    }

    pub fn get_default_timespan(&self) -> &Timespan {
        &self.default_timespan
    }

    pub fn get_account_filters(&self) -> &Vec<AccountFilter> {
        &self.accounts
    }

    pub fn add_account(&mut self, filter: AccountFilter) {
        self.accounts.push(filter);
    }

    pub fn delete_account(&mut self, filter: AccountFilter) {
        self.accounts.retain(|x| *x != filter);
    }

    pub fn edit_account(&mut self, old: AccountFilter, new: AccountFilter) {
        for f in self.accounts.iter_mut() {
            if *f == old {
                *f = new;
                return;
            }
        }
    }

    pub fn get_category_filters(&self) -> &Vec<CategoryFilter> {
        &self.categories
    }

    pub fn add_category(&mut self, filter: CategoryFilter) {
        self.categories.push(filter);
    }

    pub fn delete_category(&mut self, filter: CategoryFilter) {
        self.categories.retain(|x| *x != filter);
    }

    pub fn edit_category(&mut self, old: CategoryFilter, new: CategoryFilter) {
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

    pub fn total_timespan(&self) -> Timespan {
        let mut timespan = self.default_timespan;
        for account_timespan in self
            .accounts
            .iter()
            .map(|x| x.3)
            .chain(self.categories.iter().map(|x| x.3))
        {
            if account_timespan.is_none() {
                continue;
            }
            let (start, end) = account_timespan.unwrap();

            // check start
            if let Some(timespan_start) = timespan.0 {
                if start.is_none() {
                    timespan.0 = None;
                } else if start.unwrap() < timespan_start {
                    timespan.0 = start;
                }
            }

            // check end
            if let Some(timespan_end) = timespan.1 {
                if end.is_none() {
                    timespan.1 = None;
                } else if end.unwrap() > timespan_end {
                    timespan.1 = end;
                }
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
                .filter(|account| transaction.connection_with_account(account.1) != account.0)
                .map(|x| (x.2, x.3));
            let category_filter_iterator = self
                .categories
                .iter()
                .filter(|category| {
                    (transaction
                        .categories()
                        .iter()
                        .filter(|x| *x.0 == category.1)
                        .count()
                        >= 1)
                        != category.0
                })
                .map(|x| (x.2, x.3));
            let bill_filter_iterator = self
                .bills
                .iter()
                .filter(|bill_filter| {
                    bill_filter.id.transactions().contains_key(transaction.id())
                        != bill_filter.negated
                })
                .map(|x| (x.include, x.timespan));

            // if the transaction should stay or get removed
            let mut stay = false;

            // iterate over all filters
            for (include, timespan) in account_filter_iterator
                .chain(category_filter_iterator)
                .chain(bill_filter_iterator)
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

    fn generate_test_transactions_1() -> Vec<Transaction> {
        vec![
            generate_simple_transaction(1, 1, 2),
            generate_simple_transaction(2, 1, 2),
            generate_simple_transaction(3, 1, 2),
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
}
