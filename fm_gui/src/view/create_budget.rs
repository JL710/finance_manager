use super::super::utils;
use fm_core;
use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    BudgetCreated(fm_core::Id),
    Task(iced::Task<Message>),
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
}

#[derive(Debug, Clone)]
enum Recourung {
    Days(String, String),   // start time and days
    DayInMonth(String),     // i.e. 3. of each month
    Yearly(String, String), // month and day
}

impl From<Recourung> for fm_core::Recurring {
    fn from(value: Recourung) -> Self {
        match value {
            Recourung::Days(start, days) => fm_core::Recurring::Days(
                utils::parse_to_datetime(&start).unwrap(),
                days.parse().unwrap(),
            ),
            Recourung::DayInMonth(day) => fm_core::Recurring::DayInMonth(day.parse().unwrap()),
            Recourung::Yearly(month, day) => {
                fm_core::Recurring::Yearly(month.parse().unwrap(), day.parse().unwrap())
            }
        }
    }
}

impl std::fmt::Display for Recourung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Recourung::Days(start, days) => {
                write!(f, "Every {} days starting from {}", days, start)
            }
            Recourung::DayInMonth(day) => write!(f, "Every month on the {}th", day),
            Recourung::Yearly(month, day) => write!(f, "Every year on the {}th of {}", day, month),
        }
    }
}

#[derive(Debug)]
pub struct CreateBudgetView {
    id: Option<fm_core::Id>,
    name_input: String,
    description_input: widget::text_editor::Content,
    value_input: String,
    recurring_inputs: Recourung,
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
            recurring_inputs: Recourung::Days(String::new(), String::new()),
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
                fm_core::Recurring::Days(start, days) => {
                    Recourung::Days(start.format("%d.%m.%Y").to_string(), days.to_string())
                }
                fm_core::Recurring::DayInMonth(day) => Recourung::DayInMonth(day.to_string()),
                fm_core::Recurring::Yearly(month, day) => {
                    Recourung::Yearly(month.to_string(), day.to_string())
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
                                recurring_inputs.into(),
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
                                recurring_inputs.into(),
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
                        self.recurring_inputs = Recourung::Days(String::new(), String::new());
                    }
                    "Day in month" => {
                        self.recurring_inputs = Recourung::DayInMonth(String::new());
                    }
                    "Yearly" => {
                        self.recurring_inputs = Recourung::Yearly(String::new(), String::new());
                    }
                    _ => {}
                }
            }
            Message::RecurringFirstInput(content) => match &mut self.recurring_inputs {
                Recourung::Days(start, _) => {
                    *start = content;
                }
                Recourung::DayInMonth(day) => {
                    *day = content;
                }
                Recourung::Yearly(month, _) => {
                    *month = content;
                }
            },
            Message::RecurringSecondInput(content) => match &mut self.recurring_inputs {
                Recourung::Days(_, days) => {
                    *days = content;
                }
                Recourung::DayInMonth(_) => {}
                Recourung::Yearly(_, day) => {
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
            utils::labeled_entry("Name", &self.name_input, Message::NameInput),
            widget::row![
                "Description",
                widget::text_editor(&self.description_input).on_action(Message::DescriptionInput)
            ]
            .spacing(10),
            utils::labeled_entry("Value", &self.value_input, Message::ValueInput),
            self.generate_recurring_view(),
            widget::button::Button::new("Submit").on_press_maybe(if self.submittable() {
                Some(Message::Submit)
            } else {
                None
            }),
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
            Recourung::Days(start, days) => {
                row = row.push(
                    widget::text_input("Start Date day.month.year", start)
                        .on_input(Message::RecurringFirstInput),
                );
                row = row
                    .push(widget::text_input("Days", days).on_input(Message::RecurringSecondInput));
            }
            Recourung::DayInMonth(day) => {
                row = row.push(
                    widget::text_input("Day in Month", day).on_input(Message::RecurringFirstInput),
                );
            }
            Recourung::Yearly(month, day) => {
                row = row.push(
                    widget::text_input("Month", month).on_input(Message::RecurringFirstInput),
                );
                row = row
                    .push(widget::text_input("Day", day).on_input(Message::RecurringSecondInput));
            }
        }

        widget::column![widget::Text::new("Recurring"), row,].into()
    }

    fn submittable(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        if self.value_input.parse::<f64>().is_err() {
            return false;
        }
        // check if the recurring inputs are valid
        match &self.recurring_inputs {
            Recourung::Days(start, days) => {
                if chrono::NaiveDate::parse_from_str(start, "%d.%m.%Y").is_err() {
                    return false;
                }
                if days.parse::<usize>().is_err() {
                    return false;
                }
            }
            Recourung::DayInMonth(day) => {
                if let Ok(num) = day.parse::<u16>() {
                    if num > 31 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            Recourung::Yearly(month, day) => {
                // check if month is valid
                if let Ok(num) = month.parse::<u8>() {
                    if num > 12 {
                        return false;
                    }
                } else {
                    return false;
                }
                // check if day is valid
                if let Ok(num) = day.parse::<u16>() {
                    if num > 31 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }
}
