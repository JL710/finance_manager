mod finance;
mod gui;

use iced::advanced::Application;

fn main() {
    println!(
        "{:?}",
        finance::Currency::Eur(25.0) + finance::Currency::Eur(-25.0)
    );
    gui::App::run(iced::Settings::default()).unwrap();
}
