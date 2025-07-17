use crate::*;
use anyhow::Result;
use bigdecimal::{BigDecimal, FromPrimitive};

pub async fn generate_demo_data<T: FinanceManager>(fc: FMController<T>) -> Result<()> {
    let daily_account = fc
        .create_asset_account(
            "Daily Account".to_string(),
            Some("This is an account for daily expenses and income".to_string()),
            None,
            None,
            Currency::Eur(BigDecimal::from_f32(50.0).unwrap()),
        )
        .await?;
    let savings_account = fc
        .create_asset_account(
            "Savings Account".to_string(),
            Some("This is an account for savings".to_string()),
            None,
            None,
            Currency::Eur(BigDecimal::from_f32(50.0).unwrap()),
        )
        .await?;

    let super_market_account = fc
        .create_book_checking_account("Supermarket".to_string(), None, None, None)
        .await?;
    let book_store_account = fc
        .create_book_checking_account("Bookstore".to_string(), None, None, None)
        .await?;
    let employer_account = fc
        .create_book_checking_account("Employer".to_string(), None, None, None)
        .await?;
    let badminton_club_account = fc
        .create_book_checking_account("Badminton Club".to_string(), None, None, None)
        .await?;
    let landlord_account = fc
        .create_book_checking_account("Landlord".to_string(), None, None, None)
        .await?;
    let friends_account = fc
        .create_book_checking_account("A Friend".to_string(), None, None, None)
        .await?;
    let hardware_store_account = fc
        .create_book_checking_account("Hardware Store".to_string(), None, None, None)
        .await?;

    let food_budget = fc
        .create_budget(
            "Food Budget".to_string(),
            None,
            Currency::Eur(BigDecimal::from_f32(250.0).unwrap()),
            budget::Recurring::DayInMonth(1),
        )
        .await?;

    let need_to_survive_category = fc.create_category("Need it to Survive".to_string()).await?;
    let hobbies_category = fc.create_category("Hobbies".to_string()).await?;
    let fun_category = fc.create_category("Fun".to_string()).await?;
    let income_category = fc.create_category("Income".to_string()).await?;
    let rent_category = fc.create_category("Rent".to_string()).await?;
    let savings_category = fc.create_category("Savings".to_string()).await?;

    let mut income_transactions = Vec::new();
    let mut badminton_club_transactions = Vec::new();
    let mut food_transactions = Vec::new();
    let mut book_transactions = Vec::new();
    let mut rent_transactions = Vec::new();
    let mut savings_transactions = Vec::new();
    for month_i in 0..30 {
        let mut year = time::OffsetDateTime::now_utc().year();
        let mut month = time::OffsetDateTime::now_utc().month() as u8 as i32 - month_i;
        while month <= 0 {
            month += 12;
            year -= 1;
        }
        income_transactions.push(
            fc.create_transaction(
                Currency::Eur(BigDecimal::from_f32(1500.0).unwrap()),
                "Salary".to_string(),
                None,
                employer_account.id,
                daily_account.id,
                None,
                time::OffsetDateTime::new_utc(
                    time::Date::from_calendar_date(year, (month as u8).try_into()?, 1)?,
                    time::Time::from_hms(12, 0, 0)?,
                ),
                HashMap::default(),
                HashMap::from([(income_category.id, Sign::Positive)]),
            )
            .await?,
        );

        badminton_club_transactions.push(
            fc.create_transaction(
                Currency::Eur(BigDecimal::from_f32(80.0).unwrap()),
                "Badminton Club".to_string(),
                None,
                daily_account.id,
                badminton_club_account.id,
                None,
                time::OffsetDateTime::new_utc(
                    time::Date::from_calendar_date(year, (month as u8).try_into()?, 15)?,
                    time::Time::from_hms(12, 0, 0)?,
                ),
                HashMap::default(),
                HashMap::from([(hobbies_category.id, Sign::Negative)]),
            )
            .await?,
        );

        for i in 0..=3 {
            food_transactions.push(
                fc.create_transaction(
                    Currency::Eur(BigDecimal::from_f32(90.0).unwrap()),
                    "Groceries".to_string(),
                    None,
                    daily_account.id,
                    super_market_account.id,
                    Some((food_budget.id, Sign::Negative)),
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(year, (month as u8).try_into()?, 4 + 7 * i)?,
                        time::Time::from_hms(12, 0, 0)?,
                    ),
                    HashMap::default(),
                    HashMap::from([(need_to_survive_category.id, Sign::Negative)]),
                )
                .await?,
            );
        }

        book_transactions.push(
            fc.create_transaction(
                Currency::Eur(BigDecimal::from_f32(12.0).unwrap()),
                "Book".to_string(),
                None,
                daily_account.id,
                book_store_account.id,
                None,
                time::OffsetDateTime::new_utc(
                    time::Date::from_calendar_date(year, (month as u8).try_into()?, 19)?,
                    time::Time::from_hms(12, 0, 0)?,
                ),
                HashMap::default(),
                HashMap::from([(fun_category.id, Sign::Negative)]),
            )
            .await?,
        );

        rent_transactions.push(
            fc.create_transaction(
                Currency::Eur(BigDecimal::from_f32(1000.0).unwrap()),
                "Rent".to_string(),
                None,
                daily_account.id,
                landlord_account.id,
                None,
                time::OffsetDateTime::new_utc(
                    time::Date::from_calendar_date(year, (month as u8).try_into()?, 1)?,
                    time::Time::from_hms(12, 0, 0)?,
                ),
                HashMap::default(),
                HashMap::from([
                    (need_to_survive_category.id, Sign::Negative),
                    (rent_category.id, Sign::Negative),
                ]),
            )
            .await?,
        );

        savings_transactions.push(
            fc.create_transaction(
                Currency::Eur(BigDecimal::from_f32(45.0).unwrap()),
                "Saving".to_string(),
                None,
                daily_account.id,
                savings_account.id,
                None,
                time::OffsetDateTime::new_utc(
                    time::Date::from_calendar_date(year, (month as u8).try_into()?, 1)?,
                    time::Time::from_hms(12, 0, 0)?,
                ),
                HashMap::default(),
                HashMap::from([(savings_category.id, Sign::Positive)]),
            )
            .await?,
        );

        // bought some building material for a friend
        if month_i == 0 {
            let building_material_transaction = fc
                .create_transaction(
                    Currency::Eur(BigDecimal::from_f32(220.0).unwrap()),
                    "Building material for a friend".to_string(),
                    None,
                    daily_account.id,
                    hardware_store_account.id,
                    None,
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(year, (month as u8).try_into()?, 13)?,
                        time::Time::from_hms(12, 0, 0)?,
                    ),
                    HashMap::default(),
                    HashMap::default(),
                )
                .await?;
            fc.create_bill(
                format!("Building material for a friend from {month}/{year}"),
                None,
                Currency::Eur(BigDecimal::from_f32(220.0).unwrap()),
                HashMap::from([(building_material_transaction.id, Sign::Negative)]),
                None,
                false,
            )
            .await?;
        } else if month_i == 1 {
            let building_material_transaction = fc
                .create_transaction(
                    Currency::Eur(BigDecimal::from_f32(250.0).unwrap()),
                    "Building material for a friend".to_string(),
                    None,
                    daily_account.id,
                    hardware_store_account.id,
                    None,
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(year, (month as u8).try_into()?, 13)?,
                        time::Time::from_hms(12, 0, 0)?,
                    ),
                    HashMap::default(),
                    HashMap::default(),
                )
                .await?;
            let dept_repayment_transaction = fc
                .create_transaction(
                    Currency::Eur(BigDecimal::from_f32(250.0).unwrap()),
                    "Money for the building material".to_string(),
                    None,
                    friends_account.id,
                    daily_account.id,
                    None,
                    time::OffsetDateTime::new_utc(
                        time::Date::from_calendar_date(year, (month as u8).try_into()?, 16)?,
                        time::Time::from_hms(12, 0, 0)?,
                    ),
                    HashMap::default(),
                    HashMap::default(),
                )
                .await?;
            fc.create_bill(
                format!("Building material for a friend from {month}/{year}"),
                None,
                Currency::Eur(BigDecimal::from_f32(250.0).unwrap()),
                HashMap::from([
                    (building_material_transaction.id, Sign::Negative),
                    (dept_repayment_transaction.id, Sign::Positive),
                ]),
                None,
                true,
            )
            .await?;
        }
    }

    Ok(())
}
