//! Some implementations of the FinanceManager trait

#[cfg(feature = "ram")]
pub mod ram_finance_manager;
#[cfg(feature = "ram")]
pub use ram_finance_manager::RamFinanceManager;

#[cfg(feature = "sqlite")]
pub mod sqlite_finange_manager;
#[cfg(feature = "sqlite")]
pub use sqlite_finange_manager::SqliteFinanceManager;
