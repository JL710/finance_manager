struct State {
    column_widths: Option<Vec<f32>>,
}

pub struct InnerTableView<
    'a,
    Message,
    const COLUMNS: usize,
    Theme = iced::Theme,
    Renderer = iced::Renderer,
> where
    Renderer: iced::advanced::Renderer,
{
    /// name and sort svg/button represents one header
    header_elements: Vec<(
        iced::Element<'a, Message, Theme, Renderer>,
        iced::Element<'a, Message, Theme, Renderer>,
    )>,
    elements: Vec<iced::Element<'a, Message, Theme, Renderer>>,
    max_column_sizes: [f32; COLUMNS],
    /// if a column max is set to weak, it gets expanded if otherwise the widget with is smaller than [`Self::optimal_width`].
    column_max_is_weak: [bool; COLUMNS],
    row_spacing: f32,
    column_spacing: f32,
    optimal_width: f32,
    cell_padding: iced::Padding,
    header_background_color: Box<dyn Fn(&Theme) -> iced::Color>,
    row_color: Box<dyn Fn(&Theme, usize) -> iced::Color>,
}

impl<'a, Message, const COLUMNS: usize, Theme, Renderer>
    InnerTableView<'a, Message, COLUMNS, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        header_elements: Vec<(
            iced::Element<'a, Message, Theme, Renderer>,
            iced::Element<'a, Message, Theme, Renderer>,
        )>,
        elements: Vec<iced::Element<'a, Message, Theme, Renderer>>,
        max_column_sizes: [f32; COLUMNS],
        column_max_is_weak: [bool; COLUMNS],
        row_spacing: f32,
        column_spacing: f32,
        optimal_width: f32,
        cell_padding: iced::Padding,
        header_background_color: impl Fn(&Theme) -> iced::Color + 'static,
        row_color: impl Fn(&Theme, usize) -> iced::Color + 'static,
    ) -> Self {
        Self {
            header_elements,
            elements,
            max_column_sizes,
            column_max_is_weak,
            row_spacing,
            column_spacing,
            optimal_width,
            cell_padding,
            header_background_color: Box::new(header_background_color),
            row_color: Box::new(row_color),
        }
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
fn dynamic_header_layout_size<'a, Message, Theme, Renderer: iced::advanced::Renderer>(
    header_elements: &Vec<(
        iced::Element<'a, Message, Theme, Renderer>,
        iced::Element<'a, Message, Theme, Renderer>,
    )>,
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
        widths.push(left_layout.size().width + right_layout.size().width + 10.0); // + 10.0 for spacing
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
                table.optimal_width
                    - (reserved_space + column_widths.iter().flatten().sum::<f32>()),
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

impl<Message, const COLUMNS: usize, Theme, Renderer>
    iced::advanced::Widget<Message, Theme, Renderer>
    for InnerTableView<'_, Message, COLUMNS, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State {
            column_widths: None,
        })
    }

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
        let state = tree.state.downcast_mut::<State>();

        let state_is_valid = if let Some(column_widths) = &mut state.column_widths {
            column_widths.len() == COLUMNS
        } else {
            true
        };
        if !state_is_valid {
            state.column_widths = None;
        }

        tree.diff_children(&self.child_elements());
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        _limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let state = tree.state.downcast_mut::<State>();

        let rows = self.elements.len() / COLUMNS;
        let horizontal_padding_sum = (COLUMNS - 1) as f32 * self.column_spacing;

        let mut child_layouts = Vec::with_capacity(COLUMNS * 2 + COLUMNS * rows);

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
                renderer,
                &mut tree.children[self.header_elements.len() * 2..],
                horizontal_padding_sum,
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
                // make a space between both as bit as possible
                + space_between_them,
                self.cell_padding.top,
            ));

            total_height = total_height
                .max(layout_left.size().height + self.cell_padding.vertical())
                .max(layout_right.size().height + self.cell_padding.vertical())
        }

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

        iced::advanced::layout::Node::with_children(
            iced::Size::new(
                horizontal_padding_sum
                    + state.column_widths.as_ref().unwrap().iter().sum::<f32>()
                    + COLUMNS as f32 * self.cell_padding.horizontal(),
                total_height,
            ),
            child_layouts,
        )
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

        let mut child_draw_todos = self
            .child_elements()
            .into_iter()
            .zip(&tree.children)
            .zip(layout.children())
            .collect::<Vec<_>>();

        // draw heading
        let header_todos = pop_front_slice(&mut child_draw_todos, COLUMNS * 2);
        let (_, row_bottom_y) = layouts_max(
            layout.position().x,
            layout.position().y,
            &header_todos.iter().map(|x| x.1).collect::<Vec<_>>(),
        );
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle::new(
                    layout.position(),
                    (
                        self.optimal_width.max(row_width), // use optimal width or max header x depending on what is larger
                        row_bottom_y - layout.position().y + self.cell_padding.bottom,
                    )
                        .into(),
                ),
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            },
            iced::Background::Color((self.header_background_color)(theme)),
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

        // draw rows
        let mut row_index = 0;
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
                            self.optimal_width.max(row_width), // use optimal width or max header x depending on what is larger
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
    }

    fn operate(
        &self,
        state: &mut iced::advanced::widget::Tree,
        layout: iced::advanced::Layout<'_>,
        renderer: &Renderer,
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
        renderer: &Renderer,
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
        renderer: &Renderer,
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
        renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, Message, Theme, Renderer>> {
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

fn pop_front_slice<T>(vector: &mut Vec<T>, count: usize) -> Vec<T> {
    let mut result = Vec::new();
    for _ in 0..count {
        result.push(vector.remove(0));
    }
    result
}
