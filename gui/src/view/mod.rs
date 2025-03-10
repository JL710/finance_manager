pub mod account;
pub mod asset_accounts_overview;
pub mod bill;
pub mod bill_overview;
pub mod book_checking_account_overview;
pub mod budget;
pub mod budget_overview;
pub mod category;
pub mod category_overview;
pub mod create_asset_account;
pub mod create_bill;
pub mod create_book_checking_account;
pub mod create_budget;
pub mod create_category;
pub mod create_transaction;
pub mod filter_transactions;
pub mod settings;
pub mod transaction;

fn view<'a, Message: 'a>(
    title: &str,
    content: impl Into<iced::Element<'a, Message>>,
) -> iced::Element<'a, Message> {
    iced::widget::column![
        utils::heading(title, utils::HeadingLevel::H1),
        content.into()
    ]
    .into()
}
