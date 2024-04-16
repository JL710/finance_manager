use super::super::utils;
use super::super::AppMessage;
use super::View;
use crate::finance;
use chrono::TimeZone;
use iced::widget;

#[derive(Debug, Clone)]
pub enum Message {
    NameInput(String),
    DescriptionInput(String),
    ValueInput(String),
    RecouringCombobox(String),
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

fn parse_to_datetime(date: &str) -> anyhow::Result<chrono::DateTime<chrono::Utc>> {
    let date = chrono::NaiveDate::parse_from_str(date, "%d.%m.%Y")?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    Ok(chrono::Utc.from_utc_datetime(&date))
}

impl Into<finance::Recourung> for Recourung {
    fn into(self) -> finance::Recourung {
        match self {
            Recourung::Days(start, days) => {
                finance::Recourung::Days(parse_to_datetime(&start).unwrap(), days.parse().unwrap())
            }
            Recourung::DayInMonth(day) => finance::Recourung::DayInMonth(day.parse().unwrap()),
            Recourung::Yearly(month, day) => {
                finance::Recourung::Yearly(month.parse().unwrap(), day.parse().unwrap())
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

pub struct CreateBudgetView {
    name_input: String,
    description_input: String,
    value_input: String,
    recouring_inputs: Recourung,
    recouruing_state: widget::combo_box::State<String>,
}

impl View for CreateBudgetView {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        _message: Self::ParentMessage,
        _finance_manager: &mut finance::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>> {
        if let AppMessage::CreateBudgetViewMessage(m) = _message {
            return self.update(m, _finance_manager);
        } else {
            panic!();
        }
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        self.view().map(AppMessage::CreateBudgetViewMessage)
    }
}

impl CreateBudgetView {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            description_input: String::new(),
            value_input: String::new(),
            recouring_inputs: Recourung::Days(String::new(), String::new()),
            recouruing_state: widget::combo_box::State::new(vec![
                "Days".to_string(),
                "Day in month".to_string(),
                "Yearly".to_string(),
            ]),
        }
    }

    fn update(
        &mut self,
        message: Message,
        finance_manager: &mut finance::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = AppMessage>>> {
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
                let _budget = finance_manager.create_budget(
                    self.name_input.clone(),
                    if self.description_input.is_empty() {
                        None
                    } else {
                        Some(self.description_input.clone())
                    },
                    finance::Currency::Eur(self.value_input.parse::<f64>().unwrap()),
                    self.recouring_inputs.clone().into(),
                );
                return Some(Box::new(super::budget_overview::BudgetOverview::new(
                    finance_manager,
                )));
            }
            Message::RecouringCombobox(recouring) => match recouring.as_str() {
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
            },
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
        None
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
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
        let mut row = widget::row![widget::ComboBox::new(
            &self.recouruing_state,
            "Recouring",
            Some(&self.recouring_inputs.to_string()),
            Message::RecouringCombobox,
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
