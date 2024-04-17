use super::super::view::View;
use super::super::AppMessage;

pub struct EmptyView {}

impl View for EmptyView {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        _message: Self::ParentMessage,
        _finance_manager: &mut fm_core::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>> {
        None
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        iced::widget::text("Comming Soon").into()
    }
}
