mod import;
mod start;
use async_std::sync::Mutex;
use iced::widget;
use std::sync::Arc;

pub enum Action<FM: fm_core::FinanceManager + 'static> {
    None,
    Task(iced::Task<Message<FM>>),
}

#[derive(Debug, Clone)]
pub enum Message<FM: fm_core::FinanceManager + 'static> {
    Start(start::Message),
    Import(import::Message),
    Created(
        Arc<
            Mutex<
                fm_importer::Importer<
                    FM,
                    fm_importer::csv_parser::CSVParser<std::collections::VecDeque<u8>>,
                >,
            >,
        >,
    ),
}

#[derive(Debug)]
pub enum Importer<FM: fm_core::FinanceManager + 'static> {
    StartScreen(start::Start),
    ImportScreen(
        Box<import::Import<fm_importer::csv_parser::CSVParser<std::collections::VecDeque<u8>>, FM>>,
    ),
    #[allow(clippy::enum_variant_names)]
    CreatingImporter,
}

impl<FM: fm_core::FinanceManager + 'static> Default for Importer<FM> {
    fn default() -> Self {
        Self::StartScreen(start::Start::default())
    }
}

impl<FM: fm_core::FinanceManager + 'static> Importer<FM> {
    pub fn update(
        &mut self,
        message: Message<FM>,
        finance_controller: fm_core::FMController<FM>,
    ) -> Action<FM> {
        match message {
            Message::Start(m) => {
                if let Self::StartScreen(s) = self {
                    match s.update(m) {
                        start::Action::None => {}
                        start::Action::StartImport(path) => {
                            *self = Self::CreatingImporter;
                            return Action::Task(error::failing_task(async move {
                                let data = async_std::task::spawn_blocking(move || {
                                    fm_importer::csv_parser::csv_camt_v2_data(
                                        path.to_str().unwrap().to_owned(),
                                    )
                                })
                                .await;
                                let importer = fm_importer::Importer::new(
                                    fm_importer::csv_parser::csv_camt_v2_parser(data)?,
                                    finance_controller,
                                )
                                .await?;

                                Ok(Message::Created(Arc::new(Mutex::new(importer))))
                            }));
                        }
                        start::Action::Task(t) => {
                            return Action::Task(t.map(Message::Start));
                        }
                    }
                }
            }
            Message::Import(m) => {
                if let Self::ImportScreen(s) = self {
                    match s.update(m) {
                        import::Action::None => {}
                        import::Action::Task(t) => {
                            return Action::Task(t.map(Message::Import));
                        }
                        import::Action::FinishedImport => {
                            *self = Self::StartScreen(start::Start::default());
                            return Action::Task(
                                iced::Task::future(async {
                                    rfd::AsyncMessageDialog::new()
                                        .set_buttons(rfd::MessageButtons::Ok)
                                        .set_title("Finished")
                                        .set_description("Finished Import Process")
                                        .set_level(rfd::MessageLevel::Info)
                                        .show()
                                        .await;
                                })
                                .discard(),
                            );
                        }
                    }
                }
            }
            Message::Created(imp) => {
                let (new_self, task) = import::Import::new(imp);
                *self = Self::ImportScreen(Box::new(new_self));
                return Action::Task(task.map(Message::Import));
            }
        }
        Action::None
    }
    pub fn view(&self) -> iced::Element<'_, Message<FM>> {
        match self {
            Self::StartScreen(screen) => screen.view().map(Message::Start),
            Self::ImportScreen(screen) => screen.view().map(Message::Import),
            Self::CreatingImporter => widget::container("Creating Importer")
                .center(iced::Fill)
                .into(),
        }
    }
}
