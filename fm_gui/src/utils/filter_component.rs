use super::timespan_input;
use fm_core::transaction_filter::{Filter, TransactionFilter};
use iced::widget;

pub struct FilterComponent<'a, Message> {
    filter: TransactionFilter,
    on_submit: Box<dyn Fn(TransactionFilter) -> Message + 'a>,
    accounts: &'a Vec<fm_core::account::Account>,
    categories: &'a Vec<fm_core::Category>,
    bills: &'a Vec<fm_core::Bill>,
    budgets: &'a Vec<fm_core::Budget>,
}

impl<'a, Message: 'a> FilterComponent<'a, Message> {
    pub fn new(
        filter: TransactionFilter,
        on_submit: impl Fn(TransactionFilter) -> Message + 'a,
        accounts: &'a Vec<fm_core::account::Account>,
        categories: &'a Vec<fm_core::Category>,
        bills: &'a Vec<fm_core::Bill>,
        budgets: &'a Vec<fm_core::Budget>,
    ) -> Self {
        Self {
            filter,
            on_submit: Box::new(on_submit),
            accounts,
            categories,
            bills,
            budgets,
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
    ChangeAccount(Filter<fm_core::Id>, Filter<fm_core::Id>),
    NewAccount,
    DeleteAccount(Filter<fm_core::Id>),
    ChangeCategory(Filter<fm_core::Id>, Filter<fm_core::Id>),
    NewCategory,
    DeleteCategory(Filter<fm_core::Id>),
    NewBill,
    ChangeBill(Filter<fm_core::Bill>, Filter<fm_core::Bill>),
    DeleteBill(Filter<fm_core::Bill>),
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

impl<Message> iced::widget::Component<Message> for FilterComponent<Message> {
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
                    self.filter.add_account(Filter {
                        negated: false,
                        id: *self.accounts.first().unwrap().id(),
                        include: true,
                        timespan: None,
                    });
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
                    self.filter.add_category(Filter {
                        negated: false,
                        id: *self.categories.first().unwrap().id(),
                        include: true,
                        timespan: None,
                    });
                }
            }
            ComponentMessage::DeleteCategory(category) => {
                self.filter.delete_category(category);
            }
            ComponentMessage::NewBill => {
                if let Some(bill) = self.bills.first() {
                    self.filter.add_bill(Filter {
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
                    widget::checkbox("Negate", filter.negated).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(
                            filter.clone(),
                            Filter {
                                negated: x,
                                id: filter.id,
                                include: filter.include,
                                timespan: filter.timespan,
                            },
                        )
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
                                .find(|x| *x.id() == filter.id)
                                .unwrap()
                                .clone()
                        }),
                        |x| ComponentMessage::ChangeAccount(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: *x.account.id(),
                                include: filter.include,
                                timespan: filter.timespan
                            }
                        )
                    ),
                    widget::checkbox("Exclude", !filter.include).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: filter.id,
                                include: !x,
                                timespan: filter.timespan,
                            },
                        )
                    }),
                    widget::checkbox("Custom Timespan", filter.timespan.is_some()).on_toggle(|x| {
                        ComponentMessage::ChangeAccount(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: filter.id,
                                include: filter.include,
                                timespan: if x { Some((None, None)) } else { None },
                            },
                        )
                    })
                ]
                .push_maybe(if filter.timespan.is_some() {
                    Some(
                        timespan_input::TimespanInput::new(
                            |x| {
                                ComponentMessage::ChangeAccount(
                                    filter.clone(),
                                    Filter {
                                        negated: filter.negated,
                                        id: filter.id,
                                        include: filter.include,
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
                        .on_press(ComponentMessage::DeleteAccount(filter.clone()))
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
                widget::container(widget::scrollable(generate_filter_column(
                    self.filter.get_account_filters(),
                    self.accounts,
                    |x| DisplayedAccount { account: x.clone() },
                    |x| *x.account.id(),
                    |x| *x.id(),
                    ComponentMessage::ChangeAccount,
                    ComponentMessage::DeleteAccount
                )))
                .max_height(150),
                // category filters
                widget::row![
                    widget::text("Categories"),
                    widget::button("New").on_press(ComponentMessage::NewCategory),
                    widget::horizontal_rule(3)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                widget::container(widget::scrollable(generate_filter_column(
                    self.filter.get_category_filters(),
                    self.categories,
                    |x| DisplayedCategory {
                        category: x.clone()
                    },
                    |x| { *x.category.id() },
                    |x| *x.id(),
                    ComponentMessage::ChangeCategory,
                    ComponentMessage::DeleteCategory
                )))
                .max_height(150),
                // bill filters
                widget::row![
                    widget::text("Bills"),
                    widget::button("New").on_press(ComponentMessage::NewBill),
                    widget::horizontal_rule(3)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
                widget::container(widget::scrollable(generate_filter_column(
                    self.filter.get_bill_filters(),
                    self.bills,
                    |x| DisplayedBill { bill: x.clone() },
                    |x| x.bill.clone(),
                    |x| x.clone(),
                    ComponentMessage::ChangeBill,
                    ComponentMessage::DeleteBill
                )))
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

fn generate_filter_column<
    'a,
    O: Clone,
    T: ToString + PartialEq + Clone + 'a,
    I: Clone + std::fmt::Debug + PartialEq,
>(
    filters: &'a Vec<Filter<I>>,
    options: &'a [O],
    picklist_item_from_option: impl Fn(&O) -> T,
    id_from_picklist_item: fn(&T) -> I,
    id_from_option: impl Fn(&O) -> I,
    change_message: fn(Filter<I>, Filter<I>) -> ComponentMessage,
    delete_message: impl Fn(Filter<I>) -> ComponentMessage,
) -> iced::Element<'a, ComponentMessage> {
    let mut column = widget::Column::new().width(iced::Fill);

    for filter in filters {
        column = column.push(
            widget::row![
                widget::checkbox("Negate", filter.negated).on_toggle(move |x| {
                    (change_message)(
                        filter.clone(),
                        Filter {
                            negated: x,
                            id: filter.id.clone(),
                            include: filter.include,
                            timespan: filter.timespan,
                        },
                    )
                }),
                widget::pick_list(
                    options
                        .iter()
                        .map(&(picklist_item_from_option))
                        .collect::<Vec<_>>(),
                    Some((picklist_item_from_option)(
                        &options
                            .iter()
                            .find(|x| (id_from_option)(x) == filter.id)
                            .unwrap()
                            .clone()
                    )),
                    move |x| (change_message)(
                        filter.clone(),
                        Filter {
                            negated: filter.negated,
                            id: (id_from_picklist_item)(&x),
                            include: filter.include,
                            timespan: filter.timespan,
                        }
                    )
                ),
                widget::checkbox("Exclude", !filter.include).on_toggle(move |x| {
                    (change_message)(
                        filter.clone(),
                        Filter {
                            negated: filter.negated,
                            id: filter.id.clone(),
                            include: !x,
                            timespan: filter.timespan,
                        },
                    )
                }),
                widget::checkbox("Custom Timespan", filter.timespan.is_some()).on_toggle(
                    move |x| {
                        (change_message)(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: filter.id.clone(),
                                include: filter.include,
                                timespan: if x { Some((None, None)) } else { None },
                            },
                        )
                    }
                )
            ]
            .push_maybe(if filter.timespan.is_some() {
                Some(
                    timespan_input::TimespanInput::new(
                        move |x| {
                            (change_message)(
                                filter.clone(),
                                Filter {
                                    negated: filter.negated,
                                    id: filter.id.clone(),
                                    include: filter.include,
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
                widget::button("Delete").on_press((delete_message)(filter.clone()))
            ])
            .align_y(iced::Alignment::Center)
            .spacing(30),
        );
    }

    column.into()
}
