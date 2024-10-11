use super::timespan_input;
use fm_core::transaction_filter::{self, TransactionFilter};
use iced::widget;

pub struct FilterComponent<'a, Message> {
    filter: TransactionFilter,
    on_submit: Box<dyn Fn(TransactionFilter) -> Message + 'a>,
    accounts: &'a Vec<fm_core::account::Account>,
    categories: &'a Vec<fm_core::Category>,
    bills: &'a Vec<fm_core::Bill>,
}

impl<'a, Message: 'a> FilterComponent<'a, Message> {
    pub fn new(
        filter: TransactionFilter,
        on_submit: impl Fn(TransactionFilter) -> Message + 'a,
        accounts: &'a Vec<fm_core::account::Account>,
        categories: &'a Vec<fm_core::Category>,
        bills: &'a Vec<fm_core::Bill>,
    ) -> Self {
        Self {
            filter,
            on_submit: Box::new(on_submit),
            accounts,
            categories,
            bills,
        }
    }

    pub fn into_element(self) -> iced::Element<'a, Message> {
        iced::widget::component(self)
    }
}

#[derive(Default)]
pub struct State {}

#[derive(Debug, Clone)]
pub enum ComponentMessage {
    Submit,
    ChangeDefaultTimespan(fm_core::Timespan),
    ChangeAccount(
        transaction_filter::AccountFilter,
        transaction_filter::AccountFilter,
    ),
    NewAccount,
    DeleteAccount(transaction_filter::AccountFilter),
    ChangeCategory(
        transaction_filter::CategoryFilter,
        transaction_filter::CategoryFilter,
    ),
    NewCategory,
    DeleteCategory(transaction_filter::CategoryFilter),
    NewBill,
    ChangeBill(
        transaction_filter::Filter<fm_core::Bill>,
        transaction_filter::Filter<fm_core::Bill>,
    ),
    DeleteBill(transaction_filter::Filter<fm_core::Bill>),
}

#[derive(Debug, Clone)]
struct DisplayedAccount {
    account: fm_core::account::Account,
}

impl std::fmt::Display for DisplayedAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.account.name())
    }
}

impl PartialEq for DisplayedAccount {
    fn eq(&self, other: &Self) -> bool {
        self.account.id() == other.account.id()
    }
}

#[derive(Debug, Clone)]
struct DisplayedCategory {
    category: fm_core::Category,
}

impl std::fmt::Display for DisplayedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.category.name())
    }
}

impl PartialEq for DisplayedCategory {
    fn eq(&self, other: &Self) -> bool {
        self.category.id() == other.category.id()
    }
}

#[derive(Debug, Clone)]
struct DisplayedBill {
    bill: fm_core::Bill,
}

impl std::fmt::Display for DisplayedBill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bill.name())
    }
}

impl PartialEq for DisplayedBill {
    fn eq(&self, other: &Self) -> bool {
        self.bill.id() == other.bill.id()
    }
}

impl<'a, Message> iced::widget::Component<Message> for FilterComponent<'a, Message> {
    type State = State;
    type Event = ComponentMessage;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            ComponentMessage::Submit => return Some((self.on_submit)(self.filter.clone())),
            ComponentMessage::ChangeDefaultTimespan(timespan) => {
                self.filter.set_default_timespan(timespan);
            }
            ComponentMessage::ChangeAccount(old, new) => {
                self.filter.edit_account(old, new);
            }
            ComponentMessage::NewAccount => {
                if !self.accounts.is_empty() {
                    self.filter.add_account((
                        false,
                        *self.accounts.first().unwrap().id(),
                        true,
                        None,
                    ));
                }
            }
            ComponentMessage::DeleteAccount(account) => {
                self.filter.delete_account(account);
            }
            ComponentMessage::ChangeCategory(old, new) => {
                self.filter.edit_category(old, new);
            }
            ComponentMessage::NewCategory => {
                if !self.categories.is_empty() {
                    self.filter.add_category((
                        false,
                        *self.categories.first().unwrap().id(),
                        true,
                        None,
                    ));
                }
            }
            ComponentMessage::DeleteCategory(category) => {
                self.filter.delete_category(category);
            }
            ComponentMessage::NewBill => {
                if let Some(bill) = self.bills.first() {
                    self.filter.add_bill(fm_core::transaction_filter::Filter {
                        negated: false,
                        id: bill.clone(),
                        include: true,
                        timespan: None,
                    });
                }
            }
            ComponentMessage::DeleteBill(bill) => {
                self.filter.delete_bill(bill);
            }
            ComponentMessage::ChangeBill(old, new) => {
                self.filter.edit_bill(old, new);
            }
        }
        None
    }

    fn view(&self, _state: &Self::State) -> iced::Element<'_, Self::Event> {
        let mut account_column = widget::Column::new();
        for filter in self.filter.get_account_filters() {
            account_column = account_column.push(
                widget::row![
                    widget::checkbox("Negate", filter.0).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(*filter, (x, filter.1, filter.2, filter.3))
                    }),
                    widget::pick_list(
                        self.accounts
                            .iter()
                            .map(|x| DisplayedAccount { account: x.clone() })
                            .collect::<Vec<_>>(),
                        Some(DisplayedAccount {
                            account: self
                                .accounts
                                .iter()
                                .find(|x| *x.id() == filter.1)
                                .unwrap()
                                .clone()
                        }),
                        |x| ComponentMessage::ChangeAccount(
                            *filter,
                            (filter.0, *x.account.id(), filter.2, filter.3)
                        )
                    ),
                    widget::checkbox("Exclude", !filter.2).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(*filter, (filter.0, filter.1, !x, filter.3))
                    }),
                    widget::checkbox("Custom Timespan", filter.3.is_some()).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(
                            *filter,
                            (
                                filter.0,
                                filter.1,
                                filter.2,
                                if x { Some((None, None)) } else { None },
                            ),
                        )
                    })
                ]
                .push_maybe(if filter.3.is_some() {
                    Some(
                        timespan_input::TimespanInput::new(
                            |x| {
                                ComponentMessage::ChangeAccount(
                                    *filter,
                                    (filter.0, filter.1, filter.2, Some(x)),
                                )
                            },
                            None,
                        )
                        .into_element(),
                    )
                } else {
                    None
                })
                .push(widget::row![
                    widget::horizontal_space(),
                    widget::button("Delete").on_press(ComponentMessage::DeleteAccount(*filter))
                ])
                .align_y(iced::Alignment::Center)
                .spacing(30),
            );
        }

        let mut category_column = widget::Column::new();
        for filter in self.filter.get_category_filters() {
            category_column = category_column.push(
                widget::row![
                    widget::checkbox("Negate", filter.0).on_toggle(|x| {
                        ComponentMessage::ChangeCategory(*filter, (x, filter.1, filter.2, filter.3))
                    }),
                    widget::pick_list(
                        self.categories
                            .iter()
                            .map(|x| DisplayedCategory {
                                category: x.clone()
                            })
                            .collect::<Vec<_>>(),
                        Some(DisplayedCategory {
                            category: self
                                .categories
                                .iter()
                                .find(|x| *x.id() == filter.1)
                                .unwrap()
                                .clone()
                        }),
                        |x| ComponentMessage::ChangeCategory(
                            *filter,
                            (filter.0, *x.category.id(), filter.2, filter.3)
                        )
                    ),
                    widget::checkbox("Exclude", !filter.2).on_toggle(|x| {
                        ComponentMessage::ChangeCategory(
                            *filter,
                            (filter.0, filter.1, !x, filter.3),
                        )
                    }),
                    widget::checkbox("Custom Timespan", filter.3.is_some()).on_toggle(|x| {
                        ComponentMessage::ChangeCategory(
                            *filter,
                            (
                                filter.0,
                                filter.1,
                                filter.2,
                                if x { Some((None, None)) } else { None },
                            ),
                        )
                    })
                ]
                .push_maybe(if filter.3.is_some() {
                    Some(
                        timespan_input::TimespanInput::new(
                            |x| {
                                ComponentMessage::ChangeCategory(
                                    *filter,
                                    (filter.0, filter.1, filter.2, Some(x)),
                                )
                            },
                            None,
                        )
                        .into_element(),
                    )
                } else {
                    None
                })
                .push(widget::row![
                    widget::horizontal_space(),
                    widget::button("Delete").on_press(ComponentMessage::DeleteCategory(*filter))
                ])
                .align_y(iced::Alignment::Center)
                .spacing(30),
            );
        }

        let mut bill_column = widget::Column::new();
        for bill_filter in self.filter.get_bill_filters() {
            bill_column = bill_column.push(
                widget::row![
                    widget::checkbox("Negate", bill_filter.negated).on_toggle(|x| {
                        ComponentMessage::ChangeBill(
                            bill_filter.clone(),
                            fm_core::transaction_filter::Filter {
                                negated: x,
                                id: bill_filter.id.clone(),
                                include: bill_filter.include,
                                timespan: bill_filter.timespan,
                            },
                        )
                    }),
                    widget::pick_list(
                        self.bills
                            .iter()
                            .map(|x| DisplayedBill { bill: x.clone() })
                            .collect::<Vec<_>>(),
                        Some(DisplayedBill {
                            bill: bill_filter.id.clone()
                        }),
                        |x| ComponentMessage::ChangeBill(
                            bill_filter.clone(),
                            fm_core::transaction_filter::Filter {
                                negated: bill_filter.negated,
                                id: x.bill,
                                include: bill_filter.include,
                                timespan: bill_filter.timespan
                            }
                        )
                    ),
                    widget::checkbox("Exclude", !bill_filter.include).on_toggle(|x| {
                        ComponentMessage::ChangeBill(
                            bill_filter.clone(),
                            fm_core::transaction_filter::Filter {
                                negated: bill_filter.negated,
                                id: bill_filter.id.clone(),
                                include: !x,
                                timespan: bill_filter.timespan,
                            },
                        )
                    }),
                    widget::checkbox("Custom Timespan", bill_filter.timespan.is_some()).on_toggle(
                        |x| {
                            ComponentMessage::ChangeBill(
                                bill_filter.clone(),
                                fm_core::transaction_filter::Filter {
                                    negated: bill_filter.negated,
                                    id: bill_filter.id.clone(),
                                    include: bill_filter.include,
                                    timespan: if x { Some((None, None)) } else { None },
                                },
                            )
                        }
                    )
                ]
                .push_maybe(if bill_filter.timespan.is_some() {
                    Some(
                        timespan_input::TimespanInput::new(
                            |x| {
                                ComponentMessage::ChangeBill(
                                    bill_filter.clone(),
                                    fm_core::transaction_filter::Filter {
                                        negated: bill_filter.negated,
                                        id: bill_filter.id.clone(),
                                        include: bill_filter.include,
                                        timespan: Some(x),
                                    },
                                )
                            },
                            None,
                        )
                        .into_element(),
                    )
                } else {
                    None
                })
                .push(widget::row![
                    widget::horizontal_space(),
                    widget::button("Delete")
                        .on_press(ComponentMessage::DeleteBill(bill_filter.clone()))
                ])
                .align_y(iced::Alignment::Center)
                .spacing(30),
            );
        }

        widget::container(
            widget::column![
                // default timespan
                widget::row![
                    widget::text("Default Timespan: "),
                    timespan_input::TimespanInput::new(
                        ComponentMessage::ChangeDefaultTimespan,
                        Some(*self.filter.get_default_timespan())
                    )
                    .into_element(),
                ],
                // account filters
                widget::row![
                    widget::text("Accounts"),
                    widget::button("New").on_press(ComponentMessage::NewAccount),
                    widget::horizontal_rule(3)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                widget::container(widget::scrollable(account_column.width(iced::Length::Fill)))
                    .max_height(150),
                // category filters
                widget::row![
                    widget::text("Categories"),
                    widget::button("New").on_press(ComponentMessage::NewCategory),
                    widget::horizontal_rule(3)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                widget::container(widget::scrollable(
                    category_column.width(iced::Length::Fill)
                ))
                .max_height(150),
                // bill filters
                widget::row![
                    widget::text("Bills"),
                    widget::button("New").on_press(ComponentMessage::NewBill),
                    widget::horizontal_rule(3)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                widget::container(widget::scrollable(bill_column.width(iced::Length::Fill)))
                    .max_height(150),
                // submit footer
                widget::horizontal_rule(3),
                widget::button("Submit").on_press(ComponentMessage::Submit)
            ]
            .spacing(10),
        )
        .padding(10)
        .style(super::style::container_style_background_weak)
        .into()
    }
}
