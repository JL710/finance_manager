mod config;
mod finance_managers;
mod settings;
mod sidebar;
mod view;

use clap::Parser;
use iced::widget;

use fm_core::FinanceManager;

type Fm = finance_managers::FinanceManagers;
type Fc = fm_core::FMController<Fm>;

#[derive(Debug, Clone)]
enum Message {
    Ignore,
    PaneViewMessage(
        widget::pane_grid::Pane,
        Box<view::Message<finance_managers::FinanceManagers>>,
    ),
    PaneDragged(widget::pane_grid::DragEvent),
    PaneResize(widget::pane_grid::ResizeEvent),
    PaneSplit(widget::pane_grid::Axis, widget::pane_grid::Pane),
    PaneMaximize(widget::pane_grid::Pane),
    PaneClicked(widget::pane_grid::Pane),
    PaneClose(widget::pane_grid::Pane),
    PaneRestore,
    SidebarMessage(sidebar::Message),
    FCModified,
}

pub struct App {
    finance_controller: fm_core::FMController<finance_managers::FinanceManagers>,
    finance_controller_switched: fm_core::DateTime,
    pane_grid: widget::pane_grid::State<view::View<Fm>>,
    focused_pane: widget::pane_grid::Pane,
    side_bar: sidebar::Sidebar,
    settings: settings::Settings,
}

impl App {
    fn new(finance_controller: Fc, settings: settings::Settings) -> (Self, iced::Task<Message>) {
        let (sidebar_state, sidebar_task) = sidebar::Sidebar::new();
        let (pane_grid, focused_pane) = widget::pane_grid::State::new(view::View::Markdown(
            "Finance Manager".to_string(),
            widget::markdown::parse(include_str!("view/tutorial.md")).collect(),
        ));
        (
            App {
                finance_controller,
                finance_controller_switched: time::OffsetDateTime::now_utc(),
                settings,
                side_bar: sidebar_state,
                pane_grid,
                focused_pane,
            },
            sidebar_task.map(Message::SidebarMessage),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Ignore => {}
            Message::FCModified => {
                let mut tasks = Vec::new();
                for (pane, view) in self.pane_grid.panes.iter_mut() {
                    let pane = *pane;
                    tasks.push(
                        view.reload_from_fc(
                            self.finance_controller.clone(),
                            time::UtcOffset::from_whole_seconds(self.settings.utc_seconds_offset)
                                .unwrap(),
                        )
                        .map(move |x| Message::PaneViewMessage(pane, Box::new(x))),
                    );
                }
                return iced::Task::batch(tasks);
            }
            Message::SidebarMessage(m) => match self.side_bar.update(m) {
                sidebar::Action::Task(task) => return task.map(Message::SidebarMessage),
                #[cfg(feature = "native")]
                sidebar::Action::SwitchToImporter => {
                    *self.pane_grid.get_mut(self.focused_pane).unwrap() =
                        view::View::Importer(view::importer::Importer::default())
                }
                sidebar::Action::SwitchToBudgetOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .budget_overview(
                            self.finance_controller.clone(),
                            time::UtcOffset::from_whole_seconds(self.settings.utc_seconds_offset)
                                .unwrap(),
                        )
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::CreateTransaction => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .transaction_create(self.finance_controller.clone(), None)
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToAssetAccountView => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .asset_account_overview(self.finance_controller.clone())
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToCategoryOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .category_overview(self.finance_controller.clone())
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToBookCheckingAccountOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .book_checking_account_overview(self.finance_controller.clone())
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToSettingsView => {
                    let (view, task) = view::settings::View::new(self.settings.clone());
                    *self.pane_grid.get_mut(self.focused_pane).unwrap() =
                        view::View::Settings(view);
                    let pane = self.focused_pane;
                    return task
                        .map(view::Message::Settings)
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToFilterTransactionView => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .transaction_filter(self.finance_controller.clone())
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToBillOverview => {
                    let pane = self.focused_pane;
                    return self
                        .pane_grid
                        .get_mut(self.focused_pane)
                        .unwrap()
                        .bill_overview(self.finance_controller.clone())
                        .map(move |x| Message::PaneViewMessage(pane, x.into()));
                }
                sidebar::Action::SwitchToLicense => {
                    *self.pane_grid.get_mut(self.focused_pane).unwrap() = view::View::License;
                }
            },
            Message::PaneClose(pane) => {
                if self.pane_grid.panes.len() > 1 {
                    self.pane_grid.close(pane);
                    if self.focused_pane == pane {
                        self.focused_pane = *self.pane_grid.panes.keys().next().unwrap();
                    }
                }
            }
            Message::PaneDragged(event) => {
                if let widget::pane_grid::DragEvent::Dropped { pane, target } = event {
                    self.pane_grid.drop(pane, target);
                }
                self.focused_pane = *self.pane_grid.panes.keys().next().unwrap();
            }
            Message::PaneResize(event) => {
                self.pane_grid.resize(event.split, event.ratio);
            }
            Message::PaneSplit(axis, pane) => {
                self.pane_grid.split(
                    axis,
                    pane,
                    view::View::Markdown(
                        "Finance Manager".to_string(),
                        widget::markdown::parse(include_str!("view/tutorial.md")).collect(),
                    ),
                );
            }
            Message::PaneMaximize(pane) => {
                self.pane_grid.maximize(pane);
            }
            Message::PaneRestore => {
                self.pane_grid.restore();
            }
            Message::PaneClicked(pane) => {
                self.focused_pane = pane;
            }
            Message::PaneViewMessage(pane, view_message) => match self.pane_grid.get_mut(pane) {
                Some(current_view) => {
                    match view::view_update(
                        self.finance_controller.clone(),
                        time::UtcOffset::from_whole_seconds(self.settings.utc_seconds_offset)
                            .unwrap(),
                        current_view,
                        *view_message,
                    ) {
                        view::Action::Task(task) => {
                            return task.map(move |m| Message::PaneViewMessage(pane, m.into()));
                        }
                        view::Action::ApplySettings(new_settings) => {
                            return self.apply_settings(new_settings, Some(pane));
                        }
                        view::Action::None => return iced::Task::none(),
                    }
                }
                None => return iced::Task::none(),
            },
        }
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        static PANE_BORDER_RADIUS: u16 = 5;

        iced::widget::row![
            self.side_bar.view().map(Message::SidebarMessage),
            iced::widget::vertical_rule(5),
            iced::widget::container(
                widget::pane_grid::PaneGrid::new(
                    &self.pane_grid,
                    |pane, current_view, maximized| {
                        widget::pane_grid::Content::new(
                            widget::container(
                                iced::Element::from(current_view)
                                    .map(move |m| Message::PaneViewMessage(pane, m.into())),
                            )
                            .width(iced::Fill)
                            .height(iced::Fill)
                            .padding(style::PADDING)
                            .style(move |theme: &iced::Theme| {
                                let mut style =
                                    widget::container::background(theme.palette().background);
                                style.border.radius =
                                    style.border.radius.bottom(PANE_BORDER_RADIUS);
                                style.border.width = 5.0;
                                style.border.color = if pane == self.focused_pane {
                                    theme.extended_palette().primary.weak.color
                                } else {
                                    theme.extended_palette().secondary.strong.color
                                };
                                style
                            }),
                        )
                        .title_bar(
                            widget::pane_grid::TitleBar::new(widget::text(
                                current_view.to_string(),
                            ))
                            .controls(iced::Element::new(components::spaced_row![
                                pane_grid_control_buttons(icons::LAYOUT_SPLIT_HORIZONTAL.clone())
                                    .on_press(Message::PaneSplit(
                                        widget::pane_grid::Axis::Vertical,
                                        pane
                                    )),
                                pane_grid_control_buttons(icons::LAYOUT_SPLIT_VERTICAL.clone())
                                    .on_press(Message::PaneSplit(
                                        widget::pane_grid::Axis::Horizontal,
                                        pane
                                    )),
                                pane_grid_control_buttons(if maximized {
                                    icons::FULLSCREEN_EXIT.clone()
                                } else {
                                    icons::FULLSCREEN.clone()
                                })
                                .on_press(if maximized {
                                    Message::PaneRestore
                                } else {
                                    Message::PaneMaximize(pane)
                                }),
                                pane_grid_control_buttons(icons::X_LG.clone()).on_press_maybe(
                                    if self.pane_grid.panes.len() <= 1 {
                                        None
                                    } else {
                                        Some(Message::PaneClose(pane))
                                    }
                                ),
                            ]))
                            .style(move |theme: &iced::Theme| {
                                let mut style =
                                    widget::container::background(if pane == self.focused_pane {
                                        theme.extended_palette().primary.weak.color
                                    } else {
                                        theme.extended_palette().secondary.strong.color
                                    });
                                style.border.radius = style.border.radius.top(PANE_BORDER_RADIUS);
                                style
                            })
                            .padding(5.0),
                        )
                    }
                )
                .spacing(style::SPACING)
                .on_drag(Message::PaneDragged)
                .on_resize(10, Message::PaneResize)
                .on_click(Message::PaneClicked)
            )
            .width(iced::Fill)
            .padding(style::PADDING)
        ]
        .into()
    }

    fn apply_settings(
        &mut self,
        new_settings: settings::Settings,
        pane: Option<widget::pane_grid::Pane>,
    ) -> iced::Task<Message> {
        let mut valid_settings = true;
        match new_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                if !matches!(
                    *self.finance_controller.raw_fm().try_lock().unwrap(),
                    finance_managers::FinanceManagers::Ram(_)
                ) {
                    self.finance_controller = fm_core::FMController::with_finance_manager(
                        finance_managers::FinanceManagers::Ram(
                            fm_core::managers::RamFinanceManager::default(),
                        ),
                    );
                    self.finance_controller_switched = time::OffsetDateTime::now_utc();
                }
            }
            #[cfg(feature = "native")]
            settings::SelectedFinanceManager::SQLite => {
                let fm = match fm_core::managers::SqliteFinanceManager::new(
                    new_settings.finance_manager.sqlite_path.clone(),
                ) {
                    Ok(x) => Some(x),
                    Err(_) => {
                        if let Some(pane) = pane
                            && let view::View::Settings(settings_view) =
                                self.pane_grid.get_mut(pane).unwrap()
                        {
                            settings_view.set_unsaved();
                        }
                        rfd::MessageDialog::new()
                            .set_title("Invalid SQLite Path")
                            .set_description("The provided SQLite path is invalid.")
                            .show();
                        valid_settings = false;
                        None
                    }
                };
                if let Some(manager) = fm {
                    self.finance_controller = fm_core::FMController::with_finance_manager(
                        finance_managers::FinanceManagers::Sqlite(manager),
                    );
                    self.finance_controller_switched = time::OffsetDateTime::now_utc();
                }
            }
            #[cfg(not(feature = "native"))]
            settings::SelectedFinanceManager::SQLite => {}
            settings::SelectedFinanceManager::Server => {
                self.finance_controller = fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Server(
                        fm_server::client::Client::new((
                            new_settings.finance_manager.server_url.clone(),
                            new_settings.finance_manager.server_token.clone(),
                        ))
                        .unwrap(),
                    ),
                );
                self.finance_controller_switched = time::OffsetDateTime::now_utc();
            }
        }
        if valid_settings {
            self.settings = new_settings.clone();
            let future = settings::write_settings(new_settings);
            iced::Task::future(async move {
                future.await.unwrap();
                Message::Ignore
            })
        } else {
            iced::Task::none()
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        let controller = self.finance_controller.clone();
        iced::Subscription::run_with_id(
            self.finance_controller_switched,
            iced::stream::channel(100, |mut channel| async move {
                let mut last_modified = controller.last_modified().await.unwrap();
                use iced::futures::SinkExt;
                loop {
                    let new_last_modified = controller.last_modified().await.unwrap();
                    if new_last_modified != last_modified {
                        channel.send(Message::FCModified).await.unwrap();
                        last_modified = new_last_modified;
                    }
                    async_std::task::yield_now().await;
                }
            }),
        )
    }
}

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    /// Verbose mode
    #[clap(short, long, default_value = "false")]
    verbose: bool,
    /// Debug mode
    #[clap(short, long, default_value = "false")]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // tracing / logging
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
    if args.verbose || args.debug {
        let stdout_log = tracing_subscriber::fmt::layer().compact();
        tracing_subscriber::registry()
            .with(stdout_log.with_filter(
                tracing_subscriber::filter::Targets::default().with_target(
                    "fm_gui",
                    if args.debug {
                        tracing::Level::DEBUG
                    } else {
                        tracing::Level::INFO
                    },
                ),
            ))
            .init();
    }

    let loaded_settings = match async_std::task::block_on(settings::read_settings()) {
        Ok(loaded_setting) => loaded_setting,
        Err(err) => {
            let error_message = error::error_chain_string(err);
            error::blocking_error_popup(error_message.clone());
            panic!("{}", error_message);
        }
    };

    let (app, initial_task) = App::new(
        match loaded_settings.finance_manager.selected_finance_manager {
            settings::SelectedFinanceManager::Ram => {
                fm_core::FMController::with_finance_manager(finance_managers::FinanceManagers::Ram(
                    fm_core::managers::RamFinanceManager::new(()).unwrap(),
                ))
            }
            settings::SelectedFinanceManager::SQLite => {
                #[cfg(not(feature = "native"))]
                panic!("SQLite is not supported in the wasm version");
                #[cfg(feature = "native")]
                fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Sqlite(
                        match fm_core::managers::SqliteFinanceManager::new(
                            loaded_settings.finance_manager.sqlite_path.clone(),
                        ) {
                            Ok(fm) => fm,
                            Err(error) => {
                                rfd::MessageDialog::new()
                                    .set_title("Invalid SQLite Path")
                                    .set_description(error::error_chain_string(error))
                                    .show();
                                panic!("Invalid SQLite Path")
                            }
                        },
                    ),
                )
            }
            settings::SelectedFinanceManager::Server => {
                fm_core::FMController::with_finance_manager(
                    finance_managers::FinanceManagers::Server(
                        fm_server::client::Client::new((
                            loaded_settings.finance_manager.server_url.clone(),
                            loaded_settings.finance_manager.server_token.clone(),
                        ))
                        .unwrap(),
                    ),
                )
            }
        },
        loaded_settings,
    );

    // run the gui
    iced::application("Finance Manager", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| iced::Theme::Nord)
        .window(iced::window::Settings {
            icon: Some(icons::FM_LOGO_WINDOW_ICON.clone()),
            ..Default::default()
        })
        .font(include_bytes!("../fonts/FiraSans-Regular.ttf"))
        .font(include_bytes!("../fonts/FiraSans-Bold.ttf"))
        .default_font(iced::Font::with_name("Fira Sans"))
        .run_with(|| (app, initial_task))
        .unwrap();
}

fn pane_grid_control_buttons(svg: widget::svg::Handle) -> widget::Button<'static, Message> {
    widget::button(
        widget::svg(svg)
            .style(|theme: &iced::Theme, _| widget::svg::Style {
                color: Some(theme.palette().text),
            })
            .width(13.0)
            .height(13.0),
    )
    .style(widget::button::secondary)
}
