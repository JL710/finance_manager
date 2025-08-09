use super::{colored_currency_display, link};
use iced::widget;
use std::hash::{Hash, Hasher};

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
    #[allow(clippy::type_complexity)]
    transaction_table: crate::table_view::State<
        (
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        ),
        (Vec<fm_core::Category>, Vec<fm_core::Budget>),
    >,
    amount_positive: Box<dyn Fn(fm_core::Transaction) -> Option<bool>>,
    categories: Vec<fm_core::Category>,
    /// The id of the transaction that has the category popup open if any
    category_popup: Option<fm_core::Id>,
}

impl std::fmt::Debug for TransactionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionTable")
            .field("amount_positive", &"...")
            .field("categories", &self.categories)
            .field("category_popup", &self.category_popup)
            .finish()
    }
}

fn hash_categories(map: &std::collections::HashMap<fm_core::Id, fm_core::Sign>) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    map.keys().collect::<Vec<_>>().hash(&mut hasher);
    hasher.finish()
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
        budgets: Vec<fm_core::Budget>,
        amount_positive: impl Fn(fm_core::Transaction) -> Option<bool> + Copy + 'static,
    ) -> Self {
        let mut categories = categories;
        categories.sort();
        let mut transaction_table =
            crate::table_view::State::new(transactions, (categories.clone(), budgets))
                .sortable_columns([0, 1, 2, 3, 4, 5])
                .sort_by(move |a, b, column_index| match column_index {
                    0 => a.0.title.cmp(&b.0.title),
                    1 => a.0.date.cmp(&b.0.date),
                    2 => {
                        let a = (amount_positive)(a.0.clone()).map_or(
                            a.0.amount().clone(),
                            |positive| {
                                if positive {
                                    a.0.amount().clone()
                                } else {
                                    a.0.amount().negative()
                                }
                            },
                        );
                        let b = (amount_positive)(b.0.clone()).map_or(
                            b.0.amount().clone(),
                            |positive| {
                                if positive {
                                    b.0.amount().clone()
                                } else {
                                    b.0.amount().negative()
                                }
                            },
                        );
                        a.cmp(&b)
                    }
                    3 => a.1.name().cmp(b.1.name()),
                    4 => a.2.name().cmp(b.2.name()),
                    5 => hash_categories(&a.0.categories).cmp(&hash_categories(&b.0.categories)),
                    _ => std::cmp::Ordering::Equal,
                });
        transaction_table.sort(1, true);
        Self {
            categories,
            amount_positive: Box::new(amount_positive),
            transaction_table,
            category_popup: None,
        }
    }

    pub fn reload(
        &mut self,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
        categories: Vec<fm_core::Category>,
        budgets: Vec<fm_core::Budget>,
    ) {
        self.categories = categories;
        self.categories.sort();
        self.transaction_table
            .set_context((self.categories.clone(), budgets));
        if let Some(id) = self.category_popup
            && !transactions.iter().any(|pair| pair.0.id == id)
        {
            self.category_popup = None;
        }
        self.transaction_table
            .edit_items(|items| *items = transactions);
    }

    pub fn change_transactions(
        &mut self,
        transactions: Vec<(
            fm_core::Transaction,
            fm_core::account::Account,
            fm_core::account::Account,
        )>,
    ) {
        self.transaction_table.set_items(transactions);
        self.transaction_table.sort(1, true);
    }

    pub fn transactions(
        &self,
    ) -> &Vec<(
        fm_core::Transaction,
        fm_core::account::Account,
        fm_core::account::Account,
    )> {
        self.transaction_table.items()
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_controller: fm_core::FMController<impl fm_core::FinanceManager>,
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
                    .transaction_table
                    .items()
                    .iter()
                    .find(|x| x.0.id == transaction_id)
                    .unwrap()
                    .0;
                let transaction_id = transaction.id;
                let mut categories = transaction.categories.clone();
                categories.remove(&category_id);

                Action::Task(iced::Task::future(async move {
                    let new_transaction = finance_controller
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
                    .transaction_table
                    .items()
                    .iter()
                    .find(|x| x.0.id == transaction_id)
                    .unwrap()
                    .0;
                let transaction_id = transaction.id;
                let mut categories = transaction.categories.clone();
                categories.insert(category_id, sign);

                Action::Task(iced::Task::future(async move {
                    let new_transaction = finance_controller
                        .update_transaction_categories(transaction_id, categories)
                        .await
                        .unwrap();
                    Message::TransactionCategoryUpdated(new_transaction)
                }))
            }
            Message::TransactionCategoryUpdated(transaction) => {
                let index = self
                    .transaction_table
                    .items()
                    .iter()
                    .position(|x| x.0.id == transaction.id)
                    .unwrap();
                self.transaction_table.edit_items(move |transactions| {
                    transactions[index].0 = transaction.clone();
                });
                Action::None
            }
            Message::TransactionTable(inner) => {
                match self.transaction_table.perform(inner) {
                    crate::table_view::Action::OuterMessage(m) => {
                        return self.update(m, finance_controller);
                    }
                    crate::table_view::Action::None => {}
                    crate::table_view::Action::Task(task) => {
                        return Action::Task(task.map(Message::TransactionTable));
                    }
                }
                Action::None
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let table = crate::table_view::table_view(&self.transaction_table)
            .headers([
                "Title",
                "Date",
                "Amount",
                "Source",
                "Destination",
                "Categories",
                "Budget",
            ])
            .view(
                move |(transaction, source, destination): &(
                    fm_core::Transaction,
                    fm_core::account::Account,
                    fm_core::account::Account,
                ),
                      context| {
                    [
                        link(transaction.title.as_str())
                            .on_press(Message::ViewTransaction(transaction.id))
                            .into(),
                        widget::text(super::date_time::to_date_string(transaction.date.date()))
                            .into(),
                        match (self.amount_positive)(transaction.clone()) {
                            Some(true) => colored_currency_display(transaction.amount()),
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
                        super::spal_row![
                            widget::button(icons::pencil_fill())
                                .on_press(Message::OpenCategoryPopup(transaction.id))
                                .style(widget::button::secondary),
                            widget::text(get_category_text(transaction, &context.0)),
                        ]
                        .into(),
                        if let Some(budget) = transaction.budget {
                            widget::text(
                                context
                                    .1
                                    .iter()
                                    .find(|x| x.id == budget.0)
                                    .unwrap()
                                    .name
                                    .as_str(),
                            )
                            .into()
                        } else {
                            widget::text("").into()
                        },
                    ]
                },
            )
            .map(Message::TransactionTable);

        crate::modal(
            widget::container(table),
            if let Some(id) = self.category_popup {
                category_popup(
                    self.transaction_table
                        .items()
                        .iter()
                        .find(|x| x.0.id == id)
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
    let mut category_names = transaction
        .categories
        .iter()
        .map(|x| &categories.iter().find(|c| c.id == *x.0).unwrap().name)
        .collect::<Vec<_>>();
    category_names.sort();
    let mut category_iter = category_names.into_iter();
    let mut category_text = String::new();
    if let Some(first) = category_iter.next() {
        category_text.push_str(first);
    }
    for category in category_iter {
        category_text.push_str(", ");
        category_text.push_str(category);
    }
    category_text
}

fn category_popup(
    transaction: fm_core::Transaction,
    categories: Vec<fm_core::Category>,
) -> iced::Element<'static, Message> {
    let mut column = super::spaced_column![];
    for category in categories {
        let category_id = category.id;
        let transaction_category = transaction.categories.get(&category_id).copied();
        column = column.push(super::spal_row![
            widget::checkbox(
                &category.name,
                transaction.categories.contains_key(&category.id)
            )
            .on_toggle(move |value| if value {
                Message::SetCategory {
                    transaction_id: transaction.id,
                    category_id,
                    sign: fm_core::Sign::Positive,
                }
            } else {
                Message::RemoveCategory {
                    transaction_id: transaction.id,
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
                    transaction_id: transaction.id,
                    category_id,
                    sign: sign.invert(),
                }
            })),
        ]);
    }

    style::container_popup_styling(widget::container(column)).into()
}
