use anyhow::{Context, Result};
use iced::widget;

pub enum Action {
    SwitchToAssetAccountView,
    SwitchToBookCheckingAccountOverview,
    SwitchToBudgetOverview,
    SwitchToCategoryOverview,
    SwitchToFilterTransactionView,
    SwitchToSettingsView,
    SwitchToLicense,
    SwitchToBillOverview,
    CreateTransaction,
    Task(iced::Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Collapse(bool),
    AssetAccountView,
    BookCheckingAccountOverview,
    BudgetOverview,
    CategoryOverview,
    FilterTransactionView,
    SettingsView,
    License,
    CreateTransaction,
    BillOverview,
}

pub struct Sidebar {
    collapsed: bool,
}

impl Sidebar {
    pub fn new() -> (Self, iced::Task<Message>) {
        (
            Self { collapsed: false },
            error::failing_task(read_collapsed_config()).map(Message::Collapse),
        )
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::Collapse(collapse) => {
                self.collapsed = collapse;
                Action::Task(error::failing_task(write_collapsed_config(self.collapsed)).discard())
            }
            Message::AssetAccountView => Action::SwitchToAssetAccountView,
            Message::BookCheckingAccountOverview => Action::SwitchToBookCheckingAccountOverview,
            Message::BudgetOverview => Action::SwitchToBudgetOverview,
            Message::CategoryOverview => Action::SwitchToCategoryOverview,
            Message::FilterTransactionView => Action::SwitchToFilterTransactionView,
            Message::License => Action::SwitchToLicense,
            Message::SettingsView => Action::SwitchToSettingsView,
            Message::BillOverview => Action::SwitchToBillOverview,
            Message::CreateTransaction => Action::CreateTransaction,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::container(
            components::spaced_column![
                widget::button(
                    widget::Svg::new(icons::LIST.clone())
                        .style(|theme: &iced::Theme, _| widget::svg::Style {
                            color: Some(theme.palette().primary)
                        })
                        .width(iced::Shrink)
                )
                .on_press(Message::Collapse(!self.collapsed))
                .style(widget::button::text),
                icon_menu_item(
                    "AssetAccounts",
                    icons::BANK2.clone(),
                    Message::AssetAccountView,
                    self.collapsed
                ),
                icon_menu_item(
                    "BookCheckingAccounts",
                    icons::CASH.clone(),
                    Message::BookCheckingAccountOverview,
                    self.collapsed
                ),
                icon_menu_item(
                    "Budgets",
                    icons::PIGGY_BANK_FILL.clone(),
                    Message::BudgetOverview,
                    self.collapsed
                ),
                icon_menu_item(
                    "Categories",
                    icons::BOOKMARK_FILL.clone(),
                    Message::CategoryOverview,
                    self.collapsed
                ),
                icon_menu_item(
                    "Bills",
                    icons::FOLDER_FILL.clone(),
                    Message::BillOverview,
                    self.collapsed
                ),
                icon_menu_item(
                    "Transactions",
                    icons::SEND_FILL.clone(),
                    Message::FilterTransactionView,
                    self.collapsed
                ),
                icon_menu_item(
                    "Create Transaction",
                    icons::PLUS_CIRCLE_FILL.clone(),
                    Message::CreateTransaction,
                    self.collapsed
                ),
                widget::vertical_space(),
                icon_menu_item(
                    "Settings",
                    icons::GEAR_FILL.clone(),
                    Message::SettingsView,
                    self.collapsed
                ),
                icon_menu_item(
                    "License",
                    icons::BOOK_FILL.clone(),
                    Message::License,
                    self.collapsed
                ),
            ]
            .align_x(iced::Alignment::Start),
        )
        .style(|theme| {
            let mut color = theme.extended_palette().background.base.color;
            color.r -= 0.05;
            color.g -= 0.05;
            color.b -= 0.05;
            widget::container::background(color)
        })
        .into()
    }
}

fn icon_menu_item<'a, M: Clone + 'a>(
    text: &'a str,
    icon: widget::svg::Handle,
    message: M,
    collapsed: bool,
) -> iced::Element<'a, M> {
    if collapsed {
        widget::tooltip(
            widget::button(
                widget::Svg::new(icon)
                    .width(iced::Shrink)
                    .style(|theme: &iced::Theme, _| widget::svg::Style {
                        color: Some(theme.palette().primary),
                    })
                    .height(25),
            )
            .style(widget::button::text)
            .on_press(message),
            style::container_popup_styling(widget::container(text)),
            widget::tooltip::Position::Right,
        )
        .into()
    } else {
        widget::button(
            components::spal_row![
                widget::Svg::new(icon.clone()).width(iced::Shrink).style(
                    |theme: &iced::Theme, _| widget::svg::Style {
                        color: Some(theme.palette().primary)
                    }
                ),
                text,
            ]
            .height(25),
        )
        .style(style::button_sidebar)
        .on_press(message)
        .into()
    }
}

pub async fn read_collapsed_config() -> Result<bool> {
    if let Some(conf) = crate::config::read_config("sidebar").await? {
        Ok(conf
            .as_object()
            .context("could nof get config as object")?
            .get("collapsed")
            .context("invalid key collapsed")?
            .as_bool()
            .context("expected bool for collapsed")?)
    } else {
        Ok(false)
    }
}

pub async fn write_collapsed_config(collapsed: bool) -> Result<()> {
    crate::config::write_config(
        serde_json::value::Map::from_iter([("collapsed".to_string(), collapsed.into())]).into(),
        "sidebar",
    )
    .await
}
