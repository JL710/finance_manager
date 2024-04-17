use super::super::{utils, AppMessage};
use super::View;
use fm_core;
use iced::widget;

#[derive(Debug, Clone)]
pub enum Message {
    CreateBudget,
}

pub struct BudgetOverview {
    budgets: Vec<fm_core::Budget>,
}

impl View for BudgetOverview {
    type ParentMessage = AppMessage;

    fn update_view(
        &mut self,
        message: Self::ParentMessage,
        finance_manager: &mut fm_core::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = Self::ParentMessage>>> {
        if let AppMessage::BudgetOverViewMessage(m) = message {
            return self.update(m, finance_manager);
        } else {
            panic!();
        }
    }

    fn view_view(&self) -> iced::Element<'_, Self::ParentMessage, iced::Theme, iced::Renderer> {
        self.view().map(AppMessage::BudgetOverViewMessage)
    }
}

impl BudgetOverview {
    pub fn new(finance_manager: &fm_core::FinanceManager) -> Self {
        Self {
            budgets: finance_manager.get_budgets(),
        }
    }

    fn update(
        &mut self,
        message: Message,
        _finance_manager: &mut fm_core::FinanceManager,
    ) -> Option<Box<dyn View<ParentMessage = AppMessage>>> {
        match message {
            Message::CreateBudget => {
                return Some(Box::new(super::create_budget::CreateBudgetView::new()));
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
        widget::column![
            widget::button::Button::new("Create Budget").on_press(Message::CreateBudget),
            widget::horizontal_rule(10),
            generate_budget_list(&self.budgets),
        ]
        .into()
    }
}

fn generate_budget_entry(
    budget: &fm_core::Budget,
) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
    widget::container(widget::text(budget.name()))
        .style(utils::entry_row_container_style)
        .into()
}

fn generate_budget_list(
    budgets: &Vec<fm_core::Budget>,
) -> iced::Element<'_, Message, iced::Theme, iced::Renderer> {
    let mut column = widget::Column::new();

    for budget in budgets {
        column = column.push(generate_budget_entry(budget));
    }

    widget::scrollable(column).into()
}
