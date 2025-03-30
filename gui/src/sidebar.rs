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
    svg_cache: SvgCache,
}

impl Sidebar {
    pub fn new(collapsed: bool) -> Self {
        Self {
            collapsed,
            svg_cache: SvgCache::default(),
        }
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
                widget::Svg::new(self.svg_cache.list.clone())
                    .style(|theme: &iced::Theme, _| widget::svg::Style {
                        color: Some(theme.palette().primary)
                    })
                    .width(iced::Shrink)
            )
            .on_press(Message::ToggleCollapse)
            .style(widget::button::text),
            icon_menu_item(
                "AssetAccounts",
                &self.svg_cache.bank2,
                Message::AssetAccountView,
                self.collapsed
            ),
            icon_menu_item(
                "BookCheckingAccounts",
                &self.svg_cache.cash,
                Message::BookCheckingAccountOverview,
                self.collapsed
            ),
            icon_menu_item(
                "Budgets",
                &self.svg_cache.piggy_bank_fill,
                Message::BudgetOverview,
                self.collapsed
            ),
            icon_menu_item(
                "Categories",
                &self.svg_cache.bookmark_fill,
                Message::CategoryOverview,
                self.collapsed
            ),
            icon_menu_item(
                "Bills",
                &self.svg_cache.folder_fill,
                Message::BillOverview,
                self.collapsed
            ),
            icon_menu_item(
                "Transactions",
                &self.svg_cache.send_fill,
                Message::FilterTransactionView,
                self.collapsed
            ),
            icon_menu_item(
                "Create Transaction",
                &self.svg_cache.plus_circle_fill,
                Message::CreateTransaction,
                self.collapsed
            ),
            widget::vertical_space(),
            icon_menu_item(
                "Settings",
                &self.svg_cache.gear_fill,
                Message::SettingsView,
                self.collapsed
            ),
            icon_menu_item(
                "License",
                &self.svg_cache.book_fill,
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
    icon: &widget::svg::Handle,
    message: M,
    collapsed: bool,
) -> iced::Element<'a, M> {
    if collapsed {
        widget::tooltip(
            widget::button(
                widget::Svg::new(icon.clone())
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

struct SvgCache {
    bank2: widget::svg::Handle,
    cash: widget::svg::Handle,
    piggy_bank_fill: widget::svg::Handle,
    bookmark_fill: widget::svg::Handle,
    send_fill: widget::svg::Handle,
    folder_fill: widget::svg::Handle,
    plus_circle_fill: widget::svg::Handle,
    gear_fill: widget::svg::Handle,
    book_fill: widget::svg::Handle,
    list: widget::svg::Handle,
}

impl Default for SvgCache {
    fn default() -> Self {
        SvgCache {
            bank2: widget::svg::Handle::from_memory(include_bytes!("../assets/bank2.svg")),
            cash: widget::svg::Handle::from_memory(include_bytes!("../assets/cash.svg")),
            piggy_bank_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/piggy-bank-fill.svg"
            )),
            bookmark_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/bookmark-fill.svg"
            )),
            send_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/send-fill.svg")),
            folder_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/folder-fill.svg"
            )),
            plus_circle_fill: widget::svg::Handle::from_memory(include_bytes!(
                "../assets/plus-circle-fill.svg"
            )),
            gear_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/gear-fill.svg")),
            book_fill: widget::svg::Handle::from_memory(include_bytes!("../assets/book-fill.svg")),
            list: widget::svg::Handle::from_memory(include_bytes!("../assets/list.svg")),
        }
    }
}
