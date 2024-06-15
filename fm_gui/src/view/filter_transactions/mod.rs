use super::super::{utils, AppMessage, View};
use anyhow::Result;
use async_std::sync::Mutex;
use fm_core::transaction_filter::TransactionFilter;
use std::sync::Arc;

mod filter_component;

pub fn switch_view_command(
    finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
) -> iced::Command<AppMessage> {
    iced::Command::perform(FilterTransactionView::fetch(finance_manager), |x| {
        AppMessage::SwitchView(View::FilterTransaction(x.unwrap()))
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeFilter(TransactionFilter),
    ToggleEditFilter,
    ViewAccount(fm_core::Id),
    ViewTransaction(fm_core::Id),
    UpdateTransactions(
        Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ),
}

#[derive(Debug, Clone)]
pub struct FilterTransactionView {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    change_filter: bool,
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    filter: TransactionFilter,
}

impl FilterTransactionView {
    pub async fn fetch(
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> Result<Self> {
        let locked_manager = finance_manager.lock().await;
        let accounts = locked_manager.get_accounts().await?;
        let categories = locked_manager.get_categories().await?;
        Ok(Self {
            accounts,
            categories,
            change_filter: false,
            transactions: Vec::new(),
            filter: TransactionFilter::default(),
        })
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::ToggleEditFilter => {
                self.change_filter = !self.change_filter;
            }
            Message::ChangeFilter(filter) => {
                self.filter = filter.clone();
                self.change_filter = false;
                return (
                    None,
                    iced::Command::perform(
                        async move {
                            let locked_manager = finance_manager.lock().await;
                            let transactions = locked_manager
                                .get_filtered_transactions(filter.clone())
                                .await
                                .unwrap();
                            let accounts = locked_manager.get_accounts().await.unwrap();

                            let mut tuples = Vec::new();
                            for transaction in transactions {
                                let source = accounts
                                    .iter()
                                    .find(|x| x.id() == transaction.source())
                                    .unwrap()
                                    .clone();
                                let destination = accounts
                                    .iter()
                                    .find(|x| x.id() == transaction.destination())
                                    .unwrap()
                                    .clone();
                                tuples.push((transaction, source, destination));
                            }
                            tuples
                        },
                        |x| AppMessage::FilterTransactionMessage(Message::UpdateTransactions(x)),
                    ),
                );
            }
            Message::ViewAccount(id) => {
                return (
                    Some(View::Empty),
                    super::view_account::switch_view_command(id, finance_manager),
                );
            }
            Message::ViewTransaction(id) => {
                return (
                    Some(View::Empty),
                    super::view_transaction::switch_view_command(id, finance_manager),
                );
            }
            Message::UpdateTransactions(transactions) => {
                self.transactions = transactions;
            }
        }
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<Message> {
        iced::widget::column![
            iced::widget::text("Filter Transactions"),
            iced::widget::button(iced::widget::text("Edit Filter"))
                .on_press(Message::ToggleEditFilter),
            if self.change_filter {
                filter_component::FilterComponent::new(
                    self.filter.clone(),
                    Message::ChangeFilter,
                    &self.accounts,
                    &self.categories,
                )
                .into_element()
            } else {
                utils::transaction_table(
                    self.transactions.clone(),
                    |_| None,
                    Message::ViewTransaction,
                    Message::ViewAccount,
                )
            }
        ]
        .width(iced::Length::Fill)
        .into()
    }
}
