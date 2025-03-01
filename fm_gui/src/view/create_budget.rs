use fm_core;
use iced::widget;

use anyhow::{Context, Result};

use async_std::sync::Mutex;
use recurring_input::recurring_input;
use std::sync::Arc;

pub enum Action {
    None,
    BudgetCreated(fm_core::Id),
    Task(iced::Task<Message>),
    Cancel,
    CancelWithId(fm_core::Id),
}

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DescriptionInput(widget::text_editor::Action),
    ValueInput(String),
    RecurringPickList(String),
    RecurringInput(recurring_input::Action),
    Submit,
    BudgetCreated(fm_core::Id),
    Initialize(Option<fm_core::Budget>),
    Cancel,
}

#[derive(Debug)]
pub struct View {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: widget::text_editor::Content,
    value_input: String,
    recurring_input: recurring_input::State,
    recurring_state: Option<String>,
    submitted: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            id: None,
            name_input: String::new(),
            description_input: widget::text_editor::Content::default(),
            value_input: String::new(),
            recurring_input: recurring_input::State::Days(
                utils::date_time::date_time_input::State::default(),
                String::new(),
            ),
            recurring_state: None,
            submitted: false,
        }
    }
}

impl View {
    pub fn from_budget(budget: fm_core::Budget) -> Result<Self> {
        Ok(Self {
            id: Some(*budget.id()),
            name_input: budget.name().to_string(),
            description_input: widget::text_editor::Content::with_text(
                budget.description().unwrap_or_default(),
            ),
            value_input: budget.total_value().to_num_string(),
            recurring_input: recurring_input::State::from(budget.timespan().clone()),
            recurring_state: match budget.timespan() {
                fm_core::Recurring::Days(_, _) => Some("Days".to_string()),
                fm_core::Recurring::DayInMonth(_) => Some("Day in month".to_string()),
                fm_core::Recurring::Yearly(_, _) => Some("Yearly".to_string()),
            },
            submitted: false,
        })
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            utils::failing_task(async move {
                let budget = finance_manager
                    .lock()
                    .await
                    .get_budget(id)
                    .await?
                    .context(format!("Could not find budget {}", id))?;
                Ok(Message::Initialize(Some(budget)))
            }),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> Action {
        match message {
            Message::Cancel => {
                if let Some(id) = self.id {
                    return Action::CancelWithId(id);
                } else {
                    return Action::Cancel;
                }
            }
            Message::BudgetCreated(id) => return Action::BudgetCreated(id),
            Message::Initialize(budget) => {
                if let Some(budget) = budget {
                    match Self::from_budget(budget) {
                        Ok(new) => *self = new,
                        Err(error) => {
                            return Action::Task(iced::Task::future(async {
                                utils::error_popup(
                                    utils::error_chain_string(
                                        error.context("Error while creating create budget view from existing budget")
                                    )
                                ).await
                            }).discard());
                        }
                    }
                }
            }
            Message::NameInput(name) => {
                self.name_input = name;
            }
            Message::DescriptionInput(action) => {
                self.description_input.perform(action);
            }
            Message::ValueInput(value) => {
                self.value_input = value;
            }
            Message::Submit => {
                self.submitted = true;
                let option_id = self.id;
                let name_input = self.name_input.clone();
                let description_input = self.description_input.text();
                let value_input = self.value_input.clone();
                let recurring_inputs = (&self.recurring_input).try_into();
                return Action::Task(utils::failing_task(async move {
                    let budget = match option_id {
                        Some(id) => {
                            finance_manager
                                .lock()
                                .await
                                .update_budget(
                                    id,
                                    name_input,
                                    if description_input.is_empty() {
                                        None
                                    } else {
                                        Some(description_input)
                                    },
                                    fm_core::Currency::from(value_input.parse::<f64>()?),
                                    recurring_inputs.context(
                                        "Error while converting recurring input into timespan",
                                    )?,
                                )
                                .await?
                        }
                        None => {
                            finance_manager
                                .lock()
                                .await
                                .create_budget(
                                    name_input,
                                    if description_input.is_empty() {
                                        None
                                    } else {
                                        Some(description_input)
                                    },
                                    fm_core::Currency::from(value_input.parse::<f64>()?),
                                    recurring_inputs.context(
                                        "Error while converting recurring input into timespan",
                                    )?,
                                )
                                .await?
                        }
                    };
                    Ok(Message::BudgetCreated(*budget.id()))
                }));
            }
            Message::RecurringPickList(recurring) => {
                self.recurring_state = Some(recurring.clone());
                match recurring.as_str() {
                    "Days" => {
                        self.recurring_input = recurring_input::State::Days(
                            utils::date_time::date_time_input::State::default(),
                            String::new(),
                        );
                    }
                    "Day in month" => {
                        self.recurring_input = recurring_input::State::DayInMonth(String::new());
                    }
                    "Yearly" => {
                        self.recurring_input =
                            recurring_input::State::Yearly(String::new(), String::new());
                    }
                    _ => {}
                }
            }
            Message::RecurringInput(action) => self.recurring_input.perform(action),
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.submitted {
            return "Loading...".into();
        }

        super::view(
            "Create Budget",
            widget::scrollable(utils::spaced_column![
                utils::labeled_entry("Name", &self.name_input, Message::NameInput, true),
                utils::spaced_row![
                    "Description",
                    widget::text_editor(&self.description_input)
                        .on_action(Message::DescriptionInput)
                ],
                utils::labeled_entry("Value", &self.value_input, Message::ValueInput, true),
                self.generate_recurring_view(),
                utils::submit_cancel_row(
                    if self.submittable() {
                        Some(Message::Submit)
                    } else {
                        None
                    },
                    Some(Message::Cancel)
                ),
            ]),
        )
    }

    fn generate_recurring_view(&self) -> iced::Element<'_, Message> {
        let input_correct = TryInto::<fm_core::Recurring>::try_into(&self.recurring_input).is_ok();

        widget::column![
            widget::Text::new("Recurring"),
            widget::container(utils::spal_row![
                widget::text(self.recurring_input.to_string()),
                widget::PickList::new(
                    vec!["Days", "Day in month", "Yearly"],
                    self.recurring_state.as_deref(),
                    |x| Message::RecurringPickList(x.to_string()),
                ),
                recurring_input(&self.recurring_input).map(Message::RecurringInput)
            ])
            .style(move |theme: &iced::Theme| {
                let mut style = widget::container::Style::default();
                style.border.width = 1.0;
                if !input_correct {
                    style.border.color = theme.palette().danger;
                } else {
                    style.border.color = theme.palette().success;
                }
                style
            })
            .padding(3),
        ]
        .into()
    }

    fn submittable(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        if self.value_input.parse::<f64>().is_err() {
            return false;
        }
        // check if the recurring inputs are valid
        if TryInto::<fm_core::Recurring>::try_into(&self.recurring_input).is_err() {
            return false;
        }
        true
    }
}

mod recurring_input {
    use anyhow::Context;
    use iced::widget;
    use utils::date_time::date_time_input;

    #[derive(Debug, Clone)]
    #[allow(clippy::enum_variant_names)]
    pub enum Action {
        DateInput(date_time_input::Action),
        FirstTextInput(String),
        SecondTextInput(String),
    }

    #[derive(Debug)]
    pub enum State {
        /// start time and days
        Days(date_time_input::State, String),
        /// i.e. 3. of each month
        DayInMonth(String),
        /// month and day
        Yearly(String, String),
    }

    impl State {
        pub fn perform(&mut self, action: Action) {
            match self {
                Self::DayInMonth(day) => {
                    if let Action::FirstTextInput(new_day) = action {
                        *day = new_day;
                    }
                }
                Self::Yearly(month, day) => {
                    if let Action::FirstTextInput(new_month) = action {
                        *month = new_month;
                    } else if let Action::SecondTextInput(new_day) = action {
                        *day = new_day;
                    }
                }
                Self::Days(day, days) => {
                    if let Action::DateInput(action) = action {
                        day.perform(action);
                    } else if let Action::SecondTextInput(new_days) = action {
                        *days = new_days;
                    }
                }
            }
        }
    }

    impl std::fmt::Display for State {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                State::Days(start, days) => {
                    write!(
                        f,
                        "Every {} days starting from {}",
                        days,
                        start.datetime().map_or("ERROR".to_owned(), |x| {
                            utils::date_time::to_date_time_string(x)
                        })
                    )
                }
                State::DayInMonth(day) => write!(f, "Every month on the {}th", day),
                State::Yearly(month, day) => write!(f, "Every year on the {}th of {}", day, month),
            }
        }
    }

    impl From<fm_core::Recurring> for State {
        fn from(value: fm_core::Recurring) -> Self {
            match value {
                fm_core::Recurring::DayInMonth(day) => State::DayInMonth(day.to_string()),
                fm_core::Recurring::Days(start, days) => {
                    State::Days(date_time_input::State::new(Some(start)), days.to_string())
                }
                fm_core::Recurring::Yearly(month, day) => {
                    State::Yearly(month.to_string(), day.to_string())
                }
            }
        }
    }

    impl TryFrom<&State> for fm_core::Recurring {
        type Error = anyhow::Error;

        fn try_from(value: &State) -> Result<Self, Self::Error> {
            match value {
                State::Days(start, days) => {
                    let days = days.parse()?;
                    if days > 500 {
                        anyhow::bail!("Days cannot be more than 31");
                    }
                    Ok(fm_core::Recurring::Days(
                        start.datetime().context("Could not parse date time")?,
                        days,
                    ))
                }
                State::DayInMonth(day) => {
                    let day = day.parse()?;
                    if day > 31 {
                        anyhow::bail!("Days cannot be more than 31");
                    }
                    Ok(fm_core::Recurring::DayInMonth(day))
                }
                State::Yearly(month, day) => {
                    let month = month.parse()?;
                    if month > 12 {
                        anyhow::bail!("Month cannot be more than 12");
                    }
                    let day = day.parse()?;
                    if day > 31 {
                        anyhow::bail!("Day cannot be more than 31");
                    }
                    Ok(fm_core::Recurring::Yearly(month, day))
                }
            }
        }
    }

    pub fn recurring_input(state: &State) -> iced::Element<'_, Action> {
        match state {
            State::Days(date, days) => utils::spal_row![
                date_time_input::date_time_input(date, true)
                    .view()
                    .map(Action::DateInput),
                widget::text_input("Days", days).on_input(Action::SecondTextInput)
            ]
            .into(),
            State::DayInMonth(day) => widget::text_input("Day", day)
                .on_input(Action::FirstTextInput)
                .into(),
            State::Yearly(month, day) => utils::spal_row![
                widget::text_input("Month", month).on_input(Action::FirstTextInput),
                widget::text_input("Day", day).on_input(Action::SecondTextInput)
            ]
            .into(),
        }
    }
}
