use iced::widget;

pub struct Table<'a, Message> {
    rows: Vec<Vec<iced::Element<'a, Message, iced::Theme, iced::Renderer>>>,
    columns: usize,
}

impl<'a, Message: 'a> Table<'a, Message> {
    pub fn new(columns: usize) -> Self {
        Self {
            rows: Vec::new(),
            columns,
        }
    }

    pub fn push_row(&mut self, row: Vec<iced::Element<'a, Message, iced::Theme, iced::Renderer>>) {
        assert_eq!(self.columns, row.len());
    }

    pub fn convert_to_view(self) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
        let mut parent_column = widget::Column::new();

        for row_vec in self.rows {
            let mut row = widget::Row::new();

            for element in row_vec {
                row = row.push(widget::container(element).width(iced::Length::FillPortion(1)));
            }

            parent_column = parent_column.push(row);
        }

        widget::scrollable(parent_column).into()
    }
}
