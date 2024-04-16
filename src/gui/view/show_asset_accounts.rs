use crate::finance;

use super::super::AppMessage;
use super::View;

#[derive(Debug, Clone)]
pub enum Message {
    CreateAssetAccount,
}

#[derive(Debug, Clone)]
pub struct AssetAccountOverview {
    accounts: Vec<finance::account::AssetAccount>,
}

impl View for AssetAccountOverview {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        message: Self::ParentMessage,
        _finance_manager: &mut finance::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>> {
        if let AppMessage::AccountOverviewMessage(m) = message {
            return self.update(m);
        }
        None
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        self.view().map(AppMessage::AccountOverviewMessage)
    }
}

impl AssetAccountOverview {
    pub fn new(finance_manager: &finance::FinanceManager) -> Self {
        let asset_accounts = finance_manager
            .get_accounts()
            .iter()
            .filter_map(|x| match x {
                finance::account::Account::AssetAccount(acc) => Some(acc.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        Self {
            accounts: asset_accounts,
        }
    }

    fn update(&mut self, message: Message) -> Option<Box<dyn View<ParentMessage = AppMessage>>> {
        match message {
            Message::CreateAssetAccount => {
                return Some(Box::new(
                    super::create_asset_account::CreateAssetAccountDialog::new(),
                ))
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        let asset_accounts = self.accounts.clone();

        let account_list = iced::widget::Column::from_vec(
            asset_accounts
                .iter()
                .map(asset_account_overview_entry)
                .collect(),
        )
        .width(iced::Length::Fill);

        iced::widget::column![
            iced::widget::row![
                iced::widget::button("New AssetAccount").on_press(Message::CreateAssetAccount)
            ],
            iced::widget::horizontal_rule(10),
            iced::widget::scrollable(account_list)
        ]
        .into()
    }
}

fn entry_row_container_style(theme: &iced::Theme) -> iced::widget::container::Style {
    match theme {
        iced::Theme::Dark => iced::widget::container::Style::default().with_background(
            iced::Background::Color(iced::Color::from_rgb8(100, 100, 100)),
        ),
        _ => iced::widget::container::Style::default().with_background(iced::Background::Color(
            iced::Color::from_rgb8(100, 100, 100),
        )),
    }
}

fn asset_account_overview_entry(
    account: &finance::account::AssetAccount,
) -> iced::Element<'static, Message, iced::Theme, iced::Renderer> {
    iced::widget::container(iced::widget::text(account.name().to_owned()))
        .style(entry_row_container_style)
        .padding(10)
        .width(iced::Length::Fill)
        .into()
}
