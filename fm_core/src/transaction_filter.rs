use super::{Id, Timespan, Transaction};
pub type AccountFilter = (Id, bool, Option<Timespan>);
pub type CategoryFilter = (Id, bool, Option<Timespan>);

/// exclude > include
///
/// Default timespan is used as default for every selected category and account.
/// Included timespan includes for that account/category transactions that are in range (by default nothing is included).
/// Exclude timespan excludes only transaction in that range but does not include everything else and has higher priority than includes.
#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    default_timespan: Timespan,
    accounts: Vec<AccountFilter>, // id, include (true) or exclude (false)
    categories: Vec<CategoryFilter>, // id, include (true) or exclude (false)
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

    pub fn total_timespan(&self) -> Timespan {
        let mut timespan = self.default_timespan;
        for account_timespan in self
            .accounts
            .iter()
            .map(|x| x.2)
            .chain(self.categories.iter().map(|x| x.2))
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
            let mut stay = false;

            let account_iterator = self
                .accounts
                .iter()
                .filter(|account| {
                    *transaction.source() == account.0 || *transaction.destination() == account.0
                })
                .map(|x| (x.1, x.2));
            let category_iterator = self
                .categories
                .iter()
                .filter(|category| {
                    transaction
                        .categories
                        .iter()
                        .filter(|x| x.0 == category.0)
                        .count()
                        >= 1
                })
                .map(|x| (x.1, x.2));

            // check the accounts and categories
            for (include, timespan) in account_iterator
                .chain(category_iterator)
                .map(|(x, y)| (x, y.unwrap_or(self.default_timespan)))
            {
                // check if it is in the timespan#
                let in_timespan = if let Some(start) = timespan.0 {
                    start <= *transaction.date()
                } else {
                    true
                } | if let Some(end) = timespan.1 {
                    end >= *transaction.date()
                } else {
                    true
                };
                // kick based on that
                if in_timespan {
                    if !include {
                        return false;
                    }
                    stay = true;
                }
            }

            stay
        });

        transactions
    }
}
