use iced::widget;

pub enum Action {
    None,
    SwitchToAssetAccountView,
    SwitchToBookCheckingAccountOverview,
    SwitchToBudgetOverview,
    SwitchToCategoryOverview,
    SwitchToFilterTransactionView,
    SwitchToSettingsView,
    SwitchToLicense,
    SwitchToBillOverview,
    CreateTransaction,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleCollapse,
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
    pub fn new(collapsed: bool) -> Self {
        Self { collapsed }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ToggleCollapse => {
                self.collapsed = !self.collapsed;
                Action::None
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

    pub fn view(&self) -> iced::Element<Message> {
        components::spaced_column![
            widget::button(
                widget::Svg::new(icons::LIST.clone())
                    .style(|theme: &iced::Theme, _| widget::svg::Style {
                        color: Some(theme.palette().primary)
                    })
                    .width(iced::Shrink)
            )
            .on_press(Message::ToggleCollapse)
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
        .align_x(iced::Alignment::Start)
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
