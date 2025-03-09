use crate::date_time::date_span_input;
use fm_core::transaction_filter::Filter;
use iced::widget;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Action {
    Submit(fm_core::transaction_filter::TransactionFilter),
    None,
}

#[derive(Debug, Clone)]
pub enum InnerMessage {
    Submit,
    ChangeDefaultTimespan(date_span_input::Action),
    NewAccountFilter,
    NewBillFilter,
    NewCategoryFilter,
    NewBudgetFilter,
    AccountEntryMessage(
        usize,
        filter_entry::MessageContainer<fm_core::account::Account>,
    ),
    BillEntryMessage(usize, filter_entry::MessageContainer<Arc<fm_core::Bill>>),
    CategoryEntryMessage(usize, filter_entry::MessageContainer<fm_core::Category>),
    BudgetEntryMessage(usize, filter_entry::MessageContainer<fm_core::Budget>),
}

#[derive(Debug)]
pub struct FilterComponent {
    accounts: Vec<fm_core::account::Account>,
    categories: Vec<fm_core::Category>,
    bills: Vec<Arc<fm_core::Bill>>,
    budgets: Vec<fm_core::Budget>,
    default_transaction_input: date_span_input::State,
    bill_filter_entries: Vec<filter_entry::FilterEntry<Arc<fm_core::Bill>, Arc<fm_core::Bill>>>,
    account_filter_entries: Vec<filter_entry::FilterEntry<fm_core::account::Account, fm_core::Id>>,
    category_filter_entries: Vec<filter_entry::FilterEntry<fm_core::Category, fm_core::Id>>,
    budget_filter_entries: Vec<filter_entry::FilterEntry<fm_core::Budget, fm_core::Id>>,
}

impl FilterComponent {
    pub fn new(
        accounts: Vec<fm_core::account::Account>,
        categories: Vec<fm_core::Category>,
        bills: Vec<fm_core::Bill>,
        budgets: Vec<fm_core::Budget>,
    ) -> Self {
        let mut categories = categories;
        categories.sort_by(|a, b| a.name.cmp(&b.name));
        let mut accounts = accounts;
        accounts.sort_by(|a, b| a.name().cmp(b.name()));
        let mut bills = bills.into_iter().map(Arc::new).collect::<Vec<_>>();
        bills.sort_by(|a, b| a.name.cmp(&b.name));
        let mut budgets = budgets;
        budgets.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            accounts,
            categories,
            bills,
            budgets,
            default_transaction_input: date_span_input::State::default(),
            bill_filter_entries: Vec::default(),
            account_filter_entries: Vec::default(),
            category_filter_entries: Vec::default(),
            budget_filter_entries: Vec::default(),
        }
    }

    pub fn set_filter(&mut self, new_filter: fm_core::transaction_filter::TransactionFilter) {
        self.default_transaction_input =
            date_span_input::State::new(Some(new_filter.default_timespan));

        fn set_inputs<
            T: Clone + std::fmt::Debug + std::fmt::Display + 'static,
            ID: Clone + std::fmt::Debug + Eq,
        >(
            inputs: &mut Vec<filter_entry::FilterEntry<T, ID>>,
            filters: Vec<Filter<ID>>,
            options: Vec<T>,
            t_to_id: impl Fn(&T) -> ID + Clone + 'static,
        ) {
            inputs.clear();
            for filter in filters {
                inputs.push(filter_entry::FilterEntry::new(
                    filter,
                    options.clone(),
                    t_to_id.clone(),
                ))
            }
        }

        set_inputs(
            &mut self.account_filter_entries,
            new_filter.accounts,
            self.accounts.clone(),
            |acc| *acc.id(),
        );

        set_inputs(
            &mut self.category_filter_entries,
            new_filter.categories,
            self.categories.clone(),
            |category| category.id,
        );

        set_inputs(
            &mut self.budget_filter_entries,
            new_filter.budgets,
            self.budgets.clone(),
            |budget| budget.id,
        );

        set_inputs(
            &mut self.bill_filter_entries,
            new_filter
                .bills
                .into_iter()
                .map(|x| Filter {
                    negated: x.negated,
                    id: x.id.map(Arc::new),
                    include: x.include,
                    timespan: x.timespan,
                })
                .collect(),
            self.bills.clone(),
            |bill| bill.clone(),
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
                let mut filter = fm_core::transaction_filter::TransactionFilter {
                    default_timespan: self.default_transaction_input.timespan(),
                    ..Default::default()
                };
                for bill_entry in &self.bill_filter_entries {
                    let bill_filter = bill_entry.get_filter().clone();
                    let bill_filter = Filter {
                        negated: bill_filter.negated,
                        id: bill_filter.id.map(|x| (*x).clone()),
                        include: bill_filter.include,
                        timespan: bill_filter.timespan,
                    };
                    filter.bills.push(bill_filter);
                }
                for acc_entry in &self.account_filter_entries {
                    filter.accounts.push(acc_entry.get_filter().clone());
                }
                for category_entry in &self.category_filter_entries {
                    filter.categories.push(category_entry.get_filter().clone());
                }
                for budget_entry in &self.budget_filter_entries {
                    filter.budgets.push(budget_entry.get_filter().clone());
                }
                return Action::Submit(filter);
            }
            InnerMessage::ChangeDefaultTimespan(action) => {
                self.default_transaction_input.perform(action);
            }
            InnerMessage::NewAccountFilter => {
                if !self.accounts.is_empty() {
                    self.account_filter_entries
                        .push(filter_entry::FilterEntry::new(
                            Filter {
                                negated: false,
                                id: Some(*self.accounts[0].id()),
                                include: true,
                                timespan: None,
                            },
                            self.accounts.clone(),
                            |x| *x.id(),
                        ));
                }
            }
            InnerMessage::NewBillFilter => {
                if let Some(bill) = self.bills.first() {
                    self.bill_filter_entries
                        .push(filter_entry::FilterEntry::new(
                            Filter {
                                negated: false,
                                id: Some(bill.clone()),
                                include: true,
                                timespan: None,
                            },
                            self.bills.clone(),
                            |x| x.clone(),
                        ));
                }
            }
            InnerMessage::NewCategoryFilter => {
                if !self.categories.is_empty() {
                    self.category_filter_entries
                        .push(filter_entry::FilterEntry::new(
                            Filter {
                                negated: false,
                                id: Some(self.categories[0].id),
                                include: true,
                                timespan: None,
                            },
                            self.categories.clone(),
                            |x| x.id,
                        ));
                }
            }
            InnerMessage::NewBudgetFilter => {
                if !self.budgets.is_empty() {
                    self.budget_filter_entries
                        .push(filter_entry::FilterEntry::new(
                            Filter {
                                negated: false,
                                id: Some(self.budgets[0].id),
                                include: true,
                                timespan: None,
                            },
                            self.budgets.clone(),
                            |x| x.id,
                        ));
                }
            }
            InnerMessage::AccountEntryMessage(index, m) => {
                match self
                    .account_filter_entries
                    .get_mut(index)
                    .unwrap()
                    .update(m)
                {
                    filter_entry::Action::Delete => {
                        self.account_filter_entries.remove(index).get_filter();
                    }
                    filter_entry::Action::None => {}
                }
            }
            InnerMessage::BillEntryMessage(index, m) => {
                match self.bill_filter_entries.get_mut(index).unwrap().update(m) {
                    filter_entry::Action::Delete => {
                        self.bill_filter_entries.remove(index).get_filter();
                    }
                    filter_entry::Action::None => {}
                }
            }
            InnerMessage::BudgetEntryMessage(index, m) => {
                match self.budget_filter_entries.get_mut(index).unwrap().update(m) {
                    filter_entry::Action::Delete => {
                        self.budget_filter_entries.remove(index).get_filter();
                    }
                    filter_entry::Action::None => {}
                }
            }
            InnerMessage::CategoryEntryMessage(index, m) => {
                match self
                    .category_filter_entries
                    .get_mut(index)
                    .unwrap()
                    .update(m)
                {
                    filter_entry::Action::Delete => {
                        self.category_filter_entries.remove(index).get_filter();
                    }
                    filter_entry::Action::None => {}
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<InnerMessage> {
        widget::container(super::spaced_column![
            // default timespan
            super::spal_row![
                "Default Timespan: ",
                date_span_input::date_span_input(&self.default_transaction_input)
                    .view()
                    .map(InnerMessage::ChangeDefaultTimespan),
            ],
            // account filters
            super::spal_row![
                "Accounts",
                super::button::new("New", Some(InnerMessage::NewAccountFilter)),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                &self.account_filter_entries,
                InnerMessage::AccountEntryMessage
            )))
            .max_height(150),
            // category filters
            super::spal_row![
                "Categories",
                super::button::new("New", Some(InnerMessage::NewCategoryFilter)),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                &self.category_filter_entries,
                InnerMessage::CategoryEntryMessage
            )))
            .max_height(150),
            // bill filters
            super::spal_row![
                "Bills",
                super::button::new("New", Some(InnerMessage::NewBillFilter)),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                &self.bill_filter_entries,
                InnerMessage::BillEntryMessage
            )))
            .max_height(150),
            // budget filters
            super::spal_row![
                "Budget",
                super::button::new("New", Some(InnerMessage::NewBudgetFilter)),
                widget::horizontal_rule(3)
            ]
            .align_y(iced::Alignment::Center),
            widget::container(widget::scrollable(generate_filter_column(
                &self.budget_filter_entries,
                InnerMessage::BudgetEntryMessage
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

fn generate_filter_column<
    T: Clone + std::fmt::Debug + std::fmt::Display + 'static,
    ID: Clone + std::fmt::Debug + PartialEq + Eq,
>(
    entries: &[filter_entry::FilterEntry<T, ID>],
    f: fn(usize, filter_entry::MessageContainer<T>) -> InnerMessage,
) -> iced::Element<'_, InnerMessage> {
    let mut col = crate::spaced_column!();
    for (index, entry) in entries.iter().enumerate() {
        col = col.push(entry.view().map(move |m| (f)(index, m)));
    }
    col.into()
}

mod filter_entry {
    use crate::date_time::date_span_input;
    use fm_core::transaction_filter::Filter;
    use iced::widget;

    #[derive(Debug, Clone)]
    pub struct MessageContainer<T>(Message<T>);

    #[derive(Debug, Clone)]
    enum Message<T> {
        Delete,
        Negate(bool),
        Exclude(bool),
        Specific(bool),
        CustomTimespan(bool),
        TimespanInput(date_span_input::Action),
        SpecificSelected(T),
    }

    pub enum Action {
        Delete,
        None,
    }

    pub struct FilterEntry<T: Clone, ID: Clone + std::fmt::Debug> {
        filter: Filter<ID>,
        options: Vec<T>,
        timespan_input: date_span_input::State,
        specific_combobox: widget::combo_box::State<T>,
        specific_combobox_selected: Option<T>,
        t_to_id: Box<dyn Fn(&T) -> ID>,
    }

    impl<T: Clone + std::fmt::Debug, ID: Clone + std::fmt::Debug> std::fmt::Debug
        for FilterEntry<T, ID>
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "FilterEntry<{:?} {:?} {:?} {:?} {:?}>",
                self.filter,
                self.options,
                self.timespan_input,
                self.specific_combobox,
                self.specific_combobox_selected
            )
        }
    }

    impl<T: Clone + std::fmt::Display + 'static, ID: Clone + std::fmt::Debug + PartialEq + Eq>
        FilterEntry<T, ID>
    {
        pub fn new(
            filter: Filter<ID>,
            options: Vec<T>,
            t_to_id: impl Fn(&T) -> ID + 'static,
        ) -> Self {
            Self {
                specific_combobox: widget::combo_box::State::new(options.clone()),
                specific_combobox_selected: if let Some(id) = &filter.id {
                    options.iter().find(|x| &(t_to_id)(x) == id).cloned()
                } else {
                    None
                },
                options,
                timespan_input: date_span_input::State::new(filter.timespan),
                filter,
                t_to_id: Box::new(t_to_id),
            }
        }

        pub fn get_filter(&self) -> &Filter<ID> {
            &self.filter
        }

        pub fn update(&mut self, message: MessageContainer<T>) -> Action {
            let message = message.0;
            match message {
                Message::Delete => return Action::Delete,
                Message::Negate(new_value) => self.filter.negated = new_value,
                Message::Exclude(new_value) => self.filter.include = !new_value,
                Message::Specific(specific) => {
                    if !specific || self.options.is_empty() {
                        self.filter.id = None;
                    } else {
                        self.filter.id = Some((self.t_to_id)(&self.options[0]));
                        self.specific_combobox_selected = Some(self.options[0].clone());
                    }
                }
                Message::CustomTimespan(new_value) => {
                    self.filter.timespan = if new_value { Some((None, None)) } else { None };
                }
                Message::TimespanInput(action) => self.timespan_input.perform(action),
                Message::SpecificSelected(new_specific) => {
                    self.filter.id = Some((self.t_to_id)(&new_specific));
                    self.specific_combobox_selected = Some(new_specific);
                }
            }
            Action::None
        }

        pub fn view(&self) -> iced::Element<MessageContainer<T>> {
            iced::Element::new(
                widget::row![
                    widget::checkbox("Negate", self.filter.negated).on_toggle(Message::Negate),
                    widget::checkbox("Specific", self.filter.id.is_some())
                        .on_toggle(Message::Specific),
                ]
                .push_maybe(if self.filter.id.is_some() {
                    Some(widget::ComboBox::new(
                        &self.specific_combobox,
                        "",
                        self.specific_combobox_selected.as_ref(),
                        Message::SpecificSelected,
                    ))
                } else {
                    None
                })
                .push(widget::checkbox("Exclude", !self.filter.include).on_toggle(Message::Exclude))
                .push(
                    widget::checkbox("Custom Timespan", self.filter.timespan.is_some())
                        .on_toggle(Message::CustomTimespan),
                )
                .push_maybe(if self.filter.timespan.is_some() {
                    Some(
                        date_span_input::date_span_input(&self.timespan_input)
                            .view()
                            .map(Message::TimespanInput),
                    )
                } else {
                    None
                })
                .push(widget::row![
                    widget::horizontal_space(),
                    crate::button::delete(Some(Message::Delete))
                ])
                .align_y(iced::Alignment::Center)
                .spacing(crate::style::ROW_SPACING),
            )
            .map(MessageContainer)
        }
    }
}
