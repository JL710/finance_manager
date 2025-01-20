use crate::finance_managers::FinanceManagers;

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
    #[cfg(feature = "native")]
    ChangeSqlitePath(String),
    #[cfg(feature = "native")]
    StartSQLiteFileSelector,
    SwitchToAPI,
    #[cfg(feature = "native")]
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
            #[cfg(feature = "native")]
            Message::ChangeSqlitePath(path) => {
                self.sqlite_path = path;
            }
            Message::SwitchToAPI => {
                let api_url = self.api_url.clone();
                let api_token = self.api_token.clone();
                return Action::Task(iced::Task::future(async move {
                    let written_api_url = api_url.clone();
                    let written_api_token = api_token.clone();
                    crate::settings::write_settings(crate::settings::Settings::new(
                        crate::settings::FinanceManager::Api(written_api_url, written_api_token),
                    ))
                    .await
                    .unwrap();

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
                    crate::settings::write_settings(crate::settings::Settings::new(
                        crate::settings::FinanceManager::SQLite(written_sqlite_path),
                    ))
                    .await
                    .unwrap();

                    Message::NewFinanceManager(Arc::new(Mutex::new(
                            fm_core::FMController::with_finance_manager(
                                super::super::finance_managers::FinanceManagers::Sqlite(
                                    fm_core::managers::sqlite_finange_manager::SqliteFinanceManager::new(sqlite_path)
                                        .unwrap(),
                                )))))
                }));
            }
            Message::SwitchToRAM => {
                return Action::Task(iced::Task::future(async {
                    crate::settings::write_settings(crate::settings::Settings::new(
                        crate::settings::FinanceManager::Ram,
                    ))
                    .await
                    .unwrap();
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
            #[cfg(feature = "native")]
            Message::StartSQLiteFileSelector => {
                if let Some(filepath) = rfd::FileDialog::new()
                    .set_title("Select SQLite 3 Database")
                    .pick_file()
                {
                    self.sqlite_path = filepath.to_str().unwrap().to_string();
                }
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        let mut col = utils::spaced_column![
            widget::text!("Current Status: {}", self.current_status),
            widget::Rule::horizontal(10),
            utils::spaced_row![
                utils::spaced_column![
                    utils::labeled_entry("API URL:", &self.api_token, Message::ChangeAPIUrl, false),
                    utils::labeled_entry(
                        "API Token:",
                        &self.api_token,
                        Message::ChangeAPIToken,
                        false
                    ),
                ],
                widget::button("Switch").on_press(Message::SwitchToAPI)
            ]
        ];

        #[cfg(feature = "native")]
        {
            col = col.push(utils::spaced_row![
                widget::text("Sqlite Path:"),
                widget::text_input::TextInput::new("Sqlite Path", &self.sqlite_path)
                    .on_input(Message::ChangeSqlitePath),
                widget::button("Select File").on_press(Message::StartSQLiteFileSelector),
                widget::button("Switch").on_press(Message::SwitchToSqlite)
            ]);
        }

        super::view(
            "Settings",
            col.push(widget::Rule::horizontal(10))
                .push(widget::button("Switch to RAM").on_press(Message::SwitchToRAM)),
        )
    }
}
