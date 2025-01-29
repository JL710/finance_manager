use std::collections::HashMap;

use fm_core::transaction_filter::Filter;
use iced::widget;

#[derive(Debug, Clone)]
pub enum Action {
    Submit(fm_core::transaction_filter::TransactionFilter),
    None,
}

#[derive(Debug, Clone)]
pub enum InnerMessage {
    Submit,
    ChangeDefaultTimespan(crate::timespan_input::Action),
    NewAccountFilter,
    NewBillFilter,
    NewCategoryFilter,
    NewBudgetFilter,
    DeleteAccount(Filter<fm_core::Id>),
    DeleteBill(Filter<fm_core::Bill>),
    DeleteCategory(Filter<fm_core::Id>),
    DeleteBudget(Filter<fm_core::Id>),
    EditBillTimespan(Filter<fm_core::Bill>, crate::timespan_input::Action),
    EditCategoryTimespan(Filter<fm_core::Id>, crate::timespan_input::Action),
    EditAccountTimespan(Filter<fm_core::Id>, crate::timespan_input::Action),
    EditBudgetTimespan(Filter<fm_core::Id>, crate::timespan_input::Action),
    ChangeAccount(Filter<fm_core::Id>, Filter<fm_core::Id>),
    ChangeBill(Filter<fm_core::Bill>, Filter<fm_core::Bill>),
    ChangeCategory(Filter<fm_core::Id>, Filter<fm_core::Id>),
    ChangeBudget(Filter<fm_core::Id>, Filter<fm_core::Id>),
}

#[derive(Debug, Clone)]
pub struct FilterComponent {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    bills: Vec<fm_core::Bill>,
    budgets: Vec<fm_core::Budget>,
    default_transaction_input: crate::timespan_input::State,
    filter: fm_core::transaction_filter::TransactionFilter,
    bill_timespan_inputs: HashMap<Filter<fm_core::Bill>, crate::timespan_input::State>,
    account_timespan_inputs: HashMap<Filter<fm_core::Id>, crate::timespan_input::State>,
    category_timespan_inputs: HashMap<Filter<fm_core::Id>, crate::timespan_input::State>,
    budget_timespan_inputs: HashMap<Filter<fm_core::Id>, crate::timespan_input::State>,
}

impl FilterComponent {
    pub fn new(
        accounts: Vec<fm_core::account::Account>,
        categories: Vec<fm_core::Category>,
        bills: Vec<fm_core::Bill>,
        budgets: Vec<fm_core::Budget>,
    ) -> Self {
        Self {
            accounts,
            categories,
            bills,
            budgets,
            filter: fm_core::transaction_filter::TransactionFilter::default(),
            default_transaction_input: crate::timespan_input::State::default(),
            bill_timespan_inputs: HashMap::default(),
            account_timespan_inputs: HashMap::default(),
            category_timespan_inputs: HashMap::default(),
            budget_timespan_inputs: HashMap::default(),
        }
    }

    pub fn set_filter(&mut self, new_filter: fm_core::transaction_filter::TransactionFilter) {
        self.filter = new_filter;

        self.default_transaction_input =
            crate::timespan_input::State::new(Some(*self.filter.get_default_timespan()));

        fn set_inputs<I: Clone + std::fmt::Debug + Eq + std::hash::Hash>(
            inputs: &mut HashMap<Filter<I>, crate::timespan_input::State>,
            filters: &Vec<Filter<I>>,
        ) {
            inputs.clear();
            for filter in filters {
                if filter.timespan.is_some() {
                    inputs.insert(
                        filter.clone(),
                        crate::timespan_input::State::new(filter.timespan),
                    );
                }
            }
        }

        set_inputs(
            &mut self.account_timespan_inputs,
            self.filter.get_account_filters(),
        );

        set_inputs(
            &mut self.category_timespan_inputs,
            self.filter.get_category_filters(),
        );

        set_inputs(
            &mut self.budget_timespan_inputs,
            self.filter.get_budget_filters(),
        );

        set_inputs(
            &mut self.bill_timespan_inputs,
            self.filter.get_bill_filters(),
        );
    }

    pub fn with_filter(
        mut self,
        new_filter: fm_core::transaction_filter::TransactionFilter,
    ) -> Self {
        self.set_filter(new_filter);
        self
    }

    pub fn update(&mut self, message: InnerMessage) -> Action {
        match message {
            InnerMessage::Submit => {
                return Action::Submit(self.filter.clone());
            }
            InnerMessage::ChangeDefaultTimespan(action) => {
                self.default_transaction_input.perform(action);
                self.filter
                    .set_default_timespan(self.default_transaction_input.timespan());
            }
            InnerMessage::NewAccountFilter => {
                if !self.accounts.is_empty() {
                    self.filter.add_account(Filter {
                        negated: false,
                        id: Some(*self.accounts.first().unwrap().id()),
                        include: true,
                        timespan: None,
                    });
                }
            }
            InnerMessage::NewBillFilter => {
                if let Some(bill) = self.bills.first() {
                    self.filter.add_bill(Filter {
                        negated: false,
                        id: Some(bill.clone()),
                        include: true,
                        timespan: None,
                    });
                }
            }
            InnerMessage::NewCategoryFilter => {
                if !self.categories.is_empty() {
                    self.filter.add_category(Filter {
                        negated: false,
                        id: Some(*self.categories.first().unwrap().id()),
                        include: true,
                        timespan: None,
                    });
                }
            }
            InnerMessage::NewBudgetFilter => {
                if !self.budgets.is_empty() {
                    self.filter.add_budget(Filter {
                        negated: false,
                        id: Some(*self.budgets.first().unwrap().id()),
                        include: true,
                        timespan: None,
                    });
                }
            }
            InnerMessage::DeleteAccount(filter) => {
                self.account_timespan_inputs.remove(&filter);
                self.filter.delete_account(filter);
            }
            InnerMessage::DeleteBill(filter) => {
                self.bill_timespan_inputs.remove(&filter);
                self.filter.delete_bill(filter);
            }
            InnerMessage::DeleteCategory(filter) => {
                self.category_timespan_inputs.remove(&filter);
                self.filter.delete_category(filter);
            }
            InnerMessage::DeleteBudget(filter) => {
                self.budget_timespan_inputs.remove(&filter);
                self.filter.delete_category(filter);
            }
            InnerMessage::EditAccountTimespan(filter, action) => {
                let mut state = self
                    .account_timespan_inputs
                    .remove(&filter)
                    .unwrap()
                    .clone();
                state.perform(action);
                let mut new_filter = filter.clone();
                new_filter.timespan = Some(state.timespan());
                self.filter.edit_account(filter, new_filter.clone());
                self.account_timespan_inputs.insert(new_filter, state);
            }
            InnerMessage::EditBillTimespan(filter, action) => {
                let mut state = self.bill_timespan_inputs.remove(&filter).unwrap().clone();
                state.perform(action);
                let mut new_filter = filter.clone();
                new_filter.timespan = Some(state.timespan());
                self.filter.edit_bill(filter, new_filter.clone());
                self.bill_timespan_inputs.insert(new_filter, state);
            }
            InnerMessage::EditCategoryTimespan(filter, action) => {
                let mut state = self
                    .category_timespan_inputs
                    .remove(&filter)
                    .unwrap()
                    .clone();
                state.perform(action);
                let mut new_filter = filter.clone();
                new_filter.timespan = Some(state.timespan());
                self.filter.edit_category(filter, new_filter.clone());
                self.category_timespan_inputs.insert(new_filter, state);
            }
            InnerMessage::EditBudgetTimespan(filter, action) => {
                let mut state = self.budget_timespan_inputs.remove(&filter).unwrap().clone();
                state.perform(action);
                let mut new_filter = filter.clone();
                new_filter.timespan = Some(state.timespan());
                self.filter.edit_budget(filter, new_filter.clone());
                self.budget_timespan_inputs.insert(new_filter, state);
            }
            InnerMessage::ChangeAccount(old, new) => {
                self.account_timespan_inputs.remove(&old);
                self.filter.edit_account(old, new.clone());
                if new.timespan.is_some() {
                    self.account_timespan_inputs
                        .insert(new.clone(), crate::timespan_input::State::new(new.timespan));
                }
            }
            InnerMessage::ChangeBill(old, new) => {
                self.bill_timespan_inputs.remove(&old);
                self.filter.edit_bill(old, new.clone());
                if new.timespan.is_some() {
                    self.bill_timespan_inputs
                        .insert(new.clone(), crate::timespan_input::State::new(new.timespan));
                }
            }
            InnerMessage::ChangeCategory(old, new) => {
                self.category_timespan_inputs.remove(&old);
                self.filter.edit_category(old, new.clone());
                if new.timespan.is_some() {
                    self.category_timespan_inputs
                        .insert(new.clone(), crate::timespan_input::State::new(new.timespan));
                }
            }
            InnerMessage::ChangeBudget(old, new) => {
                self.budget_timespan_inputs.remove(&old);
                self.filter.edit_budget(old, new.clone());
                if new.timespan.is_some() {
                    self.budget_timespan_inputs
                        .insert(new.clone(), crate::timespan_input::State::new(new.timespan));
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<InnerMessage> {
        widget::container(super::spaced_column![
            // default timespan
            super::spal_row![
                widget::text("Default Timespan: "),
                crate::timespan_input::timespan_input(&self.default_transaction_input)
                    .view()
                    .map(InnerMessage::ChangeDefaultTimespan),
            ],
            // account filters
            super::spal_row![
                widget::text("Accounts"),
                widget::button("New").on_press(InnerMessage::NewAccountFilter),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                self.filter.get_account_filters(),
                &self.account_timespan_inputs,
                &self.accounts,
                |x| DisplayedAccount { account: x.clone() },
                |x| *x.account.id(),
                |x| *x.id(),
                InnerMessage::ChangeAccount,
                InnerMessage::EditAccountTimespan,
                InnerMessage::DeleteAccount
            )))
            .max_height(150),
            // category filters
            super::spal_row![
                widget::text("Categories"),
                widget::button("New").on_press(InnerMessage::NewCategoryFilter),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                self.filter.get_category_filters(),
                &self.category_timespan_inputs,
                &self.categories,
                |x| DisplayedCategory(x.clone()),
                |x| *x.0.id(),
                |x| *x.id(),
                InnerMessage::ChangeCategory,
                InnerMessage::EditCategoryTimespan,
                InnerMessage::DeleteCategory
            )))
            .max_height(150),
            // bill filters
            super::spal_row![
                widget::text("Bills"),
                widget::button("New").on_press(InnerMessage::NewBillFilter),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                self.filter.get_bill_filters(),
                &self.bill_timespan_inputs,
                &self.bills,
                |x| DisplayedBill(x.clone()),
                |x| x.0.clone(),
                |x| x.clone(),
                InnerMessage::ChangeBill,
                InnerMessage::EditBillTimespan,
                InnerMessage::DeleteBill
            )))
            .max_height(150),
            // budget filters
            super::spal_row![
                widget::text("Budget"),
                widget::button("New").on_press(InnerMessage::NewBudgetFilter),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                self.filter.get_budget_filters(),
                &self.budget_timespan_inputs,
                &self.budgets,
                |x| DisplayedBudget(x.clone()),
                |x| *x.0.id(),
                |x| *x.id(),
                InnerMessage::ChangeBudget,
                InnerMessage::EditBudgetTimespan,
                InnerMessage::DeleteBudget
            )))
            .max_height(150),
            // submit footer
            widget::horizontal_rule(3),
            widget::button("Submit").on_press(InnerMessage::Submit)
        ])
        .padding(super::style::PADDING)
        .style(super::style::container_style_background_weak)
        .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn generate_filter_column<
    'a,
    O: Clone,
    T: ToString + PartialEq + Clone + 'a,
    I: Clone + std::fmt::Debug + PartialEq + Eq + std::hash::Hash,
>(
    filters: &'a Vec<Filter<I>>,
    timespan_input_states: &'a HashMap<Filter<I>, crate::timespan_input::State>,
    options: &'a [O],
    picklist_item_from_option: impl Fn(&O) -> T,
    id_from_picklist_item: fn(&T) -> I,
    id_from_option: fn(&O) -> I,
    change_message: fn(Filter<I>, Filter<I>) -> InnerMessage,
    change_timespan: fn(Filter<I>, crate::timespan_input::Action) -> InnerMessage,
    delete_message: impl Fn(Filter<I>) -> InnerMessage,
) -> iced::Element<'a, InnerMessage> {
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
                widget::checkbox("Specific", filter.id.is_some()).on_toggle(move |x| {
                    if !x || options.is_empty() {
                        (change_message)(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: None,
                                include: filter.include,
                                timespan: filter.timespan,
                            },
                        )
                    } else {
                        (change_message)(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: Some((id_from_option)(&options[0])),
                                include: filter.include,
                                timespan: filter.timespan,
                            },
                        )
                    }
                }),
            ]
            .push_maybe(if filter.id.is_some() {
                Some(widget::pick_list(
                    options
                        .iter()
                        .map(&(picklist_item_from_option))
                        .collect::<Vec<_>>(),
                    Some((picklist_item_from_option)(
                        &options
                            .iter()
                            .find(|x| Some((id_from_option)(x)) == filter.id)
                            .unwrap()
                            .clone(),
                    )),
                    move |x| {
                        (change_message)(
                            filter.clone(),
                            Filter {
                                negated: filter.negated,
                                id: Some((id_from_picklist_item)(&x)),
                                include: filter.include,
                                timespan: filter.timespan,
                            },
                        )
                    },
                ))
            } else {
                None
            })
            .push(
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
            )
            .push(
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
                    },
                ),
            )
            .push_maybe(if filter.timespan.is_some() {
                Some(
                    crate::timespan_input::timespan_input(
                        timespan_input_states.get(filter).unwrap(),
                    )
                    .view()
                    .map(move |x| (change_timespan)(filter.clone(), x)),
                )
            } else {
                None
            })
            .push(widget::row![
                widget::horizontal_space(),
                super::button::delete(Some((delete_message)(filter.clone())))
            ])
            .align_y(iced::Alignment::Center)
            .spacing(30),
        );
    }

    column.into()
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
struct DisplayedCategory(fm_core::Category);

impl std::fmt::Display for DisplayedCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name())
    }
}

impl PartialEq for DisplayedCategory {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

#[derive(Debug, Clone)]
struct DisplayedBill(fm_core::Bill);

impl std::fmt::Display for DisplayedBill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name())
    }
}

impl PartialEq for DisplayedBill {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

#[derive(Debug, Clone)]
struct DisplayedBudget(fm_core::Budget);

impl std::fmt::Display for DisplayedBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name())
    }
}

impl PartialEq for DisplayedBudget {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}
