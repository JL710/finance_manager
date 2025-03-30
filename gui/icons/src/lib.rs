use iced::widget::svg::{Handle, Svg};
use std::sync::LazyLock;

pub static ARROW_LEFT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/arrow-left.svg")));

pub fn arrow_left() -> Svg<'static> {
    Svg::new(ARROW_LEFT.clone()).width(iced::Shrink)
}

pub static ARROW_RIGHT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/arrow-right.svg")));

pub fn arrow_right() -> Svg<'static> {
    Svg::new(ARROW_RIGHT.clone()).width(iced::Shrink)
}

pub static PENCIL_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/pencil-fill.svg")));

pub fn pencil_fill() -> Svg<'static> {
    Svg::new(PENCIL_FILL.clone()).width(iced::Shrink)
}

pub static FILTER: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/filter.svg")));

pub fn filter() -> Svg<'static> {
    Svg::new(FILTER.clone()).width(iced::Shrink)
}

pub static FILTER_CIRCLE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/filter-circle.svg")));

pub fn filter_circle() -> Svg<'static> {
    Svg::new(FILTER_CIRCLE.clone()).width(iced::Shrink)
}

pub static FILTER_CIRCLE_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/filter-circle-fill.svg")));

pub fn filter_circle_fill() -> Svg<'static> {
    Svg::new(FILTER_CIRCLE_FILL.clone()).width(iced::Shrink)
}

pub static PLUS_CIRCLE_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/plus-circle-fill.svg")));

pub fn plus_circle_fill() -> Svg<'static> {
    Svg::new(PLUS_CIRCLE_FILL.clone()).width(iced::Shrink)
}

pub static PLUS_SQUARE_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/plus-square-fill.svg")));

pub fn plus_square_fill() -> Svg<'static> {
    Svg::new(PLUS_SQUARE_FILL.clone()).width(iced::Shrink)
}

pub static TRASH_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/trash-fill.svg")));

pub fn trash_fill() -> Svg<'static> {
    Svg::new(TRASH_FILL.clone()).width(iced::Shrink)
}

pub static PENCIL_SQUARE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/pencil-square.svg")));

pub fn pencil_square() -> Svg<'static> {
    Svg::new(PENCIL_SQUARE.clone()).width(iced::Shrink)
}

pub static X_LG: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/x-lg.svg")));

pub fn x_lg() -> Svg<'static> {
    Svg::new(X_LG.clone()).width(iced::Shrink)
}

pub static CHECK_LG: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/check-lg.svg")));

pub fn check_lg() -> Svg<'static> {
    Svg::new(CHECK_LG.clone()).width(iced::Shrink)
}

pub static FULLSCREEN_EXIT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/fullscreen-exit.svg")));

pub fn full_screen() -> Svg<'static> {
    Svg::new(FULLSCREEN_EXIT.clone()).width(iced::Shrink)
}

pub static FULLSCREEN: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/fullscreen.svg")));

pub fn full() -> Svg<'static> {
    Svg::new(FULLSCREEN.clone()).width(iced::Shrink)
}

pub static LAYOUT_SPLIT_HORIZONTAL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/layout-split-horizontal.svg")));

pub fn layout_split_horizontal() -> Svg<'static> {
    Svg::new(LAYOUT_SPLIT_HORIZONTAL.clone()).width(iced::Shrink)
}

pub static LAYOUT_SPLIT_VERTICAL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/layout-split-vertical.svg")));

pub fn layout_split_vertical() -> Svg<'static> {
    Svg::new(LAYOUT_SPLIT_VERTICAL.clone()).width(iced::Shrink)
}

pub static BANK2: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/bank2.svg")));

pub fn bank2() -> Svg<'static> {
    Svg::new(BANK2.clone()).width(iced::Shrink)
}

pub static CASH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/cash.svg")));

pub fn cash() -> Svg<'static> {
    Svg::new(CASH.clone()).width(iced::Shrink)
}

pub static PIGGY_BANK_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/piggy-bank-fill.svg")));

pub fn piggy_bank_fill() -> Svg<'static> {
    Svg::new(PIGGY_BANK_FILL.clone()).width(iced::Shrink)
}

pub static BOOKMARK_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/bookmark-fill.svg")));

pub fn bookmark_fill() -> Svg<'static> {
    Svg::new(BOOKMARK_FILL.clone()).width(iced::Shrink)
}

pub static SEND_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/send-fill.svg")));

pub fn send_fill() -> Svg<'static> {
    Svg::new(SEND_FILL.clone()).width(iced::Shrink)
}

pub static FOLDER_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/folder-fill.svg")));

pub fn folder_fill() -> Svg<'static> {
    Svg::new(FOLDER_FILL.clone()).width(iced::Shrink)
}

pub static GEAR_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/gear-fill.svg")));

pub fn gear_fill() -> Svg<'static> {
    Svg::new(GEAR_FILL.clone()).width(iced::Shrink)
}

pub static BOOK_FILL: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/book-fill.svg")));

pub fn book_fill() -> Svg<'static> {
    Svg::new(BOOK_FILL.clone()).width(iced::Shrink)
}

pub static LIST: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/list.svg")));

pub fn list() -> Svg<'static> {
    Svg::new(LIST.clone()).width(iced::Shrink)
}

pub static FM_LOGO_WINDOW_ICON: LazyLock<iced::window::Icon> = LazyLock::new(|| {
    iced::window::icon::from_file_data(include_bytes!("../assets/FM_Logo.png"), None).unwrap()
});
