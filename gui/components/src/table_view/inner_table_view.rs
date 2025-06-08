use iced::advanced;

#[derive(Debug)]
struct State {
    column_widths: Option<Vec<f32>>,
    layout_id: isize,
    scroll_state: crate::scrollable::State,
}

type HeaderElementPair<'a, Message, Theme, Renderer> = (
    iced::Element<'a, Message, Theme, Renderer>,
    iced::Element<'a, Message, Theme, Renderer>,
);

type RowColorStyleFn<Theme> = Box<dyn Fn(&Theme, usize) -> iced::Color>;

pub struct InnerTableView<
    'a,
    Message,
    const COLUMNS: usize,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> where
    Renderer: iced::advanced::Renderer,
{
    scrollable_id: Option<iced::advanced::widget::Id>,
    /// name and sort svg/button represents one header
    header_elements: Vec<HeaderElementPair<'a, Message, Theme, Renderer>>,
    elements: Vec<iced::Element<'a, Message, Theme, Renderer>>,
    max_column_sizes: [f32; COLUMNS],
    /// if a column max is set to weak, it gets expanded if otherwise the widget with is smaller than [`Self::optimal_width`].
    column_max_is_weak: [bool; COLUMNS],
    row_spacing: f32,
    column_spacing: f32,
    cell_padding: iced::Padding,
    header_background_color: Box<dyn Fn(&Theme) -> iced::Color>,
    row_color: RowColorStyleFn<Theme>,
    layout_id: isize,
}

impl<'a, Message, const COLUMNS: usize, Theme, Renderer>
    InnerTableView<'a, Message, COLUMNS, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        header_elements: Vec<HeaderElementPair<'a, Message, Theme, Renderer>>,
        elements: Vec<iced::Element<'a, Message, Theme, Renderer>>,
        max_column_sizes: [f32; COLUMNS],
        column_max_is_weak: [bool; COLUMNS],
        row_spacing: f32,
        column_spacing: f32,
        cell_padding: iced::Padding,
        header_background_color: impl Fn(&Theme) -> iced::Color + 'static,
        row_color: impl Fn(&Theme, usize) -> iced::Color + 'static,
        layout_id: isize,
    ) -> Self {
        Self {
            scrollable_id: None,
            header_elements,
            elements,
            max_column_sizes,
            column_max_is_weak,
            row_spacing,
            column_spacing,
            cell_padding,
            header_background_color: Box::new(header_background_color),
            row_color: Box::new(row_color),
            layout_id,
        }
    }

    fn child_header_elements(&self) -> Vec<&iced::Element<'a, Message, Theme, Renderer>> {
        let mut children = Vec::with_capacity(COLUMNS * 2);
        for pair in &self.header_elements {
            children.push(&pair.0);
            children.push(&pair.1);
        }
        children
    }

    fn child_header_elements_mut(
        &mut self,
    ) -> Vec<&mut iced::Element<'a, Message, Theme, Renderer>> {
        let mut children = Vec::with_capacity(COLUMNS * 2);
        for pair in &mut self.header_elements {
            children.push(&mut pair.0);
            children.push(&mut pair.1);
        }
        children
    }

    fn child_elements(&self) -> Vec<&iced::Element<'a, Message, Theme, Renderer>> {
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

    fn child_elements_mut(&mut self) -> Vec<&mut iced::Element<'a, Message, Theme, Renderer>> {
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

    pub fn id(mut self, id: iced::advanced::widget::Id) -> Self {
        self.scrollable_id = Some(id);
        self
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

fn layouts_max(
    start_x: f32,
    start_y: f32,
    layouts: &[iced::advanced::layout::Layout<'_>],
) -> (f32, f32) {
    let mut x: f32 = start_x;
    let mut y: f32 = start_y;
    for child_layout in layouts {
        x = x.max(child_layout.position().x + child_layout.bounds().width);
        y = y.max(child_layout.position().y + child_layout.bounds().height);
    }
    (x, y)
}

/// Generates the dynamic layout sizes.
/// Assumes that the order of elements is left to right.
///
/// Returns the list of [`iced::advanced::layout::Node`]s for each cell and sizes of the columns.
fn dynamic_header_layout_size<Message, Theme, Renderer: iced::advanced::Renderer>(
    header_elements: &Vec<HeaderElementPair<'_, Message, Theme, Renderer>>,
    max_column_sizes: &[f32],
    renderer: &Renderer,
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
        widths.push((left_layout.size().width + right_layout.size().width).ceil() + 10.0); // + 10.0 for spacing + do the ceil because of missing float precision issues that lead to visual artifacts
        nodes.push(left_layout);
        nodes.push(right_layout);
    }

    (nodes, widths)
}

/// Generates the dynamic layout sizes.
/// Assumes that the order of elements is left to right, row for row.
///
/// Returns the list of [`iced::advanced::layout::Node`]s for each cell and sizes of the columns.
fn dynamic_cell_layout_size<
    Message,
    const COLUMNS: usize,
    Theme,
    Renderer: iced::advanced::Renderer,
>(
    table: &InnerTableView<'_, Message, COLUMNS, Theme, Renderer>,
    optimal_width: f32,
    renderer: &Renderer,
    states: &mut [iced::advanced::widget::Tree],
    reserved_space: f32,
) -> (Vec<iced::advanced::layout::Node>, Vec<f32>) {
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
            let space_left = (0.0_f32).max(
                optimal_width - (reserved_space + column_widths.iter().flatten().sum::<f32>()),
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
        column_widths[column_index] = Some(max_node_width(&column_layouts[column_index]).ceil()); // do the ceil because of missing float precision issues that lead to visual artifacts
    }

    let mut cell_layouts = Vec::with_capacity(table.elements.len());
    for row_index in 0..(table.elements.len() / COLUMNS) {
        for column_layout in column_layouts.iter().take(COLUMNS) {
            cell_layouts.push(column_layout[row_index].clone());
        }
    }

    (
        cell_layouts,
        column_widths.iter().map(|x| x.unwrap()).collect::<Vec<_>>(),
    )
}

impl<Message, const COLUMNS: usize, Theme: crate::scrollable::Catalog, Renderer>
    iced::advanced::Widget<Message, Theme, Renderer>
    for InnerTableView<'_, Message, COLUMNS, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State {
            column_widths: None,
            layout_id: self.layout_id,
            scroll_state: crate::scrollable::State::default(),
        })
    }

    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size::new(iced::Fill, iced::Fill)
    }

    fn children(&self) -> Vec<iced::advanced::widget::Tree> {
        self.child_elements()
            .into_iter()
            .map(iced::advanced::widget::Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut iced::advanced::widget::Tree) {
        if let iced::advanced::widget::tree::State::Some(state) = &mut tree.state {
            let state: &mut State = state.downcast_mut().expect("Could not downcast state");
            let column_widths_valid = if let Some(column_widths) = &mut state.column_widths {
                column_widths.len() == COLUMNS
            } else {
                true
            };

            if !column_widths_valid || state.layout_id != self.layout_id {
                state.column_widths = None;
                state.layout_id = self.layout_id
            }
        } else {
            tree.state = self.state();
        }

        tree.diff_children(&self.child_elements());
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let rows = self.elements.len() / COLUMNS;
        let horizontal_padding_spacing_sum = (COLUMNS - 1) as f32 * self.column_spacing
            + COLUMNS as f32 * self.cell_padding.horizontal();

        let mut child_layouts = Vec::with_capacity(COLUMNS * 2 + COLUMNS * rows + 2);

        let column_widths = if let Some(col_widths) = state.column_widths.clone() {
            let mut child_state_iterator = tree.children.iter_mut();

            for (column_index, column_width) in col_widths.iter().enumerate().take(COLUMNS) {
                child_layouts.push(self.header_elements[column_index].0.as_widget().layout(
                    child_state_iterator.next().unwrap(),
                    renderer,
                    &iced::advanced::layout::Limits::new(
                        iced::Size::new(0.0, 0.0),
                        iced::Size::new(*column_width, f32::INFINITY),
                    ),
                ));
                child_layouts.push(self.header_elements[column_index].1.as_widget().layout(
                    child_state_iterator.next().unwrap(),
                    renderer,
                    &iced::advanced::layout::Limits::new(
                        iced::Size::new(0.0, 0.0),
                        iced::Size::new(*column_width, f32::INFINITY),
                    ),
                ));
            }

            for row_index in 0..rows {
                for (column_index, column_width) in col_widths.iter().enumerate().take(COLUMNS) {
                    child_layouts.push(
                        self.elements[row_index * COLUMNS + column_index]
                            .as_widget()
                            .layout(
                                child_state_iterator.next().unwrap(),
                                renderer,
                                &iced::advanced::layout::Limits::new(
                                    iced::Size::new(0.0, 0.0),
                                    iced::Size::new(*column_width, f32::INFINITY),
                                ),
                            ),
                    )
                }
            }

            col_widths
        } else {
            let (header_layouts, header_widths) = dynamic_header_layout_size(
                &self.header_elements,
                &self.max_column_sizes,
                renderer,
                &mut tree.children[0..(self.header_elements.len() * 2)],
            );
            child_layouts.extend(header_layouts);

            let (cell_layouts, element_column_widths) = dynamic_cell_layout_size(
                self,
                limits.max().width,
                renderer,
                &mut tree.children[self.header_elements.len() * 2..],
                horizontal_padding_spacing_sum,
            );
            child_layouts.extend(cell_layouts);

            // calculate column_widths
            let mut column_widths = Vec::with_capacity(COLUMNS);
            for column_index in 0..COLUMNS {
                column_widths
                    .push(header_widths[column_index].max(element_column_widths[column_index]));
            }

            state.column_widths = Some(column_widths.clone());

            column_widths
        };

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
        let mut total_height: f32 = 0.0 + self.cell_padding.top;
        let mut child_layout_iterator = child_layouts.iter_mut();

        // generate header layouts
        for column_index in 0..COLUMNS {
            let layout_left = child_layout_iterator.next().unwrap();
            layout_left.move_to_mut((column_start_positions[column_index], self.cell_padding.top));

            let layout_right = child_layout_iterator.next().unwrap();
            let space_between_them =
                column_widths[column_index] - layout_left.size().width - layout_right.size().width;
            layout_right.move_to_mut((
                column_start_positions[column_index] + layout_left.size().width
                // make a space between both as big as possible
                + space_between_them,
                self.cell_padding.top,
            ));

            total_height = total_height
                .max(layout_left.size().height + self.cell_padding.vertical())
                .max(layout_right.size().height + self.cell_padding.vertical())
        }
        let header_height = total_height;

        // generate cell layouts
        for _ in 0..rows {
            let y = total_height + self.row_spacing + self.cell_padding.vertical();
            for (column_index, cell_layout) in
                (&mut child_layout_iterator).take(COLUMNS).enumerate()
            {
                if total_height < y + cell_layout.size().height {
                    total_height = y + cell_layout.size().height;
                }
                cell_layout.move_to_mut(iced::Point::new(column_start_positions[column_index], y));
            }
        }
        total_height += self.cell_padding.bottom;

        let inner_width = horizontal_padding_spacing_sum + column_widths.iter().sum::<f32>();
        let header_node = advanced::layout::Node::new((inner_width, header_height).into());
        let scrollable_node =
            advanced::layout::Node::new((inner_width, total_height - header_height).into());
        child_layouts.insert(0, scrollable_node);
        child_layouts.insert(0, header_node);

        iced::advanced::layout::Node::with_children(limits.max(), child_layouts)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let row_width = state.column_widths.as_ref().unwrap().iter().sum::<f32>()
            + COLUMNS as f32 * self.cell_padding.horizontal()
            + 0.0_f32.max((COLUMNS - 1) as f32) * self.column_spacing;

        let mut child_layout_iterator = layout.children();
        let header_layout = child_layout_iterator.next().unwrap();
        let _scrollable_layout = child_layout_iterator.next().unwrap();

        let mut child_draw_todos = self
            .child_elements()
            .into_iter()
            .zip(&tree.children)
            .zip(child_layout_iterator)
            .collect::<Vec<_>>();

        // draw heading
        let header_todos = pop_front_slice(&mut child_draw_todos, COLUMNS * 2);
        let (row_bottom_x, row_bottom_y) = layouts_max(
            layout.position().x,
            layout.position().y,
            &header_todos.iter().map(|x| x.1).collect::<Vec<_>>(),
        );
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle::new(
                    layout.position(),
                    (
                        layout.bounds().width,
                        row_bottom_y - layout.position().y + self.cell_padding.bottom,
                    )
                        .into(),
                ),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            },
            iced::Background::Color((self.header_background_color)(theme)),
        );
        let header_y_end = row_bottom_y + self.cell_padding.bottom;
        let header_x_end = row_bottom_x + self.cell_padding.right;
        let header_outer_bounds = header_layout
            .bounds()
            .intersection(&layout.bounds())
            .unwrap_or(iced::Rectangle {
                width: 0.0,
                height: 0.0,
                x: header_layout.position().x,
                y: header_layout.position().y,
            });
        renderer.with_layer(header_outer_bounds, |renderer| {
            renderer.with_translation(
                iced::Vector::new(
                    state
                        .scroll_state
                        .translation(header_outer_bounds.size(), header_layout.bounds().size())
                        .x,
                    0.0,
                ),
                |renderer| {
                    for ((child, child_state), child_layout) in header_todos {
                        child.as_widget().draw(
                            child_state,
                            renderer,
                            theme,
                            style,
                            child_layout,
                            if let iced::mouse::Cursor::Available(point) = cursor {
                                iced::mouse::Cursor::Available(
                                    point
                                        - iced::Vector::new(
                                            state
                                                .scroll_state
                                                .translation(
                                                    header_outer_bounds.size(),
                                                    header_layout.bounds().size(),
                                                )
                                                .x,
                                            0.0,
                                        ),
                                )
                            } else {
                                cursor
                            },
                            &(*viewport
                                - iced::Vector::new(
                                    state
                                        .scroll_state
                                        .translation(
                                            header_outer_bounds.size(),
                                            header_layout.bounds().size(),
                                        )
                                        .x,
                                    0.0,
                                )),
                        );
                    }
                },
            );
        });

        // draw rows
        let mut row_index = 0;
        let row_layouts = child_draw_todos.iter().map(|x| x.1).collect::<Vec<_>>();
        let rows_x_end = header_x_end
            .max(crate::scrollable::x_start_end(&row_layouts).1 + self.cell_padding.right);
        let rows_y_end = crate::scrollable::y_start_end(&row_layouts).1 + self.cell_padding.bottom;
        crate::scrollable::draw(
            &state.scroll_state,
            renderer,
            theme,
            style,
            cursor,
            iced::Size::new(rows_x_end - layout.position().x, rows_y_end - header_y_end),
            iced::Rectangle {
                y: header_y_end,
                height: layout.bounds().height - (header_y_end - layout.bounds().y),
                ..layout.bounds()
            },
            viewport,
            |renderer, viewport, cursor| {
                while !child_draw_todos.is_empty() {
                    let y_cell_start = child_draw_todos[0].1.position().y;
                    let row_todos = pop_front_slice(&mut child_draw_todos, COLUMNS);
                    let (_, row_bottom_y) = layouts_max(
                        layout.position().x,
                        layout.position().y,
                        &row_todos.iter().map(|x| x.1).collect::<Vec<_>>(),
                    );
                    renderer.fill_quad(
                        iced::advanced::renderer::Quad {
                            bounds: iced::Rectangle::new(
                                (layout.position().x, y_cell_start - self.cell_padding.top).into(),
                                (
                                    layout.bounds().width.max(row_width), // use optimal width or max header x depending on what is larger
                                    row_bottom_y - y_cell_start + self.cell_padding.vertical(),
                                )
                                    .into(),
                            ),
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        },
                        iced::Background::Color((self.row_color)(theme, row_index)),
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
            },
        );
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let mut layout_child_iterator = layout.children();
        let _header_layout = layout_child_iterator.next().unwrap();
        let scrollable_layout = layout_child_iterator.next().unwrap();

        let downcast_state: &mut State = state.state.downcast_mut();
        let translation = downcast_state.scroll_state.translation(
            layout
                .bounds()
                .intersection(&scrollable_layout.bounds())
                .unwrap()
                .size(),
            scrollable_layout.bounds().size(),
        );
        operation.scrollable(
            &mut downcast_state.scroll_state,
            self.scrollable_id.as_ref(),
            layout.bounds(),
            layout.bounds(),
            translation,
        );

        operation.container(None, layout.bounds(), &mut |operation| {
            self.child_elements()
                .into_iter()
                .zip(&mut state.children)
                .zip(&mut layout_child_iterator)
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
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced::advanced::graphics::core::event::Status {
        let mut child_layouts = layout.children();
        let header_layout = child_layouts.next().unwrap();
        let scrollable_layout = child_layouts.next().unwrap();
        let downcast_state: &mut State = state.state.downcast_mut();
        if crate::scrollable::scroll_grab_on_event(
            &mut downcast_state.scroll_state,
            event.clone(),
            cursor,
            layout
                .bounds()
                .shrink(iced::Padding::ZERO.top(header_layout.bounds().height)),
            scrollable_layout.bounds().size(),
        ) == iced::advanced::graphics::core::event::Status::Captured
        {
            return iced::advanced::graphics::core::event::Status::Captured;
        }
        if crate::scrollable::scroll_wheel_on_event(
            &mut downcast_state.scroll_state,
            event.clone(),
            cursor,
            layout
                .bounds()
                .shrink(iced::Padding::ZERO.top(header_layout.bounds().height)),
            scrollable_layout.bounds().size(),
        ) == iced::advanced::graphics::core::event::Status::Captured
        {
            return iced::advanced::graphics::core::event::Status::Captured;
        }

        let outer_scrollable_size = scrollable_layout
            .bounds()
            .intersection(&layout.bounds())
            .unwrap_or(iced::Rectangle {
                width: 0.0,
                height: 0.0,
                x: scrollable_layout.position().x,
                y: scrollable_layout.position().y,
            })
            .size();
        let translation = downcast_state
            .scroll_state
            .translation(outer_scrollable_size, scrollable_layout.bounds().size());
        let mut child_states = state.children.iter_mut();
        for header_element in self.child_header_elements_mut() {
            if let iced::event::Status::Captured = header_element.as_widget_mut().on_event(
                child_states.next().unwrap(),
                event.clone(),
                child_layouts.next().unwrap(),
                if let iced::mouse::Cursor::Available(point) = cursor {
                    iced::mouse::Cursor::Available(point - iced::Vector::new(translation.x, 0.0))
                } else {
                    cursor
                },
                renderer,
                clipboard,
                shell,
                &(*viewport - iced::Vector::new(translation.x, 0.0)),
            ) {
                return iced::event::Status::Captured;
            }
        }
        for element in &mut self.elements {
            if let iced::event::Status::Captured = crate::scrollable::on_event(
                &downcast_state.scroll_state,
                element.as_widget_mut(),
                outer_scrollable_size,
                scrollable_layout.bounds().size(),
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
        renderer: &Renderer,
    ) -> iced::advanced::mouse::Interaction {
        let downcast_state: &State = state.state.downcast_ref();

        let mut child_layouts = layout.children();
        let _header_layout = child_layouts.next().unwrap();
        let scrollable_layout = child_layouts.next().unwrap();

        let mut child_states = state.children.iter();

        let translation = downcast_state.scroll_state.translation(
            scrollable_layout
                .bounds()
                .intersection(&layout.bounds())
                .unwrap_or(iced::Rectangle {
                    width: 0.0,
                    height: 0.0,
                    x: scrollable_layout.position().x,
                    y: scrollable_layout.position().y,
                })
                .size(),
            scrollable_layout.bounds().size(),
        );

        let header = self
            .child_header_elements()
            .iter()
            .zip(child_states.by_ref().take(COLUMNS * 2))
            .zip(child_layouts.by_ref().take(COLUMNS * 2))
            .map(|((child, child_state), layout)| {
                child.as_widget().mouse_interaction(
                    child_state,
                    layout,
                    if let iced::mouse::Cursor::Available(point) = cursor {
                        iced::mouse::Cursor::Available(
                            point - iced::Vector::new(translation.x, 0.0),
                        )
                    } else {
                        cursor
                    },
                    &(*viewport - iced::Vector::new(translation.x, 0.0)),
                    renderer,
                )
            })
            .max()
            .unwrap_or_default();

        let cells = self
            .elements
            .iter()
            .zip(child_states)
            .zip(child_layouts)
            .map(|((child, child_state), child_layout)| {
                crate::scrollable::mouse_interaction(
                    &downcast_state.scroll_state,
                    scrollable_layout
                        .bounds()
                        .intersection(&layout.bounds())
                        .unwrap_or(iced::Rectangle {
                            width: 0.0,
                            height: 0.0,
                            x: scrollable_layout.position().x,
                            y: scrollable_layout.position().y,
                        }),
                    scrollable_layout.bounds().size(),
                    child.as_widget(),
                    child_state,
                    child_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default();

        [header, cells].into_iter().max().unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let mut child_layouts = layout.children();
        let _header_layout = child_layouts.next().unwrap();
        let scrollable_layout = child_layouts.next().unwrap();
        let mut child_states = state.children.iter_mut();

        let mut children = Vec::new();
        for element in self.child_elements_mut() {
            if let Some(overlay_element) = crate::scrollable::overlay(
                &state.state.downcast_mut::<State>().scroll_state,
                layout.bounds().size(),
                scrollable_layout.bounds().size(),
                element.as_widget_mut(),
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

fn pop_front_slice<T>(vector: &mut Vec<T>, count: usize) -> Vec<T> {
    let mut result = Vec::new();
    for _ in 0..count {
        result.push(vector.remove(0));
    }
    result
}

impl<'a, Message: 'a, const COLUMNS: usize, Theme: crate::scrollable::Catalog + 'a, Renderer: 'a>
    From<InnerTableView<'a, Message, COLUMNS, Theme, Renderer>>
    for iced::Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn from(value: InnerTableView<'a, Message, COLUMNS, Theme, Renderer>) -> Self {
        iced::Element::new(value)
    }
}
