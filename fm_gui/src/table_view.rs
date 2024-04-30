use super::utils;
use iced::widget;

pub struct TableView<'a, T, Message, const COLUMNS: usize, TR>
where
    TR: Fn(&T) -> [iced::Element<'a, Message, iced::Theme, iced::Renderer>; COLUMNS] + 'a,
{
    items: Vec<T>,
    headers: Option<[String; COLUMNS]>,
    sortable: [bool; COLUMNS],
    to_row: TR,
    sort_by_callback: Option<Box<dyn Fn(&T, &T, usize, bool) -> std::cmp::Ordering + 'a>>,
    spacing: u16,
    padding: u16,
}

impl<'a, T: 'a, Message: Clone + 'a, const COLUMNS: usize, TR>
    TableView<'a, T, Message, COLUMNS, TR>
where
    TR: Fn(&T) -> [iced::Element<'a, Message, iced::Theme, iced::Renderer>; COLUMNS] + 'a,
{
    pub fn new(items: Vec<T>, to_row: TR) -> Self {
        Self {
            items,
            headers: None,
            sortable: [false; COLUMNS],
            to_row,
            sort_by_callback: None,
            spacing: 10,
            padding: 10,
        }
    }

    pub fn columns_sortable(mut self, sortable: [bool; COLUMNS]) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// set the headers
    pub fn headers(mut self, headers: [String; COLUMNS]) -> Self {
        self.headers = Some(headers);
        self
    }

    /// callback produces based on the items T and the column index and reverse state a Ordering
    pub fn sort_by(
        mut self,
        callback: impl Fn(&T, &T, usize, bool) -> std::cmp::Ordering + 'a,
    ) -> Self {
        self.sort_by_callback = Some(Box::new(callback));
        self.sort(0, false);
        self
    }

    fn sort(&mut self, column: usize, reverse: bool) {
        if let Some(sort_by_callback) = &self.sort_by_callback {
            self.items
                .sort_by(|a, b| sort_by_callback(a, b, column, reverse));
        }
    }

    pub fn into_element(self) -> iced::Element<'a, Message, iced::Theme, iced::Renderer> {
        widget::component(self)
    }
}

#[derive(Default, Debug)]
pub struct TableViewState {
    sort_column: usize,
    reverse: bool,
}

#[derive(Debug, Clone)]
pub enum TableViewMessage<Message> {
    Message(Message),
    SortByColumn(usize),
}

impl<'a, T: 'a, Message, const COLUMNS: usize, TR> widget::Component<Message>
    for TableView<'a, T, Message, COLUMNS, TR>
where
    TR: Fn(&T) -> [iced::Element<'a, Message, iced::Theme, iced::Renderer>; COLUMNS] + 'a,
    Message: 'a + Clone,
{
    type State = TableViewState;
    type Event = TableViewMessage<Message>;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Self::Event::Message(message) => return Some(message),
            Self::Event::SortByColumn(column) => {
                if state.sort_column == column {
                    state.reverse = !state.reverse;
                } else {
                    state.sort_column = column;
                    state.reverse = false;
                }
                self.sort(state.sort_column, state.reverse);
            }
        }
        None
    }

    fn view(
        &self,
        state: &Self::State,
    ) -> iced::Element<'a, Self::Event, iced::Theme, iced::Renderer> {
        let mut data_column = widget::column![].spacing(self.spacing);
        for item in &self.items {
            let row_elements = (self.to_row)(item).map(|x| x.map(|m| TableViewMessage::Message(m)));
            let mut row = widget::row![];
            for element in row_elements {
                row =
                    row.push(widget::container(element).width(iced::Length::FillPortion(1)));
            }
            data_column = data_column.push(
                widget::container(row)
                    .style(utils::entry_row_container_style_weak)
                    .padding(self.padding),
            );
        }

        let mut column = widget::column![].spacing(self.spacing);

        if let Some(headers) = &self.headers {
            let mut row = widget::row![];
            for (index, header) in headers.iter().enumerate() {
                row = row.push(
                    widget::row![widget::text(header.clone()),]
                        .push_maybe(if self.sortable[index] {
                            Some(
                                widget::button("Sort")
                                    .on_press(TableViewMessage::SortByColumn(index)),
                            )
                        } else {
                            None
                        })
                        .width(iced::Length::FillPortion(1)),
                );
            }
            column = column.push(
                widget::container(row)
                    .style(utils::entry_row_container_style_strong)
                    .padding(self.padding),
            );
        }

        column = column.push(widget::scrollable(data_column));

        column.into()
    }
}
