use crate::finance_managers::FinanceManagers;
use components::ValidationTextInput;
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
    TimeZoneInput(String),
    Save,
}

#[derive(Debug)]
pub struct View {
    settings: crate::settings::Settings,
    api_url: ValidationTextInput,
    api_token: ValidationTextInput,
    time_zone_input: ValidationTextInput,
    unsaved: bool,
}

impl View {
    pub fn new(settings: crate::settings::Settings) -> (Self, iced::Task<Message>) {
        (
            Self {
                time_zone_input: ValidationTextInput::new(settings.utc_seconds_offset.to_string())
                    .validation(|content| {
                        if content.is_empty() {
                            Some("empty input".to_string())
                        } else if components::parse_number(content).is_none() {
                            Some("invalid number".to_string())
                        } else {
                            None
                        }
                    }),
                api_token: ValidationTextInput::new(settings.finance_manager.server_token.clone()),
                api_url: ValidationTextInput::new(settings.finance_manager.server_url.clone()),
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
            Message::TimeZoneInput(value) => {
                self.time_zone_input.edit_content(value);
                if let Some(number) = components::parse_number(self.time_zone_input.value()) {
                    self.settings.utc_seconds_offset = number as i32;
                    self.unsaved = true;
                }
            }
        }
        Action::None
    }

    fn savable(&self) -> bool {
        self.time_zone_input.is_valid()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        components::spaced_column![
            widget::scrollable(components::spaced_column![
                components::labeled_frame::LabeledFrame::new(
                    "Finance Manager",
                    self.fm_settings_view(&self.settings)
                )
                .width(iced::Fill),
                components::labeled_frame::LabeledFrame::new(
                    "Timezone",
                    self.time_zone_input.view("", Some(Message::TimeZoneInput)),
                )
                .width(iced::Fill),
            ]),
            widget::vertical_space(),
            components::button::submit(if self.unsaved && self.savable() {
                Some(Message::Save)
            } else {
                None
            })
        ]
        .into()
    }

    fn fm_settings_view(&self, settings: &crate::settings::Settings) -> iced::Element<'_, Message> {
        let mut col = components::spaced_column![fm_radio_helper(
            widget::radio(
                "Server",
                crate::settings::SelectedFinanceManager::Server,
                Some(settings.finance_manager.selected_finance_manager),
                Message::FmChoice
            ),
            components::spaced_column![
                components::labeled_entry(
                    "API URL:",
                    "https://...",
                    &self.api_url,
                    Some(Message::ChangeAPIUrl),
                ),
                components::labeled_entry(
                    "API Token:",
                    "",
                    &self.api_token,
                    Some(Message::ChangeAPIToken)
                ),
            ],
        )];

        #[cfg(feature = "native")]
        {
            let valid_path = valid_sqlite_path(&settings.finance_manager.sqlite_path);
            col = col.push(widget::Rule::horizontal(10));
            col = col.push(fm_radio_helper(
                widget::radio(
                    "Sqlite",
                    crate::settings::SelectedFinanceManager::SQLite,
                    Some(settings.finance_manager.selected_finance_manager),
                    Message::FmChoice,
                ),
                components::spaced_row![
                    "Sqlite Path:",
                    widget::text_input::TextInput::new(
                        "Sqlite Path",
                        &settings.finance_manager.sqlite_path
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
                Some(settings.finance_manager.selected_finance_manager),
                Message::FmChoice,
            ))
            .into()
    }
}

fn fm_radio_helper<'a>(
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
