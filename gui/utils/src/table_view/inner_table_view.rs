use iced::advanced::Renderer;

pub struct InnerTableView<'a, Message, const COLUMNS: usize> {
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
    pub fn new(
        header_elements: Vec<(iced::Element<'a, Message>, iced::Element<'a, Message>)>,
        elements: Vec<iced::Element<'a, Message>>,
        max_column_sizes: [f32; COLUMNS],
        column_max_is_weak: [bool; COLUMNS],
        row_spacing: f32,
        column_spacing: f32,
        optimal_width: f32,
        cell_padding: iced::Padding,
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
        }
    }

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
                horizontal_padding_sum as f32
                    + column_widths.iter().sum::<f32>()
                    + COLUMNS as f32 * self.cell_padding.horizontal(),
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
