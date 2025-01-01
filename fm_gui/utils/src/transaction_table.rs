use super::{colored_currency_display, link};
use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action {
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    Task(iced::Task<Message>),
    None,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    ViewTransaction(fm_core::Id),
    ViewAccount(fm_core::Id),
    RemoveCategory {
        transaction_id: fm_core::Id,
        category_id: fm_core::Id,
    },
    SetCategory {
        transaction_id: fm_core::Id,
        category_id: fm_core::Id,
        sign: fm_core::Sign,
    },
    OpenCategoryPopup(fm_core::Id),
    ClosePopup,
    TransactionCategoryUpdated(fm_core::Transaction),
    TransactionTable(crate::table_view::InnerMessage<Message>),
}

pub struct TransactionTable {
    transactions: Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )>,
    transaction_table: crate::table_view::State<
        (
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        ),
        Vec<fm_core::Category>,
    >,
    amount_positive: Box<dyn Fn(fm_core::Transaction) -> Option<bool>>,
    categories: Vec<fm_core::Category>,
    /// The id of the transaction that has the category popup open if any
    category_popup: Option<fm_core::Id>,
    edit_svg: widget::svg::Handle,
}

impl std::fmt::Debug for TransactionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionTable")
            .field("transactions", &self.transactions)
            .field("amount_positive", &"...")
            .field("categories", &self.categories)
            .field("category_popup", &self.category_popup)
            .finish()
    }
}

impl TransactionTable {
    /// Create a table of transactions
    ///
    /// # Arguments
    /// - `transactions`: A slice of tuples of transactions and their source and destination accounts
    /// - `amount_positive`: A function that takes a transaction and returns a boolean that indicates how the amount should be colored
    ///     - `true`: The amount should be colored in a positive color
    ///     - `false`: The amount should be colored in a negative color
    ///     - `None`: The amount should not be colored
    pub fn new(
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        categories: Vec<fm_core::Category>,
        amount_positive: impl Fn(fm_core::Transaction) -> Option<bool> + Copy + 'static,
    ) -> Self {
        Self {
            transactions: transactions.clone(),
            categories: categories.clone(),
            amount_positive: Box::new(amount_positive),
            transaction_table: crate::table_view::State::new(transactions, categories)
                .sortable_columns([0, 1, 2, 3, 4, 5])
                .sort_by(move |a, b, column_index| match column_index {
                    0 => a.0.title().cmp(b.0.title()),
                    1 => a.0.date().cmp(b.0.date()),
                    2 => {
                        let a = (amount_positive)(a.0.clone()).map_or(a.0.amount(), |positive| {
                            if positive {
                                a.0.amount()
                            } else {
                                a.0.amount().negative()
                            }
                        });
                        let b = (amount_positive)(b.0.clone()).map_or(b.0.amount(), |positive| {
                            if positive {
                                b.0.amount()
                            } else {
                                b.0.amount().negative()
                            }
                        });
                        a.cmp(&b)
                    }
                    3 => a.1.name().cmp(b.1.name()),
                    4 => a.2.name().cmp(b.2.name()),
                    5 => a.0.categories().len().cmp(&b.0.categories().len()),
                    _ => std::cmp::Ordering::Equal,
                }),
            category_popup: None,
            edit_svg: widget::svg::Handle::from_memory(include_bytes!(
                "../../assets/pencil-fill.svg"
            )),
        }
    }

    pub fn change_transactions(
        &mut self,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ) {
        self.transactions = transactions;
        self.transaction_table.set_items(self.transactions.clone());
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::ViewTransaction(id) => Action::ViewTransaction(id),
            Message::ViewAccount(id) => Action::ViewAccount(id),
            Message::OpenCategoryPopup(id) => {
                if self.category_popup == Some(id) {
                    self.category_popup = None;
                } else {
                    self.category_popup = Some(id);
                }
                Action::None
            }
            Message::ClosePopup => {
                self.category_popup = None;
                Action::None
            }
            Message::RemoveCategory {
                transaction_id,
                category_id,
            } => {
                let transaction = &self
                    .transactions
                    .iter()
                    .find(|x| *x.0.id() == transaction_id)
                    .unwrap()
                    .0;
                let transaction_id = *transaction.id();
                let mut categories = transaction.categories().clone();
                categories.remove(&category_id);

                Action::Task(iced::Task::future(async move {
                    let new_transaction = finance_manager
                        .lock()
                        .await
                        .update_transaction_categories(transaction_id, categories)
                        .await
                        .unwrap();
                    Message::TransactionCategoryUpdated(new_transaction)
                }))
            }
            Message::SetCategory {
                transaction_id,
                category_id,
                sign,
            } => {
                let transaction = &self
                    .transactions
                    .iter()
                    .find(|x| *x.0.id() == transaction_id)
                    .unwrap()
                    .0;
                let transaction_id = *transaction.id();
                let mut categories = transaction.categories().clone();
                categories.insert(category_id, sign);

                Action::Task(iced::Task::future(async move {
                    let new_transaction = finance_manager
                        .lock()
                        .await
                        .update_transaction_categories(transaction_id, categories)
                        .await
                        .unwrap();
                    Message::TransactionCategoryUpdated(new_transaction)
                }))
            }
            Message::TransactionCategoryUpdated(transaction) => {
                let transaction_id = *transaction.id();
                let index = self
                    .transactions
                    .iter()
                    .position(|x| *x.0.id() == transaction_id)
                    .unwrap();
                self.transactions[index].0 = transaction;
                Action::None
            }
            Message::TransactionTable(inner) => {
                match self.transaction_table.perform(inner) {
                    crate::table_view::Action::OuterMessage(m) => {
                        return self.update(m, finance_manager);
                    }
                    crate::table_view::Action::PageChange(_) => {
                        self.category_popup = None;
                    }
                    crate::table_view::Action::None => {}
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let mut transactions = self.transactions.clone();
        transactions.sort_by(|(a, _, _), (b, _, _)| b.date().cmp(a.date()));
        let table = crate::table_view::table_view(&self.transaction_table)
            .headers([
                "Title".to_owned(),
                "Date".to_owned(),
                "Amount".to_owned(),
                "Source".to_owned(),
                "Destination".to_owned(),
                "Categories".to_owned(),
            ])
            .view(
                move |(transaction, source, destination): &(
                    fm_core::Transaction,
                    fm_core::account::Account,
                    fm_core::account::Account,
                ),
                      categories| {
                    [
                        link(widget::text(transaction.title().clone()))
                            .on_press(Message::ViewTransaction(*transaction.id()))
                            .into(),
                        widget::text(
                            transaction
                                .date()
                                .to_offset(fm_core::get_local_timezone().unwrap())
                                .format(
                                    &time::format_description::parse("[day].[month].[year]")
                                        .unwrap(),
                                )
                                .unwrap(),
                        )
                        .into(),
                        match (self.amount_positive)(transaction.clone()) {
                            Some(true) => colored_currency_display(&transaction.amount()),
                            Some(false) => {
                                colored_currency_display(&transaction.amount().negative())
                            }
                            None => widget::text(transaction.amount().to_string()).into(),
                        },
                        link(widget::text(source.to_string().clone()))
                            .on_press(Message::ViewAccount(*source.id()))
                            .into(),
                        link(widget::text(destination.to_string().clone()))
                            .on_press(Message::ViewAccount(*destination.id()))
                            .into(),
                        widget::row![
                            widget::button(
                                widget::Svg::new(self.edit_svg.clone()).width(iced::Shrink)
                            )
                            .on_press(Message::OpenCategoryPopup(*transaction.id()))
                            .style(widget::button::secondary),
                            widget::text(get_category_text(transaction, categories)),
                        ]
                        .spacing(10)
                        .into(),
                    ]
                },
            )
            .map(Message::TransactionTable);

        crate::modal(
            widget::container(table).padding(10.0),
            if let Some(id) = self.category_popup {
                category_popup(
                    self.transactions
                        .iter()
                        .find(|x| *x.0.id() == id)
                        .unwrap()
                        .0
                        .clone(),
                    self.categories.clone(),
                )
            } else {
                "".into()
            },
            Message::ClosePopup,
            self.category_popup.is_some(),
        )
    }
}

fn get_category_text(
    transaction: &fm_core::Transaction,
    categories: &[fm_core::Category],
) -> String {
    let mut category_text = String::new();
    for category in transaction.categories() {
        if let Some(category) = categories.iter().find(|x| x.id() == category.0) {
            category_text.push_str(category.name());
            category_text.push_str(", ");
        }
    }
    category_text
}

fn category_popup(
    transaction: fm_core::Transaction,
    categories: Vec<fm_core::Category>,
) -> iced::Element<'static, Message> {
    let transaction_id = *transaction.id();
    let mut column = widget::column![];
    for category in categories {
        let category_id = *category.id();
        let transaction_category = transaction.categories().get(&category_id).copied();
        column = column.push(
            widget::row![
                widget::checkbox(
                    category.name(),
                    transaction.categories().contains_key(category.id())
                )
                .on_toggle(move |value| if value {
                    Message::SetCategory {
                        transaction_id,
                        category_id,
                        sign: fm_core::Sign::Positive,
                    }
                } else {
                    Message::RemoveCategory {
                        transaction_id,
                        category_id,
                    }
                }),
                widget::checkbox(
                    "Negative",
                    if let Some(sign) = transaction_category {
                        sign == fm_core::Sign::Negative
                    } else {
                        false
                    }
                )
                .on_toggle_maybe(transaction_category.map(|sign| {
                    move |_| Message::SetCategory {
                        transaction_id,
                        category_id,
                        sign: sign.invert(),
                    }
                })),
            ]
            .spacing(10),
        );
    }

    super::style::container_popup_styling(widget::Container::new(column)).into()
}
