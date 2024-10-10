use iced::widget;

pub struct TableView<'a, T, C, Message, const COLUMNS: usize, TR>
where
    TR: Fn(&T, &C) -> [iced::Element<'a, Message>; COLUMNS] + 'a,
{
    items: Vec<T>,
    context: C,
    headers: Option<[String; COLUMNS]>,
    sortable: [bool; COLUMNS],
    to_row: TR,
    sort_by_callback: Option<Box<dyn Fn(&T, &T, usize) -> std::cmp::Ordering + 'a>>,
    alignment: Option<
        Box<
            dyn Fn(&T, usize, usize) -> (iced::alignment::Horizontal, iced::alignment::Vertical)
                + 'a,
        >,
    >,
    spacing: u16,
    padding: u16,
    page_size: usize,
    page_count: usize,
    on_page_change: Option<Box<dyn Fn(usize) -> Message + 'a>>,
}

impl<'a, T: 'a, C: 'a, Message: Clone + 'a, const COLUMNS: usize, TR>
    TableView<'a, T, C, Message, COLUMNS, TR>
where
    TR: Fn(&T, &C) -> [iced::Element<'a, Message>; COLUMNS] + 'a,
{
    pub fn new(items: Vec<T>, context: C, to_row: TR) -> Self {
        const PAGE_SIZE: usize = 10;
        let page_count = items.len() / PAGE_SIZE;
        Self {
            page_count: if page_count > 0 { page_count } else { 1 },
            items,
            context,
            headers: None,
            sortable: [false; COLUMNS],
            to_row,
            sort_by_callback: None,
            alignment: None,
            spacing: 10,
            padding: 10,
            page_size: PAGE_SIZE,
            on_page_change: None,
        }
    }

    pub fn on_page_change(mut self, callback: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_page_change = Some(Box::new(callback));
        self
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
    pub fn headers(mut self, headers: [impl Into<String>; COLUMNS]) -> Self {
        self.headers = Some(
            headers
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<String>>()
                .try_into()
                .unwrap(),
        );
        self
    }

    /// Callback produces based on the items T and the column index and reverse state a Ordering.
    pub fn sort_by(mut self, callback: impl Fn(&T, &T, usize) -> std::cmp::Ordering + 'a) -> Self {
        self.sort_by_callback = Some(Box::new(callback));
        self
    }

    fn sort(&mut self, column: usize, reverse: bool) {
        if let Some(sort_by_callback) = &self.sort_by_callback {
            self.items.sort_by(|a, b| {
                let mut ordering = sort_by_callback(a, b, column);
                if reverse {
                    ordering = ordering.reverse();
                }
                ordering
            });
        }
    }

    /// Sets the alignment for the content of cells.
    ///
    /// Params:
    ///     - row item
    ///     - x
    ///     - y
    ///
    /// Returns:
    ///     vertical and horizontal alignment
    pub fn alignment(
        mut self,
        callback: impl Fn(&T, usize, usize) -> (iced::alignment::Horizontal, iced::alignment::Vertical)
            + 'a,
    ) -> Self {
        self.alignment = Some(Box::new(callback));
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        let page_count = self.items.len() / page_size;
        self.page_count = if page_count > 0 { page_count } else { 1 };
        self.page_size = page_size;
        self
    }
}

#[derive(Default, Debug)]
pub struct TableViewState {
    sort_column: usize,
    reverse: bool,
    page: usize,
}

#[derive(Debug, Clone)]
pub enum TableViewMessage<Message> {
    Message(Message),
    SortByColumn(usize),
    ChangePageBy(isize),
}

impl<'a, T: 'a, C: 'a, Message, const COLUMNS: usize, TR> widget::Component<Message>
    for TableView<'a, T, C, Message, COLUMNS, TR>
where
    TR: Fn(&T, &C) -> [iced::Element<'a, Message>; COLUMNS] + 'a,
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
                state.page = 0;
                if let Some(on_page_change) = &self.on_page_change {
                    return Some(on_page_change(0));
                }
            }
            Self::Event::ChangePageBy(page) => {
                let new_page = (state.page as i32 + page as i32).max(0) as usize;
                if new_page < self.page_count {
                    state.page = new_page;
                }
                if let Some(on_page_change) = &self.on_page_change {
                    return Some(on_page_change(state.page));
                }
            }
        }
        None
    }

    fn view(&self, state: &Self::State) -> iced::Element<'a, Self::Event> {
        let mut data_column = widget::column![].spacing(self.spacing);
        for (row_index, item) in self
            .items
            .iter()
            .enumerate()
            .skip(state.page * self.page_size)
            .take(self.page_size)
        {
            let row_elements: [iced::Element<TableViewMessage<Message>>; COLUMNS] =
                (self.to_row)(item, &self.context).map(|x| x.map(|m| TableViewMessage::Message(m)));
            let mut row = widget::row![].spacing(self.spacing);
            for (column_index, element) in row_elements.into_iter().enumerate() {
                let mut cell = widget::container(element).width(iced::Length::FillPortion(1));
                if let Some(alignment) = &self.alignment {
                    let (x_alignment, y_alignment) = (alignment)(item, column_index, row_index);
                    cell = cell.align_x(x_alignment);
                    cell = cell.align_y(y_alignment);
                }
                row = row.push(cell);
            }
            data_column = data_column.push(
                widget::container(row)
                    .style(super::style::container_style_background_weak)
                    .padding(self.padding),
            );
        }

        let mut column = widget::column![].spacing(self.spacing);

        if let Some(headers) = &self.headers {
            let mut row = widget::row![].spacing(self.spacing);
            for (index, header) in headers.iter().enumerate() {
                row = row.push(
                    widget::row![widget::text(header.clone()),]
                        .push_maybe(if self.sortable[index] {
                            Some(
                                widget::button(
                                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                                        std::borrow::Cow::from(if index == state.sort_column {
                                            if state.reverse {
                                                &include_bytes!("../assets/filter-circle-fill.svg")
                                                    [..]
                                            } else {
                                                &include_bytes!("../assets/filter-circle.svg")[..]
                                            }
                                        } else {
                                            &include_bytes!("../assets/filter.svg")[..]
                                        }),
                                    ))
                                    .content_fit(iced::ContentFit::Fill)
                                    .width(iced::Length::Shrink),
                                )
                                .padding(3)
                                .on_press(TableViewMessage::SortByColumn(index)),
                            )
                        } else {
                            None
                        })
                        .spacing(10)
                        .align_y(iced::Alignment::Center)
                        .width(iced::Length::FillPortion(1)),
                );
            }
            column = column.push(
                widget::container(row)
                    .style(super::style::container_style_background_strong)
                    .padding(self.padding),
            );
        }

        column = column.push(widget::scrollable(data_column).height(iced::Fill));
        column = column.push(
            widget::row![
                widget::button("Previous").on_press(TableViewMessage::ChangePageBy(-1)),
                widget::text!("Page {}/{}", state.page + 1, self.page_count),
                widget::button("Next").on_press(TableViewMessage::ChangePageBy(1))
            ]
            .spacing(10),
        );

        column.into()
    }
}

impl<'a, T: 'a, C: 'a, Message: Clone + 'a, const COLUMNS: usize, TR>
    From<TableView<'a, T, C, Message, COLUMNS, TR>> for iced::Element<'a, Message>
where
    TR: Fn(&T, &C) -> [iced::Element<'a, Message>; COLUMNS] + 'a,
{
    fn from(value: TableView<'a, T, C, Message, COLUMNS, TR>) -> Self {
        widget::component(value)
    }
}
