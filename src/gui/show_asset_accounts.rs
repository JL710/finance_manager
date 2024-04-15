use crate::finance;

use super::AppMessage;

#[derive(Debug, Clone)]
pub struct AssetAccountOverview {
    accounts: Vec<finance::account::AssetAccount>,
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

    pub fn view(&self) -> iced::Element<'_, AppMessage, iced::Theme, iced::Renderer> {
        let asset_accounts = self.accounts.clone();

        let account_list = iced::widget::Column::from_vec(
            asset_accounts
                .iter()
                .map(asset_account_overview_entry)
                .collect(),
        )
        .width(iced::Length::Fill);

        iced::widget::column![
            iced::widget::row![iced::widget::button("New AssetAccount")
                .on_press(AppMessage::CreateAssetAccountView)],
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
) -> iced::Element<'static, AppMessage, iced::Theme, iced::Renderer> {
    iced::widget::container(iced::widget::text(account.name().to_owned()))
        .style(entry_row_container_style)
        .padding(10)
        .width(iced::Length::Fill)
        .into()
}
