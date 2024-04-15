use crate::finance;

use iced::advanced::Application;

mod create_asset_account;
mod show_asset_accounts;
mod view_account;

#[derive(Debug, Clone)]
pub enum AppMessage {
    CreateAssetAccountMessage(create_asset_account::Message),
    AssetAccountOverview,
    TransactionOverview,
    BudgetOverview,
    CreateAssetAccountView,
}

#[derive(Debug, Clone)]
enum CurrentView {
    Empty,
    ViewAccount(view_account::ViewAccount),
    AssetAccountOverview(show_asset_accounts::AssetAccountOverview),
    CreateAssetAccount(create_asset_account::CreateAssetAccountDialog),
}

impl CurrentView {
    fn as_create_asset_account(
        &mut self,
    ) -> Option<&mut create_asset_account::CreateAssetAccountDialog> {
        match self {
            CurrentView::CreateAssetAccount(view) => Some(view),
            _ => None,
        }
    }
}

pub struct App {
    finance_manager: finance::FinanceManager,
    current_view: CurrentView,
}

impl Application for App {
    type Message = AppMessage;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            App {
                finance_manager: finance::FinanceManager::new(),
                current_view: CurrentView::Empty,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Finance Manager")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::CreateAssetAccountMessage(m) => {
                if let Some(account) = self
                    .current_view
                    .as_create_asset_account()
                    .unwrap()
                    .update(m, &mut self.finance_manager)
                {
                    self.current_view = CurrentView::ViewAccount(view_account::ViewAccount::new(
                        &self.finance_manager,
                        account.into(),
                    ))
                }
            }
            AppMessage::AssetAccountOverview => {
                self.current_view = CurrentView::AssetAccountOverview(
                    show_asset_accounts::AssetAccountOverview::new(&self.finance_manager),
                )
            }
            AppMessage::BudgetOverview => self.current_view = CurrentView::Empty,
            AppMessage::TransactionOverview => self.current_view = CurrentView::Empty,
            AppMessage::CreateAssetAccountView => {
                self.current_view = CurrentView::CreateAssetAccount(
                    create_asset_account::CreateAssetAccountDialog::new(),
                )
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        iced::widget::row![
            iced::widget::column![
                iced::widget::button("AssetAccounts")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::AssetAccountOverview),
                iced::widget::button("Transactions")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::TransactionOverview),
                iced::widget::button("Budgets")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::BudgetOverview)
            ]
            .align_items(iced::Alignment::Start)
            .spacing(10)
            .width(iced::Length::FillPortion(2)),
            iced::widget::vertical_rule(5),
            iced::widget::column![match &self.current_view {
                CurrentView::AssetAccountOverview(view) => view.view(),
                CurrentView::CreateAssetAccount(view) =>
                    view.view().map(AppMessage::CreateAssetAccountMessage),
                CurrentView::ViewAccount(view) => view.view(),
                CurrentView::Empty => iced::widget::text("This Page is empty").into(),
            }]
            .width(iced::Length::FillPortion(9))
        ]
        .into()
    }
}
