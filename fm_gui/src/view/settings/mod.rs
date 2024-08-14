use crate::finance_managers::FinanceManagers;

use super::super::{utils, AppMessage, View};
use fm_core::FinanceManager;
use fm_server::client::Client;

use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    ChangeAPIUrl(String),
    ChangeSqlitePath(String),
    SwitchToAPI,
    SwitchToSqlite,
    SwitchToRAM,
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    current_status: String,
    api_url: String,
    sqlite_path: String,
}

impl SettingsView {
    pub fn new(finance_manager: Arc<Mutex<fm_core::FMController<FinanceManagers>>>) -> Self {
        let locked_fm = finance_manager.try_lock().unwrap();
        Self {
            current_status: locked_fm.raw_fm().to_string(),
            api_url: if let FinanceManagers::Server(fm) = (*locked_fm).raw_fm() {
                fm.url().to_string()
            } else {
                String::from("http://localhost:3000")
            },
            #[cfg(feature = "native")]
            sqlite_path: if let FinanceManagers::Sqlite(fm) = (*locked_fm).raw_fm() {
                fm.path().to_string()
            } else {
                String::new()
            },
            #[cfg(not(feature = "native"))]
            sqlite_path: String::new(),
        }
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<fm_core::FMController<FinanceManagers>>>,
    ) -> (Option<View>, iced::Task<AppMessage>) {
        match message {
            Message::ChangeAPIUrl(url) => {
                self.api_url = url;
            }
            Message::ChangeSqlitePath(path) => {
                self.sqlite_path = path;
            }
            Message::SwitchToAPI => {
                let api_url = self.api_url.clone();
                return (
                    Some(View::Empty),
                    iced::Task::perform(async move { api_url }, |x| {
                        AppMessage::ChangeFM(Arc::new(Mutex::new(
                            fm_core::FMController::with_finance_manager(
                                super::super::finance_managers::FinanceManagers::Server(
                                    Client::new(x).unwrap(),
                                ),
                            ),
                        )))
                    }),
                );
            }
            #[cfg(feature = "native")]
            Message::SwitchToSqlite => {
                let sqlite_path = self.sqlite_path.clone();
                return (
                    Some(View::Empty),
                    iced::Task::perform(async move { sqlite_path }, |x| {
                        AppMessage::ChangeFM(Arc::new(Mutex::new(
                            fm_core::FMController::with_finance_manager(
                                super::super::finance_managers::FinanceManagers::Sqlite(
                                    fm_core::managers::sqlite_finange_manager::SqliteFinanceManager::new(x)
                                        .unwrap(),
                                ),
                            )
                        )))
                    }),
                );
            }
            #[cfg(not(feature = "native"))]
            Message::SwitchToSqlite => {
                return (None, iced::Task::none());
            }
            Message::SwitchToRAM => {
                return (
                    Some(View::Empty),
                    iced::Task::perform(async {}, |_| {
                        AppMessage::ChangeFM(Arc::new(Mutex::new(
                            fm_core::FMController::with_finance_manager(
                            super::super::finance_managers::FinanceManagers::Ram(
                                fm_core::managers::ram_finance_manager::RamFinanceManager::default(
                                ),
                            ),)
                        )))
                    }),
                )
            }
        }
        (None, iced::Task::none())
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
            utils::heading("Setting", utils::HeadingLevel::H1),
            widget::text!("Current Status: {}", self.current_status),
            widget::Rule::horizontal(10),
            widget::row![
                widget::text("API URL:"),
                widget::text_input::TextInput::new("API Url", &self.api_url)
                    .on_input(Message::ChangeAPIUrl),
                widget::button("Switch").on_press(Message::SwitchToAPI)
            ]
            .spacing(10),
            widget::Rule::horizontal(10),
            widget::row![
                widget::text("Sqlite Path:"),
                widget::text_input::TextInput::new("Sqlite Path", &self.sqlite_path)
                    .on_input(Message::ChangeSqlitePath),
                widget::button("Switch").on_press(Message::SwitchToSqlite)
            ]
            .spacing(10),
            widget::Rule::horizontal(10),
            widget::button("Switch to RAM").on_press(Message::SwitchToRAM),
        ]
        .into()
    }
}
