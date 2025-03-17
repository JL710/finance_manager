use iced::advanced::Renderer;
use iced::widget;
use std::collections::HashSet;

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
        self.page = 0;
        self.sort_column = None;
        self.sort_reverse = false;
    }

    pub fn edit_items(&mut self, update: impl Fn(&mut Vec<T>)) {
        (update)(&mut self.items);
        self.sort_column = None;
        self.sort_reverse = false;
        if self.page > self.max_page() {
            self.page = self.max_page();
        }
    }

    pub fn set_context(&mut self, context: C) {
        self.context = context;
    }
}

struct InnerTableView<'a, Message, const COLUMNS: usize> {
    /// name and sort svg/button represents one header
    header_elements: Vec<(iced::Element<'a, Message>, iced::Element<'a, Message>)>,
    elements: Vec<iced::Element<'a, Message>>,
    max_column_sizes: [f32; COLUMNS],
    /// if a column max is set to weak, it gets expanded if otherwise the widget with is smaller than [`Self::optimal_width`].
    column_max_is_weak: [bool; COLUMNS],
    row_spacing: f32,
    column_spacing: f32,
    optimal_width: f32,
    cell_padding: iced::Padding,
}

impl<'a, Message, const COLUMNS: usize> InnerTableView<'a, Message, COLUMNS> {
    fn child_elements(&self) -> Vec<&iced::Element<'a, Message>> {
        let mut children = Vec::with_capacity(self.header_elements.len() + self.elements.len());
        for header_element in &self.header_elements {
            children.push(&header_element.0);
            children.push(&header_element.1);
        }
        for element in &self.elements {
            children.push(element);
        }
        children
    }

    fn child_elements_mut(&mut self) -> Vec<&mut iced::Element<'a, Message>> {
        let mut children = Vec::with_capacity(self.header_elements.len() + self.elements.len());
        for header_element in &mut self.header_elements {
            children.push(&mut header_element.0);
            children.push(&mut header_element.1);
        }
        for element in &mut self.elements {
            children.push(element);
        }
        children
    }
}

fn max_node_width(nodes: &[iced::advanced::layout::Node]) -> f32 {
    let mut smallest = 0.0;
    for node in nodes {
        if node.size().width > smallest {
            smallest = node.size().width;
        }
    }
    smallest
}

fn unpositioned_header_layouts<'a, Message>(
    header_elements: &Vec<(iced::Element<'a, Message>, iced::Element<'a, Message>)>,
    max_column_sizes: &[f32],
    renderer: &iced::Renderer,
    states: &mut [iced::advanced::widget::Tree],
) -> (Vec<iced::advanced::layout::Node>, Vec<f32>) {
    let mut nodes = Vec::with_capacity(header_elements.len() * 2);
    let mut widths = Vec::with_capacity(header_elements.len());

    for (column_index, (left_element, right_element)) in header_elements.iter().enumerate() {
        let left_layout = left_element.as_widget().layout(
            &mut states[column_index * 2],
            renderer,
            &iced::advanced::layout::Limits::new(
                iced::Size::ZERO,
                iced::Size::new(max_column_sizes[column_index], f32::MAX),
            ),
        );
        let right_layout = right_element.as_widget().layout(
            &mut states[column_index * 2 + 1],
            renderer,
            &iced::advanced::layout::Limits::new(
                iced::Size::ZERO,
                iced::Size::new(
                    max_column_sizes[column_index] - left_layout.size().width,
                    f32::MAX,
                ),
            ),
        );
        widths.push(left_layout.size().width + right_layout.size().width + 10.0); // + 10.0 for spacing
        nodes.push(left_layout);
        nodes.push(right_layout);
    }

    (nodes, widths)
}

fn unpositioned_cell_layouts<'a, Message, const COLUMNS: usize>(
    table: &InnerTableView<'a, Message, COLUMNS>,
    renderer: &iced::Renderer,
    states: &mut [iced::advanced::widget::Tree],
    reserved_space: f32,
) -> ([Vec<iced::advanced::layout::Node>; COLUMNS], Vec<f32>) {
    let mut column_layouts = [const { Vec::new() }; COLUMNS];
    let mut column_widths: [Option<f32>; COLUMNS] = [None; COLUMNS];

    // iterate over columns -> strong max ones first
    for (column_index, weak_max) in table
        .column_max_is_weak
        .into_iter()
        .enumerate()
        .filter(|x| !x.1)
        .chain(
            table
                .column_max_is_weak
                .into_iter()
                .enumerate()
                .filter(|x| x.1),
        )
    {
        let mut max_width = table.max_column_sizes[column_index];
        // if max is weak expand max_width if possible
        if weak_max {
            let space_left = (0.0 as f32).max(
                table.optimal_width
                    - (reserved_space as f32
                        + column_widths
                            .iter()
                            .filter(|x| x.is_some())
                            .map(|x| x.unwrap())
                            .sum::<f32>()),
            );
            if space_left > 0.0 {
                let spaced_needed = column_widths
                    .iter()
                    .zip(table.max_column_sizes.iter())
                    .filter(|x| x.0.is_some())
                    .map(|x| x.1)
                    .sum::<f32>();
                if spaced_needed < space_left {
                    max_width += space_left - spaced_needed;
                }
            }
        }
        // do layout for each cell element
        if table.elements.len() / COLUMNS > 0 {
            for element_index in (column_index..table.elements.len()).step_by(COLUMNS) {
                column_layouts[column_index].push(table.elements[element_index].as_widget().layout(
                    &mut states[element_index],
                    renderer,
                    &iced::advanced::layout::Limits::new(
                        iced::Size::new(0.0, 0.0),
                        iced::Size::new(max_width, f32::MAX),
                    ),
                ))
            }
        }

        // save column width
        column_widths[column_index] = Some(max_node_width(&column_layouts[column_index]));
    }

    (
        column_layouts,
        column_widths.iter().map(|x| x.unwrap()).collect::<Vec<_>>(),
    )
}

impl<'a, Message, const COLUMNS: usize> iced::advanced::Widget<Message, iced::Theme, iced::Renderer>
    for InnerTableView<'a, Message, COLUMNS>
{
    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size::new(iced::Shrink, iced::Shrink)
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.child_elements()
            .into_iter()
            .map(iced::advanced::widget::Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        tree.diff_children(&self.child_elements());
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced::Renderer,
        _limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let rows = self.elements.len() / COLUMNS;
        let horizontal_padding_sum = (COLUMNS - 1) as f32 * self.column_spacing;

        let (header_layouts, header_widths) = unpositioned_header_layouts(
            &self.header_elements,
            &self.max_column_sizes,
            renderer,
            &mut tree.children[0..(self.header_elements.len() * 2)],
        );

        let (column_layouts, element_column_widths) = unpositioned_cell_layouts(
            self,
            renderer,
            &mut tree.children[self.header_elements.len() * 2..],
            horizontal_padding_sum,
        );

        // calculate column_widths
        let mut column_widths = Vec::with_capacity(COLUMNS);
        for column_index in 0..COLUMNS {
            column_widths
                .push(header_widths[column_index].max(element_column_widths[column_index]));
        }

        // calculate column start/x positions
        let mut column_start_positions = Vec::with_capacity(COLUMNS);
        column_start_positions.push(0.0 + self.cell_padding.left);
        for i in 1..COLUMNS {
            column_start_positions.push(
                column_start_positions[i - 1]
                    + column_widths[i - 1]
                    + self.column_spacing
                    + self.cell_padding.horizontal(),
            );
        }

        // generate child layouts
        let mut child_layouts = Vec::with_capacity(header_layouts.len() + self.elements.len());
        let mut total_height: f32 = 0.0 + self.cell_padding.top;

        // generate header layouts
        for column_index in 0..COLUMNS {
            let layout_left = header_layouts[column_index * 2]
                .clone()
                .move_to((column_start_positions[column_index], self.cell_padding.top));
            let mut layout_right = header_layouts[column_index * 2 + 1].clone();
            let space_between_them =
                column_widths[column_index] - layout_left.size().width - layout_right.size().width;
            layout_right = layout_right.clone().move_to((
                column_start_positions[column_index] + layout_left.size().width
                // make a space between both as bit as possible
                + space_between_them,
                self.cell_padding.top,
            ));

            child_layouts.push(layout_left);
            child_layouts.push(layout_right);

            total_height = total_height
                .max(header_layouts[column_index * 2].size().height + self.cell_padding.vertical())
                .max(
                    header_layouts[column_index * 2 + 1].size().height
                        + self.cell_padding.vertical(),
                )
        }

        // generate cell layouts
        for row_index in 0..rows {
            let y = total_height + self.row_spacing + self.cell_padding.vertical();
            for column_index in 0..COLUMNS {
                if total_height < y + column_layouts[column_index][row_index].size().height {
                    total_height = y + column_layouts[column_index][row_index].size().height;
                }
                child_layouts.push(
                    column_layouts[column_index][row_index]
                        .clone() // FIXME: remote this clone if possible
                        .move_to(iced::Point::new(column_start_positions[column_index], y)),
                );
            }
        }

        iced::advanced::layout::Node::with_children(
            iced::Size::new(
                horizontal_padding_sum as f32 + column_widths.iter().sum::<f32>(),
                total_height,
            ),
            child_layouts,
        )
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let mut child_draw_todos = self
            .child_elements()
            .into_iter()
            .zip(&tree.children)
            .zip(layout.children())
            .collect::<Vec<_>>();

        // draw heading border
        let mut max_header_border_x: f32 = layout.position().x;
        let mut max_header_border_y: f32 = layout.position().y;
        let header_todos = pop_front_slice(&mut child_draw_todos, COLUMNS * 2);
        for (_, child_layout) in &header_todos {
            max_header_border_x =
                max_header_border_x.max(child_layout.position().x + child_layout.bounds().width);
            max_header_border_y =
                max_header_border_y.max(child_layout.position().y + child_layout.bounds().height);
        }
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle::new(
                    layout.position(),
                    (
                        self.optimal_width.max(
                            max_header_border_x - layout.position().x
                                + self.cell_padding.horizontal(),
                        ), // use optimal width or max header x depending on what is larger
                        max_header_border_y - layout.position().y + self.cell_padding.bottom,
                    )
                        .into(),
                ),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            },
            iced::Background::Color(theme.extended_palette().background.strong.color),
        );
        for ((child, state), child_layout) in header_todos {
            child.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                child_layout,
                cursor,
                viewport,
            );
        }

        // draw row border
        let mut row_index = 0;
        while !child_draw_todos.is_empty() {
            let y_cell_start = child_draw_todos[0].1.position().y;
            let mut max_row_border_x: f32 = layout.position().x;
            let mut max_row_border_y: f32 = layout.position().y;
            let row_todos = pop_front_slice(&mut child_draw_todos, COLUMNS);
            for (_, child_layout) in &row_todos {
                max_row_border_x =
                    max_row_border_x.max(child_layout.position().x + child_layout.bounds().width);
                max_row_border_y =
                    max_row_border_y.max(child_layout.position().y + child_layout.bounds().height);
            }
            renderer.fill_quad(
                iced::advanced::renderer::Quad {
                    bounds: iced::Rectangle::new(
                        (layout.position().x, y_cell_start - self.cell_padding.top).into(),
                        (
                            self.optimal_width.max(
                                max_row_border_x - layout.position().x
                                    + self.cell_padding.horizontal(),
                            ), // use optimal width or max header x depending on what is larger
                            max_row_border_y - y_cell_start + self.cell_padding.vertical(),
                        )
                            .into(),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                },
                iced::Background::Color(row_background_color(row_index, theme)),
            );
            for ((child, state), child_layout) in row_todos {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    viewport,
                );
            }

            row_index += 1;
        }
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.child_elements()
                .into_iter()
                .zip(&mut state.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        state: &mut iced::advanced::widget::Tree,
        event: iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let mut child_layouts = layout.children();
        let mut child_states = state.children.iter_mut();
        for element in self.child_elements_mut() {
            if let iced::event::Status::Captured = element.as_widget_mut().on_event(
                child_states.next().unwrap(),
                event.clone(),
                child_layouts.next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            ) {
                return iced::event::Status::Captured;
            }
        }

        iced::event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        state: &iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &iced::Renderer,
    ) -> iced::advanced::mouse::Interaction {
        self.child_elements()
            .iter()
            .zip(&state.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &iced::Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let mut child_layouts = layout.children();
        let mut child_states = state.children.iter_mut();

        let mut children = Vec::new();
        for element in self.child_elements_mut() {
            if let Some(overlay_element) = element.as_widget_mut().overlay(
                child_states.next().unwrap(),
                child_layouts.next().unwrap(),
                renderer,
                translation,
            ) {
                children.push(overlay_element);
            }
        }

        (!children.is_empty())
            .then(|| iced::advanced::overlay::Group::with_children(children).overlay())
    }
}

pub type AlignmentFunction<'a, T> = dyn Fn(&T, usize, usize) -> iced::alignment::Horizontal + 'a;

pub struct TableView<'a, T, C, const COLUMNS: usize> {
    state: &'a State<T, C>,
    headers: Option<[String; COLUMNS]>,
    row_spacing: f32,
    column_spacing: f32,
    cell_padding: iced::Padding,
}

impl<'a, T, C, const COLUMNS: usize> TableView<'a, T, C, COLUMNS> {
    pub fn new(state: &'a State<T, C>) -> Self {
        Self {
            state,
            headers: None,
            row_spacing: 10.0,
            column_spacing: 30.0,
            cell_padding: iced::Padding::new(10.0),
        }
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
                                widget::svg::Svg::new(widget::svg::Handle::from_memory(
                                    std::borrow::Cow::from(if Some(index) == self.state.sort_column {
                                        if self.state.sort_reverse {
                                            &include_bytes!("../../assets/filter-circle-fill.svg")[..]
                                        } else {
                                            &include_bytes!("../../assets/filter-circle.svg")[..]
                                        }
                                    } else {
                                        &include_bytes!("../../assets/filter.svg")[..]
                                    }),
                                ))
                                .content_fit(iced::ContentFit::Fill)
                                .width(iced::Length::Shrink),
                            )
                            .padding(3)
                            .on_press(InnerMessage::SortByColumn(index)),
                        )
                    } else {
                        iced::Element::new(widget::Space::new(0.0, 0.0))
                    }
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
                iced::widget::scrollable(iced::Element::new(InnerTableView {
                    header_elements,
                    elements: cell_elements,
                    max_column_sizes: [400.0; COLUMNS],
                    column_max_is_weak: [false; COLUMNS],
                    row_spacing: self.row_spacing,
                    column_spacing: self.column_spacing,
                    optimal_width: size.width,
                    cell_padding: self.cell_padding,
                }))
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

fn row_background_color(row_index: usize, theme: &iced::Theme) -> iced::Color {
    let factor = if row_index % 2 == 0 { 0.25 } else { 0.5 };
    let mut weak = theme.extended_palette().background.weak.color;
    let strong = theme.extended_palette().background.base.color;
    weak.r += (strong.r - weak.r) * factor;
    weak.g += (strong.g - weak.g) * factor;
    weak.b += (strong.b - weak.b) * factor;
    weak
}

fn pop_front_slice<T>(vector: &mut Vec<T>, count: usize) -> Vec<T> {
    let mut result = Vec::new();
    for _ in 0..count {
        result.push(vector.remove(0));
    }
    result
}
