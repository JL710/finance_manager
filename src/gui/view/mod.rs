use crate::finance;
use std::fmt::Debug;

pub mod comming_soon_view;
pub mod create_asset_account;
pub mod create_transaction;
pub mod show_asset_accounts;
pub mod view_account;

pub trait View {
    type ParentMessage: Debug + Clone;

    fn update_view(
        &mut self,
        message: Self::ParentMessage,
        finance_manager: &mut finance::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>>;

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer>;
}
