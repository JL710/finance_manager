use iced::Size;
use iced::advanced;
use iced::advanced::layout::{Limits, Node};
pub use iced::widget::scrollable::{AbsoluteOffset, Rail, RelativeOffset, Status};

pub struct Style {
    pub vertical_rail: Rail,
    pub horizontal_rail: Rail,
    pub gap: Option<iced::Background>,
}

static SCROLLBAR_THICKNESS: f32 = 10.0;

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|theme: &Self, status: Status| {
            let style = iced::widget::scrollable::default(theme, status);
            Style {
                vertical_rail: style.vertical_rail,
                horizontal_rail: style.horizontal_rail,
                gap: style.gap,
            }
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        (class)(self, status)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
    Both,
}

impl Direction {
    pub fn vertical(&self) -> bool {
        !matches!(self, Self::Horizontal)
    }

    pub fn horizontal(&self) -> bool {
        !matches!(self, Self::Vertical)
    }
}

#[derive(Debug, Clone)]
enum Offset {
    Relative(f32),
    Absolute(f32),
}

impl Default for Offset {
    fn default() -> Self {
        Self::Relative(0.0)
    }
}

impl Offset {
    fn relative(&self, mut size: f32, content_size: f32, scrollbar: bool) -> f32 {
        if scrollbar {
            size = (size - SCROLLBAR_THICKNESS).max(0.0)
        }
        match self {
            Offset::Relative(relative) => *relative,
            Offset::Absolute(absolute) => (absolute / (content_size - size)).clamp(0.0, 1.0),
        }
    }

    fn absolute(&self, mut size: f32, content_size: f32, scrollbar: bool) -> f32 {
        if scrollbar {
            size = (size - SCROLLBAR_THICKNESS).max(0.0)
        }
        match self {
            Offset::Absolute(absolute) => absolute.clamp(0.0, (content_size - size).max(0.0)),
            Offset::Relative(relative) => relative * scroll_space(size, content_size),
        }
    }
}

fn scroll_space(size: f32, content_size: f32) -> f32 {
    0.0f32.max(content_size - size)
}

#[derive(Default, Clone, Copy, Debug)]
pub enum Placement {
    Start,
    #[default]
    End,
}

#[derive(Default, Debug)]
pub struct State {
    offset_x: Offset,
    offset_y: Offset,
    mouse_grabbed_at_x: Option<f32>,
    mouse_grabbed_at_y: Option<f32>,
    keyboard_modifiers: iced::keyboard::Modifiers,
    direction: Direction,
    vertical_scrollbar_placement: Placement,
    horizontal_scrollbar_placement: Placement,
}

impl State {
    pub fn new(
        direction: Direction,
        horizontal_scrollbar_placement: Placement,
        vertical_scrollbar_placement: Placement,
    ) -> Self {
        Self {
            direction,
            vertical_scrollbar_placement,
            horizontal_scrollbar_placement,
            ..Default::default()
        }
    }

    pub fn vertical_scrollbar_placement(&mut self, placement: Placement) {
        self.vertical_scrollbar_placement = placement;
    }

    pub fn horizontal_scrollbar_placement(&mut self, placement: Placement) {
        self.horizontal_scrollbar_placement = placement;
    }

    pub fn direction(&mut self, direction: Direction) {
        self.direction = direction;
        if !direction.horizontal() {
            self.offset_x = Offset::Relative(0.0);
            self.mouse_grabbed_at_x = None;
        }
        if !direction.vertical() {
            self.offset_y = Offset::Relative(0.0);
            self.mouse_grabbed_at_y = None;
        }
    }

    fn horizontal_scroll_factor(&self, size: f32, content_size: f32) -> f32 {
        self.offset_x.relative(
            size,
            content_size,
            self.direction.horizontal() && size < content_size,
        )
    }

    fn vertical_scroll_factor(&self, size: f32, content_size: f32) -> f32 {
        self.offset_y.relative(
            size,
            content_size,
            self.direction.vertical() && size < content_size,
        )
    }

    pub fn scroll_by_x(&mut self, absolute_x: f32, size: f32, content_size: f32) {
        self.scroll_to_x(
            self.offset_x.absolute(
                size,
                content_size,
                self.direction.horizontal() && size < content_size,
            ) + absolute_x,
        );
    }

    pub fn scroll_by_y(&mut self, absolute_y: f32, size: f32, content_size: f32) {
        self.scroll_to_y(
            self.offset_y.absolute(
                size,
                content_size,
                self.direction.vertical() && size < content_size,
            ) + absolute_y,
        );
    }

    pub fn scroll_by(&mut self, offset: AbsoluteOffset, size: Size, content_size: Size) {
        self.scroll_by_x(offset.x, size.width, content_size.width);
        self.scroll_by_y(offset.y, size.height, content_size.height);
    }

    pub fn scroll_to_x(&mut self, absolute_x: f32) {
        self.offset_x = Offset::Absolute(absolute_x)
    }
    pub fn scroll_to_y(&mut self, absolute_y: f32) {
        self.offset_y = Offset::Absolute(absolute_y)
    }

    pub fn scroll_to(&mut self, offset: AbsoluteOffset) {
        self.scroll_to_x(offset.x);
        self.scroll_to_y(offset.y);
    }

    pub fn snap_to_x(&mut self, relative_x: f32) {
        self.offset_x = Offset::Relative(relative_x);
    }

    pub fn snap_to_y(&mut self, relative_y: f32) {
        self.offset_y = Offset::Relative(relative_y);
    }

    pub fn snap_to(&mut self, offset: RelativeOffset) {
        self.snap_to_x(offset.x);
        self.snap_to_y(offset.y);
    }

    pub fn translation(&self, size: Size, content_size: Size) -> iced::Vector {
        let horizontal_scroll = self.direction.horizontal() && size.width < content_size.width;
        let vertical_scroll = self.direction.vertical() && size.height < content_size.height;
        let scroll_x = self
            .offset_x
            .absolute(size.width, content_size.width, horizontal_scroll)
            - if vertical_scroll && matches!(self.vertical_scrollbar_placement, Placement::Start) {
                SCROLLBAR_THICKNESS
            } else {
                0.0
            };
        let scroll_y = self
            .offset_y
            .absolute(size.height, content_size.height, vertical_scroll)
            - if horizontal_scroll
                && matches!(self.horizontal_scrollbar_placement, Placement::Start)
            {
                SCROLLBAR_THICKNESS
            } else {
                0.0
            };
        iced::Vector::new(-scroll_x, -scroll_y)
    }
}

impl advanced::widget::operation::Scrollable for State {
    fn scroll_by(
        &mut self,
        offset: AbsoluteOffset,
        bounds: iced::Rectangle,
        content_bounds: iced::Rectangle,
    ) {
        self.scroll_by(offset, bounds.size(), content_bounds.size());
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset) {
        self.scroll_to(offset);
    }

    fn snap_to(&mut self, offset: iced::widget::scrollable::RelativeOffset) {
        self.snap_to(offset);
    }
}

pub fn layout(
    direction: Direction,
    limits: Limits,
    mut content_layout: impl FnMut(Limits) -> advanced::layout::Node,
) -> advanced::layout::Node {
    fn max_height(limits: Limits, max_height: f32) -> Limits {
        let mut max_size = limits.max();
        max_size.height = max_height;
        Limits::new(limits.min(), max_size)
    }

    fn max_width(limits: Limits, max_width: f32) -> Limits {
        let mut max_size = limits.max();
        max_size.width = max_width;
        Limits::new(limits.min(), max_size)
    }

    match direction {
        Direction::Both => {
            let child_layout = content_layout(Limits::new(Size::ZERO, Size::INFINITY));
            Node::with_children(
                Size::new(
                    child_layout.size().width.min(limits.max().width),
                    child_layout.size().height.min(limits.max().height),
                ),
                vec![child_layout],
            )
        }
        Direction::Vertical => {
            let first_layout = content_layout(max_height(limits, f32::INFINITY));
            if first_layout.bounds().height > limits.max().height {
                let child_layout = content_layout(max_width(
                    max_height(limits, f32::INFINITY),
                    limits.max().width - SCROLLBAR_THICKNESS,
                ));
                Node::with_children(
                    Size::new(
                        (child_layout.size().width + SCROLLBAR_THICKNESS).min(limits.max().width),
                        child_layout.size().height.min(limits.max().height),
                    ),
                    vec![child_layout],
                )
            } else {
                Node::with_children(
                    Size::new(
                        (first_layout.size().width).min(limits.max().width),
                        first_layout.size().height.min(limits.max().height),
                    ),
                    vec![first_layout],
                )
            }
        }
        Direction::Horizontal => {
            let first_layout = content_layout(max_width(limits, f32::INFINITY));
            if first_layout.bounds().width > limits.max().width {
                let child_layout = content_layout(max_height(
                    max_width(limits, f32::INFINITY),
                    limits.max().height - SCROLLBAR_THICKNESS,
                ));
                Node::with_children(
                    Size::new(
                        child_layout.size().width.min(limits.max().width),
                        (child_layout.size().height + SCROLLBAR_THICKNESS).min(limits.max().height),
                    ),
                    vec![child_layout],
                )
            } else {
                Node::with_children(
                    Size::new(
                        (first_layout.size().width).min(limits.max().width),
                        first_layout.size().height.min(limits.max().height),
                    ),
                    vec![first_layout],
                )
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw<Theme: Catalog, Renderer: advanced::Renderer>(
    state: &State,
    style: &Theme::Class<'_>,
    renderer: &mut Renderer,
    theme: &Theme,
    cursor: advanced::mouse::Cursor,
    content_size: Size,
    bounds: iced::Rectangle,
    viewport: &iced::Rectangle,
    draw_job: impl FnOnce(&mut Renderer, &iced::Rectangle, advanced::mouse::Cursor),
) {
    let Some(visible_bounds) = bounds.intersection(viewport) else {
        // return if nothing would be visible on the screen
        return;
    };

    // draw inner content
    let translation = state.translation(bounds.size(), content_size);
    renderer.with_layer(visible_bounds, |renderer| {
        renderer.with_translation(translation, |renderer| {
            (draw_job)(
                renderer,
                &iced::Rectangle {
                    x: viewport.x - translation.x,
                    y: viewport.y - translation.y,
                    ..*viewport
                },
                match cursor {
                    iced::mouse::Cursor::Available(point) => {
                        iced::mouse::Cursor::Available(point + translation * -1.0)
                    }
                    iced::mouse::Cursor::Unavailable => iced::mouse::Cursor::Unavailable,
                },
            )
        });
    });

    let horizontal_scrollbar = content_size.width > bounds.width && state.direction.horizontal();
    let vertical_scrollbar = content_size.height > bounds.height && state.direction.vertical();

    let (horizontal_scrollbar_bounds, vertical_scrollbar_bounds) = scrollbar_bounds(
        bounds,
        horizontal_scrollbar,
        state.horizontal_scrollbar_placement,
        vertical_scrollbar,
        state.vertical_scrollbar_placement,
    );
    let (horizontal_scroller_bounds, vertical_scroller_bounds) = scroller_bounds(
        state,
        content_size,
        bounds.size(),
        horizontal_scrollbar_bounds,
        vertical_scrollbar_bounds,
    );

    let style = theme.style(
        style,
        calculate_status(
            vertical_scrollbar_bounds,
            horizontal_scrollbar_bounds,
            cursor,
            state.mouse_grabbed_at_x.is_some(),
            state.mouse_grabbed_at_y.is_some(),
        ),
    );

    renderer.with_layer(bounds, |renderer| {
        if vertical_scrollbar && horizontal_scrollbar {
            renderer.fill_quad(
                advanced::renderer::Quad {
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    bounds: iced::Rectangle {
                        x: horizontal_scrollbar_bounds.width + horizontal_scrollbar_bounds.x,
                        y: vertical_scrollbar_bounds.height + vertical_scrollbar_bounds.y,
                        width: SCROLLBAR_THICKNESS,
                        height: SCROLLBAR_THICKNESS,
                    },
                },
                style
                    .gap
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
        }
        if horizontal_scrollbar {
            // draw horizontal scrollbar
            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: horizontal_scrollbar_bounds,
                    border: style.horizontal_rail.border,
                    shadow: iced::Shadow::default(),
                },
                style
                    .horizontal_rail
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
            // draw horizontal scroller
            renderer.fill_quad(
                advanced::renderer::Quad {
                    shadow: iced::Shadow::default(),
                    border: style.horizontal_rail.scroller.border,
                    bounds: horizontal_scroller_bounds,
                },
                style.horizontal_rail.scroller.color,
            );
        }
        if vertical_scrollbar {
            // draw vertical scrollbar
            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds: vertical_scrollbar_bounds,
                    border: style.vertical_rail.border,
                    shadow: iced::Shadow::default(),
                },
                style
                    .horizontal_rail
                    .background
                    .unwrap_or(iced::Background::Color(iced::Color::TRANSPARENT)),
            );
            // draw vertical scroller
            renderer.fill_quad(
                advanced::renderer::Quad {
                    shadow: iced::Shadow::default(),
                    border: style.vertical_rail.scroller.border,
                    bounds: vertical_scroller_bounds,
                },
                style.vertical_rail.scroller.color,
            );
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn mouse_interaction<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    bounds: iced::Rectangle,
    content_size: Size,
    widget: &dyn advanced::Widget<Message, Theme, Renderer>,
    tree: &advanced::widget::Tree,
    layout: advanced::Layout<'_>,
    cursor: advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
    renderer: &Renderer,
) -> advanced::mouse::Interaction {
    if state.mouse_grabbed_at_x.is_some() || state.mouse_grabbed_at_y.is_some() {
        return advanced::mouse::Interaction::Idle;
    } else if let iced::mouse::Cursor::Available(position) = cursor {
        let scrollbar_bounds = scrollbar_bounds(
            bounds,
            bounds.width < content_size.width,
            state.horizontal_scrollbar_placement,
            bounds.height < content_size.height,
            state.vertical_scrollbar_placement,
        );
        let scroller_bounds = scroller_bounds(
            state,
            content_size,
            bounds.size(),
            scrollbar_bounds.0,
            scrollbar_bounds.1,
        );
        if scroller_bounds.0.contains(position) || scroller_bounds.1.contains(position) {
            return advanced::mouse::Interaction::Idle;
        }
    }

    widget.mouse_interaction(
        tree,
        layout,
        if let iced::mouse::Cursor::Available(point) = cursor {
            iced::mouse::Cursor::Available(
                point + state.translation(bounds.size(), content_size) * -1.0,
            )
        } else {
            cursor
        },
        &(*viewport + state.translation(bounds.size(), content_size) * -1.0),
        renderer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn on_event<Message, Renderer: advanced::Renderer, Theme>(
    state: &State,
    widget: &mut dyn advanced::Widget<Message, Theme, Renderer>,
    size: Size,
    content_size: Size,
    widget_tree: &mut advanced::widget::Tree,
    event: iced::Event,
    widget_layout: advanced::Layout<'_>,
    cursor: advanced::mouse::Cursor,
    renderer: &Renderer,
    clipboard: &mut dyn advanced::Clipboard,
    shell: &mut advanced::Shell<'_, Message>,
    viewport: &iced::Rectangle,
) -> advanced::graphics::core::event::Status {
    widget.on_event(
        widget_tree,
        event,
        widget_layout,
        if let iced::mouse::Cursor::Available(point) = cursor {
            iced::mouse::Cursor::Available(point + state.translation(size, content_size) * -1.0)
        } else {
            cursor
        },
        renderer,
        clipboard,
        shell,
        &(*viewport - state.translation(size, content_size)),
    )
}

pub fn scroll_wheel_on_event(
    state: &mut State,
    event: iced::Event,
    cursor: advanced::mouse::Cursor,
    bounds: iced::Rectangle,
    content_size: Size,
) -> advanced::graphics::core::event::Status {
    if let Some(position) = cursor.position() {
        if !bounds.contains(position) {
            return advanced::graphics::core::event::Status::Ignored;
        }
    } else {
        return advanced::graphics::core::event::Status::Ignored;
    };
    match event {
        iced::Event::Mouse(iced::mouse::Event::WheelScrolled {
            delta: iced::mouse::ScrollDelta::Lines { mut y, mut x },
        }) => {
            if state.keyboard_modifiers.shift() && !cfg!(target_os = "macos") {
                (x, y) = (y, x);
            }
            y *= -30.0;
            x *= -30.0;
            state.scroll_by(AbsoluteOffset { x, y }, bounds.size(), content_size);
            return advanced::graphics::core::event::Status::Captured;
        }
        iced::Event::Keyboard(keyboard_event) => {
            state.keyboard_modifiers = match keyboard_event {
                iced::keyboard::Event::KeyPressed { modifiers, .. } => modifiers,
                iced::keyboard::Event::KeyReleased { modifiers, .. } => modifiers,
                iced::keyboard::Event::ModifiersChanged(modifiers) => modifiers,
            };
        }
        _ => {}
    }
    advanced::graphics::core::event::Status::Ignored
}

pub fn scroll_grab_on_event(
    state: &mut State,
    event: iced::Event,
    cursor: advanced::mouse::Cursor,
    bounds: iced::Rectangle,
    content_size: Size,
) -> advanced::graphics::core::event::Status {
    let scrollbar_bounds = scrollbar_bounds(
        bounds,
        content_size.width > bounds.width,
        state.horizontal_scrollbar_placement,
        content_size.height > bounds.height,
        state.vertical_scrollbar_placement,
    );
    let scroller_bounds = scroller_bounds(
        state,
        content_size,
        bounds.size(),
        scrollbar_bounds.0,
        scrollbar_bounds.1,
    );

    // mouse drag movement
    if state.mouse_grabbed_at_x.is_some() || state.mouse_grabbed_at_y.is_some() {
        let new_position = if let iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) =
            event
        {
            Some(position)
        } else if let iced::Event::Touch(iced::touch::Event::FingerMoved { position, .. }) = event {
            Some(position)
        } else {
            None
        };
        if let Some(position) = new_position {
            if let Some(x) = state.mouse_grabbed_at_x {
                let diff = position.x - x;

                state.scroll_by(
                    AbsoluteOffset {
                        x: scroll_space(bounds.size().width, content_size.width)
                            * (diff / (scrollbar_bounds.0.width - scroller_bounds.0.width)),
                        y: 0.0,
                    },
                    bounds.size(),
                    content_size,
                );
                state.mouse_grabbed_at_x = Some(position.x);
                return advanced::graphics::core::event::Status::Captured;
            }
            if let Some(y) = state.mouse_grabbed_at_y {
                let diff = position.y - y;

                state.scroll_by(
                    AbsoluteOffset {
                        x: 0.0,
                        y: scroll_space(bounds.height, content_size.height)
                            * (diff / (scrollbar_bounds.1.height - scroller_bounds.1.height)),
                    },
                    bounds.size(),
                    content_size,
                );
                state.mouse_grabbed_at_y = Some(position.y);
                return advanced::graphics::core::event::Status::Captured;
            }
        }
    }

    if let iced::Event::Mouse(iced::mouse::Event::ButtonReleased(button)) = event {
        if button == iced::mouse::Button::Left
            || matches!(
                event,
                iced::Event::Touch(iced::touch::Event::FingerLifted { .. })
            )
        {
            state.mouse_grabbed_at_x = None;
            state.mouse_grabbed_at_y = None;
        }
    }

    let mut clicked_position = None;
    if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(button)) = event {
        if button == iced::mouse::Button::Left {
            if let Some(position) = cursor.position() {
                if bounds.contains(position) {
                    clicked_position = Some(position);
                }
            }
        }
    } else if let iced::Event::Touch(iced::touch::Event::FingerPressed { position, .. }) = event {
        clicked_position = Some(position);
    }

    if let Some(clicked_position) = clicked_position {
        if state.direction.horizontal() && scroller_bounds.0.contains(clicked_position) {
            state.mouse_grabbed_at_x = Some(clicked_position.x);
            return advanced::graphics::core::event::Status::Captured;
        } else if state.direction.vertical() && scroller_bounds.1.contains(clicked_position) {
            state.mouse_grabbed_at_y = Some(clicked_position.y);
            return advanced::graphics::core::event::Status::Captured;
        } else if state.direction.horizontal() && scrollbar_bounds.0.contains(clicked_position) {
            // calculate smaller size -> the size that is relevant for the click
            let limited_size = scrollbar_bounds.0.shrink(
                iced::Padding::ZERO
                    .left(scroller_bounds.0.size().width / 2.0)
                    .right(scroller_bounds.0.size().width / 2.0),
            );
            // calculate the new relative offset
            let relative = (clicked_position.x.clamp(
                limited_size.position().x,
                limited_size.position().x + limited_size.width,
            ) - limited_size.position().x)
                / limited_size.width;
            // set the new offset
            state.snap_to(RelativeOffset {
                x: relative,
                y: state.vertical_scroll_factor(bounds.width, content_size.width),
            });
            return advanced::graphics::core::event::Status::Captured;
        } else if state.direction.vertical() && scrollbar_bounds.1.contains(clicked_position) {
            // calculate smaller size -> the size that is relevant wor the click
            let limited_size = scrollbar_bounds.1.shrink(
                iced::Padding::ZERO
                    .top(scroller_bounds.1.size().height / 2.0)
                    .bottom(scroller_bounds.1.size().height / 2.0),
            );
            // calculate the new relative offset
            let relative = (clicked_position.y.clamp(
                limited_size.position().y,
                limited_size.position().y + limited_size.height,
            ) - limited_size.position().y)
                / limited_size.height;
            // set the new offset
            state.snap_to(RelativeOffset {
                x: state.horizontal_scroll_factor(bounds.height, content_size.height),
                y: relative,
            });
            return advanced::graphics::core::event::Status::Captured;
        }
    }

    advanced::graphics::core::event::Status::Ignored
}

#[allow(clippy::too_many_arguments)]
pub fn overlay<'b, Message, Theme, Renderer: advanced::Renderer>(
    state: &State,
    size: Size,
    content_size: Size,
    widget: &'b mut dyn advanced::Widget<Message, Theme, Renderer>,
    widget_state: &'b mut advanced::widget::Tree,
    widget_layout: advanced::Layout<'_>,
    renderer: &Renderer,
    translation: iced::Vector,
) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
    widget.overlay(
        widget_state,
        widget_layout,
        renderer,
        translation + state.translation(size, content_size),
    )
}

fn calculate_status(
    vertical_scrollbar_bounds: iced::Rectangle,
    horizontal_scrollbar_bounds: iced::Rectangle,
    cursor: iced::mouse::Cursor,
    mouse_drag_x: bool,
    mouse_drag_y: bool,
) -> Status {
    if let iced::mouse::Cursor::Available(position) = cursor {
        if mouse_drag_y {
            return Status::Dragged {
                is_horizontal_scrollbar_dragged: false,
                is_vertical_scrollbar_dragged: true,
            };
        } else if mouse_drag_x {
            return Status::Dragged {
                is_horizontal_scrollbar_dragged: true,
                is_vertical_scrollbar_dragged: false,
            };
        }
        if vertical_scrollbar_bounds.contains(position) {
            return Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
            };
        } else if horizontal_scrollbar_bounds.contains(position) {
            return Status::Hovered {
                is_horizontal_scrollbar_hovered: true,
                is_vertical_scrollbar_hovered: false,
            };
        }
    }
    Status::Active
}

fn scrollbar_bounds(
    bounds: iced::Rectangle,
    horizontal_scrollbar: bool,
    horizontal_scrollbar_placement: Placement,
    vertical_scrollbar: bool,
    vertical_scrollbar_placement: Placement,
) -> (iced::Rectangle, iced::Rectangle) {
    (
        iced::Rectangle {
            x: bounds.x
                + if vertical_scrollbar && matches!(vertical_scrollbar_placement, Placement::Start)
                {
                    SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            y: bounds.y
                + if matches!(horizontal_scrollbar_placement, Placement::End) {
                    bounds.height - SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            width: bounds.width
                - if vertical_scrollbar {
                    SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            height: SCROLLBAR_THICKNESS,
        },
        iced::Rectangle {
            x: bounds.x
                + if matches!(vertical_scrollbar_placement, Placement::End) {
                    bounds.width - SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            y: bounds.y
                + if horizontal_scrollbar
                    && matches!(horizontal_scrollbar_placement, Placement::Start)
                {
                    SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
            width: SCROLLBAR_THICKNESS,
            height: bounds.height
                - if horizontal_scrollbar {
                    SCROLLBAR_THICKNESS
                } else {
                    0.0
                },
        },
    )
}

fn scroller_bounds(
    state: &State,
    content_size: Size,
    size: Size,
    horizontal_scrollbar_bounds: iced::Rectangle,
    vertical_scrollbar_bounds: iced::Rectangle,
) -> (iced::Rectangle, iced::Rectangle) {
    (
        {
            // horizontal scroller
            let (scroller_start, scroller_end) = scroller_position(
                state.horizontal_scroll_factor(size.width, content_size.width),
                horizontal_scrollbar_bounds.x,
                horizontal_scrollbar_bounds.x + horizontal_scrollbar_bounds.width,
                content_size.width,
            );
            iced::Rectangle {
                height: horizontal_scrollbar_bounds.height,
                width: scroller_end - scroller_start,
                y: horizontal_scrollbar_bounds.y,
                x: scroller_start,
            }
        },
        {
            // vertical scroller
            let (scroller_start, scroller_end) = scroller_position(
                state.vertical_scroll_factor(size.height, content_size.height),
                vertical_scrollbar_bounds.y,
                vertical_scrollbar_bounds.y + vertical_scrollbar_bounds.height,
                content_size.height,
            );
            iced::Rectangle {
                height: scroller_end - scroller_start,
                width: vertical_scrollbar_bounds.width,
                y: scroller_start,
                x: vertical_scrollbar_bounds.x,
            }
        },
    )
}

fn scroller_position(factor: f32, start: f32, end: f32, content_size: f32) -> (f32, f32) {
    let size = end - start;
    let scroller_size = 30.0f32.max(size * 1.0f32.min(size / content_size));

    let start_pos = start + (size - scroller_size) * factor;
    (start_pos, start_pos + scroller_size)
}

pub fn x_start_end(layouts: &Vec<advanced::Layout<'_>>) -> (f32, f32) {
    let mut layout_iter = layouts.iter();
    let (mut start, mut end) = if let Some(first) = layout_iter.next() {
        (first.position().x, first.position().x)
    } else {
        return (0.0, 0.0);
    };
    for layout in layout_iter {
        start = start.min(layout.position().x);
        end = end.max(layout.position().x + layout.bounds().width);
    }
    (start, end)
}

pub fn y_start_end(layouts: &Vec<advanced::Layout<'_>>) -> (f32, f32) {
    let mut layout_iter = layouts.iter();
    let (mut start, mut end) = if let Some(first) = layout_iter.next() {
        (first.position().y, first.position().y)
    } else {
        return (0.0, 0.0);
    };
    for layout in layout_iter {
        start = start.min(layout.position().y);
        end = end.max(layout.position().y + layout.bounds().height);
    }
    (start, end)
}
