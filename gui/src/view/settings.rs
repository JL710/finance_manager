use crate::finance_managers::FinanceManagers;
use iced::widget;

pub enum Action {
    None,
    ApplySettings(crate::settings::Settings),
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeAPIUrl(String),
    ChangeAPIToken(String),
    #[cfg(feature = "native")]
    ChangeSqlitePath(String),
    #[cfg(feature = "native")]
    StartSQLiteFileSelector,
    #[cfg(feature = "native")]
    StartSQLiteNewFileSelector,
    FmChoice(crate::settings::SelectedFinanceManager),
    Save,
}

#[derive(Debug, Clone)]
pub struct View {
    settings: crate::settings::Settings,
    unsaved: bool,
}

impl View {
    pub fn new(settings: crate::settings::Settings) -> (Self, iced::Task<Message>) {
        (
            Self {
                settings,
                unsaved: false,
            },
            iced::Task::none(),
        )
    }

    pub fn set_unsaved(&mut self) {
        self.unsaved = true;
    }

    pub fn update(
        &mut self,
        message: Message,
        _finance_controller: fm_core::FMController<FinanceManagers>,
    ) -> Action {
        match message {
            Message::ChangeAPIUrl(url) => {
                self.settings.finance_manager.server_url = url;
                self.unsaved = true;
            }
            Message::ChangeAPIToken(token) => {
                self.settings.finance_manager.server_token = token;
                self.unsaved = true;
            }
            #[cfg(feature = "native")]
            Message::ChangeSqlitePath(path) => {
                self.settings.finance_manager.sqlite_path = path;
                self.unsaved = true;
            }
            Message::FmChoice(new_selected) => {
                self.settings.finance_manager.selected_finance_manager = new_selected;
                self.unsaved = true;
            }
            #[cfg(feature = "native")]
            Message::StartSQLiteFileSelector => {
                if let Some(filepath) = rfd::FileDialog::new()
                    .set_title("Select SQLite 3 Database")
                    .pick_file()
                {
                    self.settings.finance_manager.sqlite_path =
                        filepath.to_str().unwrap().to_string();
                    self.unsaved = true;
                }
            }
            #[cfg(feature = "native")]
            Message::StartSQLiteNewFileSelector => {
                if let Some(filepath) = rfd::FileDialog::new()
                    .set_title("New SQLite 3 Database")
                    .save_file()
                {
                    self.settings.finance_manager.sqlite_path =
                        filepath.to_str().unwrap().to_string();
                    self.unsaved = true;
                }
            }
            Message::Save => {
                self.unsaved = false;
                return Action::ApplySettings(self.settings.clone());
            }
        }
        Action::None
    }

    pub fn view(&self) -> iced::Element<Message> {
        let mut col = components::spaced_column![fm_settings_view(
            widget::radio(
                "Server",
                crate::settings::SelectedFinanceManager::Server,
                Some(self.settings.finance_manager.selected_finance_manager),
                Message::FmChoice
            ),
            components::spaced_column![
                components::labeled_entry(
                    "API URL:",
                    &self.settings.finance_manager.server_url,
                    Message::ChangeAPIUrl,
                    false
                ),
                components::labeled_entry(
                    "API Token:",
                    &self.settings.finance_manager.server_token,
                    Message::ChangeAPIToken,
                    false
                ),
            ],
        )];

        #[cfg(feature = "native")]
        {
            let valid_path = valid_sqlite_path(&self.settings.finance_manager.sqlite_path);
            col = col.push(widget::Rule::horizontal(10));
            col = col.push(fm_settings_view(
                widget::radio(
                    "Sqlite",
                    crate::settings::SelectedFinanceManager::SQLite,
                    Some(self.settings.finance_manager.selected_finance_manager),
                    Message::FmChoice,
                ),
                components::spaced_row![
                    "Sqlite Path:",
                    widget::text_input::TextInput::new(
                        "Sqlite Path",
                        &self.settings.finance_manager.sqlite_path
                    )
                    .on_input(Message::ChangeSqlitePath)
                    .style(if valid_path {
                        style::text_input_success
                    } else {
                        style::text_input_danger
                    }),
                    widget::button("Select File").on_press(Message::StartSQLiteFileSelector),
                    widget::button("New").on_press(Message::StartSQLiteNewFileSelector),
                ],
            ));
        }

        col.push(widget::Rule::horizontal(10))
            .push(widget::radio(
                "Ram",
                crate::settings::SelectedFinanceManager::Ram,
                Some(self.settings.finance_manager.selected_finance_manager),
                Message::FmChoice,
            ))
            .push(widget::vertical_space())
            .push(components::button::submit(if self.unsaved {
                Some(Message::Save)
            } else {
                None
            }))
            .into()
    }
}

fn fm_settings_view<'a>(
    radio: impl Into<iced::Element<'a, Message>>,
    settings: impl Into<iced::Element<'a, Message>>,
) -> iced::Element<'a, Message> {
    components::spaced_row![
        radio.into(),
        widget::column![widget::Space::new(0, 30), settings.into()],
    ]
    .into()
}

#[cfg(feature = "native")]
fn valid_sqlite_path(path: &String) -> bool {
    let path = std::path::Path::new(path);
    if path.is_dir() {
        return false;
    }
    if path.is_file() {
        return true;
    }
    if let Some(parent) = path.parent() {
        return parent.is_dir();
    }
    false
}
