use super::super::utils;
use super::super::{AppMessage, View};
use fm_core;
use iced::widget;

use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DescriptionInput(String),
    ValueInput(String),
    RecouringPickList(String),
    RecouringFirstInput(String),
    RecouringSecondInput(String),
    Submit,
}

#[derive(Debug, Clone)]
enum Recourung {
    Days(String, String),   // start time and days
    DayInMonth(String),     // i.e. 3. of each month
    Yearly(String, String), // month and day
}

impl Into<fm_core::Recourung> for Recourung {
    fn into(self) -> fm_core::Recourung {
        match self {
            Recourung::Days(start, days) => fm_core::Recourung::Days(
                utils::parse_to_datetime(&start).unwrap(),
                days.parse().unwrap(),
            ),
            Recourung::DayInMonth(day) => fm_core::Recourung::DayInMonth(day.parse().unwrap()),
            Recourung::Yearly(month, day) => {
                fm_core::Recourung::Yearly(month.parse().unwrap(), day.parse().unwrap())
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

#[derive(Debug, Clone)]
pub struct CreateBudgetView {
    name_input: String,
    description_input: String,
    value_input: String,
    recouring_inputs: Recourung,
    recouring_state: Option<String>,
}

impl CreateBudgetView {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            description_input: String::new(),
            value_input: String::new(),
            recouring_inputs: Recourung::Days(String::new(), String::new()),
            recouring_state: None,
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        finance_manager: Arc<Mutex<impl fm_core::FinanceManager + 'static>>,
    ) -> (Option<View>, iced::Command<AppMessage>) {
        match message {
            Message::NameInput(name) => {
                self.name_input = name;
            }
            Message::DescriptionInput(description) => {
                self.description_input = description;
            }
            Message::ValueInput(value) => {
                self.value_input = value;
            }
            Message::Submit => {
                let name_input = self.name_input.clone();
                let description_input = self.description_input.clone();
                let value_input = self.value_input.clone();
                let recouring_inputs = self.recouring_inputs.clone();
                return (
                    Some(View::Empty),
                    iced::Command::perform(
                        async move {
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
                                    fm_core::Currency::Eur(value_input.parse::<f64>().unwrap()),
                                    recouring_inputs.into(),
                                )
                                .await;
                            finance_manager.lock().await.get_budgets().await
                        },
                        |budgets| {
                            AppMessage::SwitchView(View::BudgetOverview(
                                super::budget_overview::BudgetOverview::new(budgets),
                            ))
                        },
                    ),
                );
            }
            Message::RecouringPickList(recouring) => {
                self.recouring_state = Some(recouring.clone());
                match recouring.as_str() {
                    "Days" => {
                        self.recouring_inputs = Recourung::Days(String::new(), String::new());
                    }
                    "Day in month" => {
                        self.recouring_inputs = Recourung::DayInMonth(String::new());
                    }
                    "Yearly" => {
                        self.recouring_inputs = Recourung::Yearly(String::new(), String::new());
                    }
                    _ => {}
                }
            }
            Message::RecouringFirstInput(content) => match &mut self.recouring_inputs {
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
            Message::RecouringSecondInput(content) => match &mut self.recouring_inputs {
                Recourung::Days(_, days) => {
                    *days = content;
                }
                Recourung::DayInMonth(_) => {}
                Recourung::Yearly(_, day) => {
                    *day = content;
                }
            },
        }
        (None, iced::Command::none())
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        widget::column![
            utils::labeled_entry("Name", &self.name_input, Message::NameInput),
            utils::labeled_entry(
                "Description",
                &self.description_input,
                Message::DescriptionInput
            ),
            utils::labeled_entry("Value", &self.value_input, Message::ValueInput),
            self.generate_recouring_view(),
            widget::button::Button::new("Submit").on_press_maybe(if self.submit() {
                Some(Message::Submit)
            } else {
                None
            }),
        ]
        .spacing(10)
        .into()
    }

    fn generate_recouring_view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        let mut row = widget::row![widget::PickList::new(
            vec!["Days", "Day in month", "Yearly"],
            if let Some(state) = &self.recouring_state {
                Some(state.as_str())
            } else {
                None
            },
            |x| Message::RecouringPickList(x.to_string()),
        ),];
        match &self.recouring_inputs {
            Recourung::Days(start, days) => {
                row = row.push(
                    widget::text_input("Start Date day.month.year", &start)
                        .on_input(Message::RecouringFirstInput),
                );
                row = row.push(
                    widget::text_input("Days", &days).on_input(Message::RecouringSecondInput),
                );
            }
            Recourung::DayInMonth(day) => {
                row = row.push(
                    widget::text_input("Day in Month", &day).on_input(Message::RecouringFirstInput),
                );
            }
            Recourung::Yearly(month, day) => {
                row = row.push(
                    widget::text_input("Month", &month).on_input(Message::RecouringFirstInput),
                );
                row = row
                    .push(widget::text_input("Day", &day).on_input(Message::RecouringSecondInput));
            }
        }

        widget::column![widget::Text::new("Recouring"), row,].into()
    }

    fn submit(&self) -> bool {
        if self.name_input.is_empty() {
            return false;
        }
        if self.value_input.parse::<f64>().is_err() {
            return false;
        }
        // check if the recouring inputs are valid
        match &self.recouring_inputs {
            Recourung::Days(start, days) => {
                if chrono::NaiveDate::parse_from_str(&start, "%d.%m.%Y").is_err() {
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
