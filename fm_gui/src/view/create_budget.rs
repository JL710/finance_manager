use fm_core;
use iced::widget;

use anyhow::Result;

use async_std::sync::Mutex;
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
    RecurringFirstInput(String),
    RecurringSecondInput(String),
    Submit,
    BudgetCreated(fm_core::Id),
    Initialize(Option<fm_core::Budget>),
    Cancel,
}

#[derive(Debug, Clone)]
enum Recurring {
    Days(String, String),   // start time and days
    DayInMonth(String),     // i.e. 3. of each month
    Yearly(String, String), // month and day
}

impl TryFrom<Recurring> for fm_core::Recurring {
    type Error = anyhow::Error;

    fn try_from(value: Recurring) -> Result<Self> {
        match value {
            Recurring::Days(start, days) => {
                let days = days.parse()?;
                if days > 31 {
                    anyhow::bail!("Days cannot be more than 31");
                }
                Ok(fm_core::Recurring::Days(
                    utils::parse_to_datetime(&start)?,
                    days,
                ))
            }
            Recurring::DayInMonth(day) => Ok(fm_core::Recurring::DayInMonth(day.parse()?)),
            Recurring::Yearly(month, day) => {
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

impl std::fmt::Display for Recurring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Recurring::Days(start, days) => {
                write!(f, "Every {} days starting from {}", days, start)
            }
            Recurring::DayInMonth(day) => write!(f, "Every month on the {}th", day),
            Recurring::Yearly(month, day) => write!(f, "Every year on the {}th of {}", day, month),
        }
    }
}

#[derive(Debug)]
pub struct CreateBudgetView {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: widget::text_editor::Content,
    value_input: String,
    recurring_inputs: Recurring,
    recurring_state: Option<String>,
    submitted: bool,
}

impl Default for CreateBudgetView {
    fn default() -> Self {
        Self {
            id: None,
            name_input: String::new(),
            description_input: widget::text_editor::Content::default(),
            value_input: String::new(),
            recurring_inputs: Recurring::Days(String::new(), String::new()),
            recurring_state: None,
            submitted: false,
        }
    }
}

impl CreateBudgetView {
    pub fn from_budget(budget: fm_core::Budget) -> Self {
        Self {
            id: Some(*budget.id()),
            name_input: budget.name().to_string(),
            description_input: widget::text_editor::Content::with_text(
                budget.description().unwrap_or_default(),
            ),
            value_input: budget.total_value().to_num_string(),
            recurring_inputs: match budget.timespan() {
                fm_core::Recurring::Days(start, days) => Recurring::Days(
                    start
                        .to_offset(fm_core::get_local_timezone().unwrap())
                        .format(&time::format_description::parse("[day].[month].[year]").unwrap())
                        .unwrap(),
                    days.to_string(),
                ),
                fm_core::Recurring::DayInMonth(day) => Recurring::DayInMonth(day.to_string()),
                fm_core::Recurring::Yearly(month, day) => {
                    Recurring::Yearly(month.to_string(), day.to_string())
                }
            },
            recurring_state: match budget.timespan() {
                fm_core::Recurring::Days(_, _) => Some("Days".to_string()),
                fm_core::Recurring::DayInMonth(_) => Some("Day in month".to_string()),
                fm_core::Recurring::Yearly(_, _) => Some("Yearly".to_string()),
            },
            submitted: false,
        }
    }

    pub fn fetch(
        id: fm_core::Id,
        finance_manager: Arc<Mutex<fm_core::FMController<impl fm_core::FinanceManager>>>,
    ) -> (Self, iced::Task<Message>) {
        (
            Self::default(),
            iced::Task::future(async move {
                let budget = finance_manager
                    .lock()
                    .await
                    .get_budget(id)
                    .await
                    .unwrap()
                    .unwrap();
                Message::Initialize(Some(budget))
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
                    *self = Self::from_budget(budget);
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
                let recurring_inputs = self.recurring_inputs.clone();
                return Action::Task(iced::Task::future(async move {
                    let budget = match option_id {
                        Some(id) => finance_manager
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
                                fm_core::Currency::from(value_input.parse::<f64>().unwrap()),
                                recurring_inputs.try_into().unwrap(),
                            )
                            .await
                            .unwrap(),
                        None => finance_manager
                            .lock()
                            .await
                            .create_budget(
                                name_input,
                                if description_input.is_empty() {
                                    None
                                } else {
                                    Some(description_input)
                                },
                                fm_core::Currency::from(value_input.parse::<f64>().unwrap()),
                                recurring_inputs.try_into().unwrap(),
                            )
                            .await
                            .unwrap(),
                    };
                    Message::BudgetCreated(*budget.id())
                }));
            }
            Message::RecurringPickList(recurring) => {
                self.recurring_state = Some(recurring.clone());
                match recurring.as_str() {
                    "Days" => {
                        self.recurring_inputs = Recurring::Days(String::new(), String::new());
                    }
                    "Day in month" => {
                        self.recurring_inputs = Recurring::DayInMonth(String::new());
                    }
                    "Yearly" => {
                        self.recurring_inputs = Recurring::Yearly(String::new(), String::new());
                    }
                    _ => {}
                }
            }
            Message::RecurringFirstInput(content) => match &mut self.recurring_inputs {
                Recurring::Days(start, _) => {
                    *start = content;
                }
                Recurring::DayInMonth(day) => {
                    *day = content;
                }
                Recurring::Yearly(month, _) => {
                    *month = content;
                }
            },
            Message::RecurringSecondInput(content) => match &mut self.recurring_inputs {
                Recurring::Days(_, days) => {
                    *days = content;
                }
                Recurring::DayInMonth(_) => {}
                Recurring::Yearly(_, day) => {
                    *day = content;
                }
            },
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.submitted {
            return "Loading...".into();
        }

        widget::column![
            utils::heading("Create Budget", utils::HeadingLevel::H1),
            utils::labeled_entry("Name", &self.name_input, Message::NameInput, true),
            widget::row![
                "Description",
                widget::text_editor(&self.description_input).on_action(Message::DescriptionInput)
            ]
            .spacing(10),
            utils::labeled_entry("Value", &self.value_input, Message::ValueInput, true),
            self.generate_recurring_view(),
            widget::row![
                widget::button("Cancel")
                    .on_press(Message::Cancel)
                    .style(widget::button::danger),
                widget::horizontal_space(),
                widget::button::Button::new("Submit")
                    .on_press_maybe(if self.submittable() {
                        Some(Message::Submit)
                    } else {
                        None
                    })
                    .style(widget::button::success),
            ],
        ]
        .spacing(10)
        .into()
    }

    fn generate_recurring_view(&self) -> iced::Element<'_, Message> {
        let mut row = widget::row![widget::PickList::new(
            vec!["Days", "Day in month", "Yearly"],
            self.recurring_state.as_deref(),
            |x| Message::RecurringPickList(x.to_string()),
        ),];
        match &self.recurring_inputs {
            Recurring::Days(start, days) => {
                row = row.push(
                    widget::text_input("Start Date day.month.year", start)
                        .on_input(Message::RecurringFirstInput),
                );
                row = row
                    .push(widget::text_input("Days", days).on_input(Message::RecurringSecondInput));
            }
            Recurring::DayInMonth(day) => {
                row = row.push(
                    widget::text_input("Day in Month", day).on_input(Message::RecurringFirstInput),
                );
            }
            Recurring::Yearly(month, day) => {
                row = row.push(
                    widget::text_input("Month", month).on_input(Message::RecurringFirstInput),
                );
                row = row
                    .push(widget::text_input("Day", day).on_input(Message::RecurringSecondInput));
            }
        }

        let input_correct =
            TryInto::<fm_core::Recurring>::try_into(self.recurring_inputs.clone()).is_ok();

        widget::column![
            widget::Text::new("Recurring"),
            widget::container(row)
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
        if TryInto::<fm_core::Recurring>::try_into(self.recurring_inputs.clone()).is_err() {
            return false;
        }
        true
    }
}
