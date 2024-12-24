use iced::widget;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Action<Message> {
    OuterMessage(Message),
    PageChange(usize),
    None,
}

#[derive(Debug, Clone)]
pub enum InnerMessage<Message> {
    OuterMessage(Box<Message>),
    SortByColumn(usize),
    ChangePageBy(isize),
}

#[allow(clippy::type_complexity)]
pub struct State<T, C> {
    items: Vec<T>,
    context: C,
    page_size: usize,
    page: usize,
    sort_column: Option<usize>,
    sort_reverse: bool,
    sortable: HashSet<usize>,
    sort_by_callback: Option<Box<dyn Fn(&T, &T, usize) -> std::cmp::Ordering>>,
}

impl<T, C> std::fmt::Debug for State<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ items length: {:?}, context, page_size: {:?}, page: {:?}, sort_column: {:?}, sort_reverse: {:?}, sortable: {:?} }}",
         self.items.len(), self.page_size, self.page, self.sort_column, self.sort_reverse, self.sortable)
    }
}

impl<T, C> State<T, C> {
    pub fn new(items: Vec<T>, context: C) -> Self {
        Self {
            items,
            context,
            page_size: 50,
            page: 0,
            sort_column: None,
            sort_reverse: false,
            sortable: HashSet::default(),
            sort_by_callback: None,
        }
    }

    pub fn sortable_columns(mut self, sortable: impl Into<HashSet<usize>>) -> Self {
        self.sortable = sortable.into();
        self
    }

    pub fn column_sortable(mut self, column: usize, sortable: bool) -> Self {
        if sortable {
            self.sortable.insert(column);
        }
        self
    }

    /// Callback produces based on the items T and the column index and reverse state a Ordering.
    pub fn sort_by(
        mut self,
        callback: impl Fn(&T, &T, usize) -> std::cmp::Ordering + 'static,
    ) -> Self {
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

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self.page = 0;
        self
    }

    pub fn page(mut self, page: usize) {
        if page > self.max_page() {
            self.page = self.max_page();
            return;
        }
        self.page = page;
    }

    fn max_page(&self) -> usize {
        self.items.len() / self.page_size
    }

    pub fn perform<Message>(&mut self, message: InnerMessage<Message>) -> Action<Message> {
        match message {
            InnerMessage::ChangePageBy(value) => {
                let new_page = (self.page as i32 + value as i32).max(0) as usize;
                if new_page < self.max_page() {
                    self.page = new_page;
                    Action::PageChange(new_page)
                } else {
                    Action::None
                }
            }
            InnerMessage::OuterMessage(outer) => Action::OuterMessage(*outer),
            InnerMessage::SortByColumn(column) => {
                if self.sort_column == Some(column) {
                    self.sort_reverse = !self.sort_reverse;
                } else {
                    self.sort_column = Some(column);
                    self.sort_reverse = false;
                }
                self.sort(column, self.sort_reverse);
                self.page = 0;
                Action::PageChange(0)
            }
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.page = 0;
        self.sort_column = None;
        self.sort_reverse = false;
    }
}

pub type AlignmentFunction<'a, T> =
    dyn Fn(&T, usize, usize) -> (iced::alignment::Horizontal, iced::alignment::Vertical) + 'a;

pub struct TableView<'a, T, C, const COLUMNS: usize> {
    state: &'a State<T, C>,
    headers: Option<[String; COLUMNS]>,
    alignment: Option<Box<AlignmentFunction<'a, T>>>,
    spacing: u16,
    padding: u16,
}

impl<'a, T, C, const COLUMNS: usize> TableView<'a, T, C, COLUMNS> {
    pub fn new(state: &'a State<T, C>) -> Self {
        Self {
            state,
            headers: None,
            alignment: None,
            spacing: 10,
            padding: 10,
        }
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

    pub fn view<Message: Clone + 'a>(
        self,
        to_row: impl Fn(&T, &C) -> [iced::Element<'a, Message>; COLUMNS],
    ) -> iced::Element<'a, InnerMessage<Message>> {
        let mut data_column = widget::column![].spacing(self.spacing);
        for (row_index, item) in self
            .state
            .items
            .iter()
            .enumerate()
            .skip(self.state.page * self.state.page_size)
            .take(self.state.page_size)
        {
            let row_elements: [iced::Element<InnerMessage<Message>>; COLUMNS] =
                (to_row)(item, &self.state.context)
                    .map(|x| x.map(|m| InnerMessage::OuterMessage(Box::new(m))));

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
                        .push_maybe(if self.state.sortable.contains(&index) {
                            Some(
                                widget::button(
                                    widget::svg::Svg::new(widget::svg::Handle::from_memory(
                                        std::borrow::Cow::from(
                                            if Some(index) == self.state.sort_column {
                                                if self.state.sort_reverse {
                                                    &include_bytes!(
                                                        "../../assets/filter-circle-fill.svg"
                                                    )[..]
                                                } else {
                                                    &include_bytes!(
                                                        "../../assets/filter-circle.svg"
                                                    )[..]
                                                }
                                            } else {
                                                &include_bytes!("../../assets/filter.svg")[..]
                                            },
                                        ),
                                    ))
                                    .content_fit(iced::ContentFit::Fill)
                                    .width(iced::Length::Shrink),
                                )
                                .padding(3)
                                .on_press(InnerMessage::SortByColumn(index)),
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
                widget::button("Previous").on_press(InnerMessage::ChangePageBy(-1)),
                widget::text!("Page {}/{}", self.state.page + 1, self.state.max_page() + 1),
                widget::button("Next").on_press(InnerMessage::ChangePageBy(1))
            ]
            .spacing(10),
        );

        column.into()
    }
}

pub fn table_view<T, C, const COLUMNS: usize>(state: &State<T, C>) -> TableView<'_, T, C, COLUMNS> {
    TableView::new(state)
}
