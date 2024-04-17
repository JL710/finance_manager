use fm_core;

use iced::advanced::Application;

mod table;
mod utils;
mod view;

#[derive(Debug, Clone)]
pub enum AppMessage {
    CreateAssetAccountMessage(view::create_asset_account::Message),
    AccountOverviewMessage(view::show_asset_accounts::Message),
    AssetAccountOverview,
    TransactionOverview,
    BudgetOverview,
    CreateTransactionView,
    CreateTransactionViewMessage(view::create_transaction::Message),
    BudgetOverViewMessage(view::budget_overview::Message),
    CreateBudgetViewMessage(view::create_budget::Message),
}

pub struct App {
    finance_manager: fm_core::FinanceManager,
    current_view: Box<dyn view::View<ParentMessage = AppMessage>>,
}

impl Application for App {
    type Message = AppMessage;
    type Flags = ();
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Renderer = iced::Renderer;

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let finance_manager = fm_core::FinanceManager::new();
        (
            App {
                current_view: Box::new(view::show_asset_accounts::AssetAccountOverview::new(
                    &finance_manager,
                )),
                finance_manager,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Finance Manager")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            AppMessage::AssetAccountOverview => {
                self.current_view = Box::new(view::show_asset_accounts::AssetAccountOverview::new(
                    &self.finance_manager,
                ))
            }
            AppMessage::BudgetOverview => {
                self.current_view = Box::new(view::budget_overview::BudgetOverview::new(
                    &self.finance_manager,
                ))
            }
            AppMessage::TransactionOverview => {
                self.current_view = Box::new(view::comming_soon_view::EmptyView {})
            }
            AppMessage::CreateTransactionView => {
                self.current_view = Box::new(view::create_transaction::CreateTransactionView::new(
                    &self.finance_manager,
                ))
            }
            _ => {
                if let Some(new_view) = self
                    .current_view
                    .update_view(message, &mut self.finance_manager)
                {
                    self.current_view = new_view;
                }
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
                    .on_press(AppMessage::BudgetOverview),
                iced::widget::button("Create Transaction")
                    .width(iced::Length::Fill)
                    .on_press(AppMessage::CreateTransactionView)
            ]
            .align_items(iced::Alignment::Start)
            .spacing(10)
            .width(iced::Length::FillPortion(2)),
            iced::widget::vertical_rule(5),
            iced::widget::column![self.current_view.view_view()]
                .width(iced::Length::FillPortion(9))
        ]
        .into()
    }
}

pub fn run() {
    App::run(iced::Settings::default()).unwrap();
}