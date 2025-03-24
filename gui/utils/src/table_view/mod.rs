use iced::widget;
use std::collections::HashSet;

mod inner_table_view;

pub enum Action<Message> {
    OuterMessage(Message),
    Task(iced::Task<InnerMessage<Message>>),
    None,
}

#[derive(Debug, Clone)]
pub enum InnerMessage<Message> {
    OuterMessage(Box<Message>),
    SortByColumn(usize),
    ChangePageBy(isize),
    ScrollToTop,
}

#[allow(clippy::type_complexity)]
pub struct State<T, C> {
    items: Vec<T>,
    inner_layout_id: isize,
    context: C,
    page_size: usize,
    page: usize,
    sort_column: Option<usize>,
    sort_reverse: bool,
    sortable: HashSet<usize>,
    sort_by_callback: Option<Box<dyn Fn(&T, &T, usize) -> std::cmp::Ordering>>,
    scrollable_id: widget::scrollable::Id,
}

impl<T, C> std::fmt::Debug for State<T, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ items length: {:?}, context, page_size: {:?}, page: {:?}, sort_column: {:?}, sort_reverse: {:?}, sortable: {:?} }}",
            self.items.len(),
            self.page_size,
            self.page,
            self.sort_column,
            self.sort_reverse,
            self.sortable
        )
    }
}

impl<T, C> State<T, C> {
    pub fn new(items: Vec<T>, context: C) -> Self {
        Self {
            items,
            inner_layout_id: uuid::Uuid::new_v4().as_u128() as isize,
            context,
            page_size: 50,
            page: 0,
            sort_column: None,
            sort_reverse: false,
            sortable: HashSet::default(),
            sort_by_callback: None,
            scrollable_id: widget::scrollable::Id::unique(),
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

    pub fn sort(&mut self, column: usize, reverse: bool) {
        if let Some(sort_by_callback) = &self.sort_by_callback {
            self.items.sort_by(|a, b| {
                let mut ordering = sort_by_callback(a, b, column);
                if reverse {
                    ordering = ordering.reverse();
                }
                ordering
            });
            self.sort_column = Some(column);
            self.sort_reverse = reverse;
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
                if new_page <= self.max_page() {
                    self.page = new_page;
                    Action::Task(widget::scrollable::scroll_to(
                        self.scrollable_id.clone(),
                        widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                    ))
                } else {
                    Action::None
                }
            }
            InnerMessage::OuterMessage(outer) => Action::OuterMessage(*outer),
            InnerMessage::SortByColumn(column) => {
                self.sort(
                    column,
                    if self.sort_column == Some(column) {
                        !self.sort_reverse
                    } else {
                        false
                    },
                );
                self.page = 0;
                Action::Task(widget::scrollable::scroll_to(
                    self.scrollable_id.clone(),
                    widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                ))
            }
            InnerMessage::ScrollToTop => Action::Task(widget::scrollable::scroll_to(
                self.scrollable_id.clone(),
                widget::scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            )),
        }
    }

    pub fn items(&self) -> &Vec<T> {
        &self.items
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.inner_layout_id = uuid::Uuid::new_v4().as_u128() as isize;
        self.page = 0;
        self.sort_column = None;
        self.sort_reverse = false;
    }

    pub fn edit_items(&mut self, update: impl Fn(&mut Vec<T>)) {
        (update)(&mut self.items);
        self.inner_layout_id = uuid::Uuid::new_v4().as_u128() as isize;
        self.sort_column = None;
        self.sort_reverse = false;
        if self.page > self.max_page() {
            self.page = self.max_page();
        }
    }

    pub fn set_context(&mut self, context: C) {
        self.inner_layout_id = uuid::Uuid::new_v4().as_u128() as isize;
        self.context = context;
    }
}

pub struct TableView<'a, T, C, const COLUMNS: usize> {
    state: &'a State<T, C>,
    headers: Option<[String; COLUMNS]>,
    row_spacing: f32,
    column_spacing: f32,
    cell_padding: iced::Padding,
    max_column_sizes: [f32; COLUMNS],
    column_max_is_weak: [bool; COLUMNS],
}

impl<'a, T, C, const COLUMNS: usize> TableView<'a, T, C, COLUMNS> {
    pub fn new(state: &'a State<T, C>) -> Self {
        Self {
            state,
            headers: None,
            row_spacing: 10.0,
            column_spacing: 30.0,
            cell_padding: iced::Padding::new(10.0),
            max_column_sizes: [300.0; COLUMNS],
            column_max_is_weak: [false; COLUMNS],
        }
    }

    pub fn max_column_sizes(mut self, max_column_sizes: [f32; COLUMNS]) -> Self {
        self.max_column_sizes = max_column_sizes;
        self
    }

    pub fn column_max_is_weak(mut self, column_max_is_weak: [bool; COLUMNS]) -> Self {
        self.column_max_is_weak = column_max_is_weak;
        self
    }

    pub fn row_spacing(mut self, spacing: f32) -> Self {
        self.row_spacing = spacing;
        self
    }

    pub fn column_spacing(mut self, spacing: f32) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn cell_padding(mut self, padding: iced::Padding) -> Self {
        self.cell_padding = padding;
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

    pub fn view<Message: Clone + 'a>(
        self,
        to_row: impl Fn(&'a T, &'a C) -> [iced::Element<'a, Message>; COLUMNS] + 'a,
    ) -> iced::Element<'a, InnerMessage<Message>> {
        let headers = self.headers.clone().unwrap();
        let table = iced::widget::responsive(move |size| {
            let mut header_elements = Vec::new();
            for (index, header) in headers.iter().enumerate() {
                header_elements.push((
                    iced::Element::new(widget::text(header.clone())),
                    if self.state.sortable.contains(&index) {
                        iced::Element::new(
                            widget::button(
                                widget::svg::Svg::new(if Some(index) == self.state.sort_column {
                                    if self.state.sort_reverse {
                                        widget::svg::Handle::from_memory(include_bytes!(
                                            "../../../assets/filter-circle-fill.svg"
                                        ))
                                    } else {
                                        widget::svg::Handle::from_memory(include_bytes!(
                                            "../../../assets/filter-circle.svg"
                                        ))
                                    }
                                } else {
                                    widget::svg::Handle::from_memory(include_bytes!(
                                        "../../../assets/filter.svg"
                                    ))
                                })
                                .content_fit(iced::ContentFit::Fill)
                                .width(iced::Length::Shrink),
                            )
                            .padding(3)
                            .on_press(InnerMessage::SortByColumn(index)),
                        )
                    } else {
                        iced::Element::new(widget::Space::new(0.0, 0.0))
                    },
                ));
            }

            let mut cell_elements = Vec::new();
            for item_index in (self.state.page * self.state.page_size)
                ..(self
                    .state
                    .items
                    .len()
                    .min(self.state.page * self.state.page_size + self.state.page_size))
            {
                cell_elements.extend(
                    (to_row)(&self.state.items()[item_index], &self.state.context)
                        .map(|element| element.map(|x| InnerMessage::OuterMessage(Box::new(x)))),
                );
            }

            iced::Element::new(
                iced::widget::scrollable(
                    widget::container(inner_table_view::InnerTableView::new(
                        header_elements,
                        cell_elements,
                        self.max_column_sizes,
                        self.column_max_is_weak,
                        self.row_spacing,
                        self.column_spacing,
                        size.width,
                        self.cell_padding,
                        |theme: &iced::Theme| theme.extended_palette().background.strong.color,
                        |theme: &iced::Theme, row: usize| {
                            let factor = if row % 2 == 0 { 0.25 } else { 0.5 };
                            let mut weak = theme.extended_palette().background.weak.color;
                            let strong = theme.extended_palette().background.base.color;
                            weak.r += (strong.r - weak.r) * factor;
                            weak.g += (strong.g - weak.g) * factor;
                            weak.b += (strong.b - weak.b) * factor;
                            weak
                        },
                        self.state.inner_layout_id,
                    ))
                    .padding(iced::Padding::ZERO.bottom(10)),
                )
                .direction(iced::widget::scrollable::Direction::Both {
                    horizontal: iced::widget::scrollable::Scrollbar::new(),
                    vertical: iced::widget::scrollable::Scrollbar::new(),
                })
                .id(self.state.scrollable_id.clone())
                .width(iced::Fill),
            )
        });

        widget::column![
            table,
            super::spal_row![
                widget::button("Previous").on_press_maybe(if self.state.page == 0 {
                    None
                } else {
                    Some(InnerMessage::ChangePageBy(-1))
                }),
                widget::text!("Page {}/{}", self.state.page + 1, self.state.max_page() + 1),
                widget::button("Next").on_press_maybe(
                    if self.state.page == self.state.max_page() {
                        None
                    } else {
                        Some(InnerMessage::ChangePageBy(1))
                    }
                ),
                widget::button("Scroll to Top").on_press(InnerMessage::ScrollToTop)
            ]
        ]
        .into()
    }
}

pub fn table_view<T, C, const COLUMNS: usize>(state: &State<T, C>) -> TableView<'_, T, C, COLUMNS> {
    TableView::new(state)
}
