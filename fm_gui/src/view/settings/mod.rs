use crate::finance_managers::FinanceManagers;

use super::super::utils;
use fm_core::FinanceManager;
use fm_server::client::Client;

use iced::widget;

use async_std::sync::Mutex;
use std::sync::Arc;

pub enum Action {
    None,
    Task(iced::Task<Message>),
    NewFinanceManager(Arc<Mutex<fm_core::FMController<FinanceManagers>>>),
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeAPIUrl(String),
    ChangeAPIToken(String),
    ChangeSqlitePath(String),
    SwitchToAPI,
    SwitchToSqlite,
    SwitchToRAM,
    NewFinanceManager(Arc<Mutex<fm_core::FMController<FinanceManagers>>>),
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    current_status: String,
    api_url: String,
    api_token: String,
    sqlite_path: String,
}

impl SettingsView {
    pub fn new(
        finance_manager: Arc<Mutex<fm_core::FMController<FinanceManagers>>>,
    ) -> (Self, iced::Task<Message>) {
        let locked_fm = finance_manager.try_lock().unwrap();
        (
            Self {
                current_status: locked_fm.raw_fm().to_string(),
                api_url: if let FinanceManagers::Server(fm) = (*locked_fm).raw_fm() {
                    fm.url().to_string()
                } else {
                    String::from("http://localhost:3000")
                },
                api_token: String::new(),
                #[cfg(feature = "native")]
                sqlite_path: if let FinanceManagers::Sqlite(fm) = (*locked_fm).raw_fm() {
                    fm.path().to_string()
                } else {
                    String::new()
                },
                #[cfg(not(feature = "native"))]
                sqlite_path: String::new(),
            },
            iced::Task::none(),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_manager: Arc<Mutex<fm_core::FMController<FinanceManagers>>>,
    ) -> Action {
        match message {
            Message::ChangeAPIUrl(url) => {
                self.api_url = url;
            }
            Message::ChangeAPIToken(token) => {
                self.api_token = token;
            }
            Message::ChangeSqlitePath(path) => {
                self.sqlite_path = path;
            }
            Message::SwitchToAPI => {
                let api_url = self.api_url.clone();
                let api_token = self.api_token.clone();
                return Action::Task(iced::Task::future(async move {
                    let written_api_url = api_url.clone();
                    let written_api_token = api_token.clone();
                    async_std::task::spawn_blocking(|| {
                        crate::settings::write_settings(&crate::settings::Settings::new(
                            crate::settings::FinanceManager::API(
                                written_api_url,
                                written_api_token,
                            ),
                        ))
                        .unwrap()
                    })
                    .await;

                    Message::NewFinanceManager(Arc::new(Mutex::new(
                        fm_core::FMController::with_finance_manager(
                            super::super::finance_managers::FinanceManagers::Server(
                                Client::new((api_url, api_token)).unwrap(),
                            ),
                        ),
                    )))
                }));
            }
            #[cfg(feature = "native")]
            Message::SwitchToSqlite => {
                let sqlite_path = self.sqlite_path.clone();
                return Action::Task(iced::Task::future(async move {
                    let written_sqlite_path = sqlite_path.clone();
                    async_std::task::spawn_blocking(|| {
                        crate::settings::write_settings(&crate::settings::Settings::new(
                            crate::settings::FinanceManager::SQLite(written_sqlite_path),
                        ))
                        .unwrap()
                    })
                    .await;

                    Message::NewFinanceManager(Arc::new(Mutex::new(
                            fm_core::FMController::with_finance_manager(
                                super::super::finance_managers::FinanceManagers::Sqlite(
                                    fm_core::managers::sqlite_finange_manager::SqliteFinanceManager::new(sqlite_path)
                                        .unwrap(),
                                )))))
                }));
            }
            #[cfg(not(feature = "native"))]
            Message::SwitchToSqlite => {
                return Action::None;
            }
            Message::SwitchToRAM => {
                return Action::Task(iced::Task::future(async {
                    async_std::task::spawn_blocking(|| {
                        crate::settings::write_settings(&crate::settings::Settings::new(
                            crate::settings::FinanceManager::RAM,
                        ))
                        .unwrap()
                    })
                    .await;
                    Message::NewFinanceManager(Arc::new(Mutex::new(
                        fm_core::FMController::with_finance_manager(
                            super::super::finance_managers::FinanceManagers::Ram(
                                fm_core::managers::ram_finance_manager::RamFinanceManager::default(
                                ),
                            ),
                        ),
                    )))
                }))
            }
            Message::NewFinanceManager(finance_manager) => {
                self.current_status = finance_manager.try_lock().unwrap().raw_fm().to_string();
                return Action::NewFinanceManager(finance_manager);
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        widget::column![
            utils::heading("Setting", utils::HeadingLevel::H1),
            widget::text!("Current Status: {}", self.current_status),
            widget::Rule::horizontal(10),
            widget::row![
                widget::column![
                    widget::row![
                        widget::text("API URL:"),
                        widget::text_input::TextInput::new("API Url", &self.api_url)
                            .on_input(Message::ChangeAPIUrl),
                    ]
                    .spacing(10),
                    widget::row![
                        widget::text("API Token:"),
                        widget::text_input::TextInput::new("API Token", &self.api_token)
                            .on_input(Message::ChangeAPIToken)
                    ]
                    .spacing(10)
                ]
                .spacing(10),
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
