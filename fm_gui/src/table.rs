use iced::widget;

use crate::utils;

const ROW_PADDING: u16 = 5;
const ROW_SPACING: u16 = 10;

pub struct Table<'a, Message> {
    rows: Vec<Vec<iced::Element<'a, Message>>>,
    headers: Option<Vec<String>>,
    columns: usize,
}

impl<'a, Message: 'a> Table<'a, Message> {
    pub fn new(columns: usize) -> Self {
        Self {
            rows: Vec::new(),
            headers: None,
            columns,
        }
    }

    pub fn set_headers(mut self, headers: Vec<String>) -> Self {
        assert_eq!(self.columns, headers.len());
        self.headers = Some(headers);
        self
    }

    pub fn push_row(&mut self, row: Vec<iced::Element<'a, Message>>) {
        assert_eq!(self.columns, row.len());
        self.rows.push(row);
    }

    pub fn convert_to_view(self) -> iced::Element<'a, Message> {
        let mut parent_column = widget::Column::new().spacing(10);

        if let Some(headers) = self.headers {
            let mut row = widget::Row::new().spacing(ROW_SPACING).padding(ROW_PADDING);

            for header in headers {
                row = row.push(
                    widget::Text::new(header)
                        .size(20)
                        .width(iced::Length::FillPortion(1)),
                );
            }

            parent_column = parent_column
                .push(widget::container(row).style(utils::container_style_background_strongg));
        }

        for row_vec in self.rows {
            let mut row = widget::Row::new().spacing(ROW_SPACING).padding(ROW_PADDING);

            for element in row_vec {
                row = row.push(widget::container(element).width(iced::Length::FillPortion(1)));
            }

            parent_column = parent_column
                .push(widget::container(row).style(utils::container_style_background_weak));
        }

        widget::scrollable(parent_column)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}
