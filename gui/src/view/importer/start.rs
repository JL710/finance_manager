use std::path::PathBuf;

use iced::widget;

pub enum Action {
    None,
    Task(iced::Task<Message>),
    StartImport(PathBuf),
}

#[derive(Debug, Clone)]
pub enum Message {
    PickFile,
    FilePicked(PathBuf),
    StartImport,
}

#[derive(Debug, Default)]
pub struct Start {
    selected_file: Option<PathBuf>,
}

impl Start {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::PickFile => Action::Task(
                error::failing_task(async {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .set_title("Select File")
                        .pick_file()
                        .await
                    {
                        Ok(Some(file.path().to_path_buf()))
                    } else {
                        Ok(None)
                    }
                })
                .and_then(|x| iced::Task::done(Message::FilePicked(x))),
            ),
            Message::FilePicked(file) => {
                self.selected_file = Some(file);
                Action::None
            }
            Message::StartImport => {
                if let Some(path) = &self.selected_file {
                    Action::StartImport(path.clone())
                } else {
                    Action::None
                }
            }
        }
    }
    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::container(components::spal_column![
            widget::text(
                self.selected_file
                    .as_ref()
                    .map_or(String::new(), |x| x.to_str().unwrap().to_owned())
            ),
            widget::button("Select CAMT-V2 File").on_press(Message::PickFile),
            widget::button("Start").on_press_maybe(if self.selected_file.is_some() {
                Some(Message::StartImport)
            } else {
                None
            })
        ])
        .center(iced::Fill)
        .into()
    }
}
